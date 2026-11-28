// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-1f (P0-6 §6.2) — `cargo xtask trace-diff`: đối chiếu bộ 3 vết vàng (`tsf`, `linux`, `mac`)
//! trên 58 ca `corpus/shared/`, chặn mọi khác biệt không có quy tắc được duyệt trong
//! `corpus/traces/ALLOWED_DIFF.md`, đồng thời gate chống trôi 136 tệp vết trong `corpus/traces/tsf/`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::check_mac_targets::{self, Json};

const TSF_DIR: &str = "corpus/traces/tsf/shared";
const LINUX_DIR: &str = "corpus/traces/linux/shared";
/// Gốc vết Linux (`trace_compose.c`) — SF-3 so với vết `replay --adapter linux` qua session.
const LINUX_ALL_DIR: &str = "corpus/traces/linux";
const MAC_DIR: &str = "corpus/traces/mac/shared";
const TSF_ALL_DIR: &str = "corpus/traces/tsf";
const ALLOWED_MD: &str = "corpus/traces/ALLOWED_DIFF.md";

const KNOWN_RULES: &[(&str, &str)] = &[
    ("CUAS_PRINTABLE_BOUNDARY", "tsf-linux"),
    ("MAC_IMK_PREEDIT_LIFECYCLE", "linux-mac"),
    ("MAC_PRINTABLE_BOUNDARY_COMMIT", "linux-mac"),
    ("MAC_BACKSPACE_TYPE_CLEAR_ORDER", "linux-mac"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceCmd {
    DeleteBefore(u32),
    Commit(String),
    SetPreedit(String, u32),
    ForwardKey,
    EatKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedRule {
    pub id: String,
    pub pair: String,
}

fn json_str_field<'a>(obj: &'a Json, key: &str, ctx: &str) -> Result<&'a str, String> {
    obj.get(key)
        .and_then(Json::as_str)
        .ok_or_else(|| format!("{ctx}: thiếu hoặc sai kiểu chuỗi `{key}`"))
}

fn json_u32_field(obj: &Json, key: &str, ctx: &str) -> Result<u32, String> {
    let n = obj
        .get(key)
        .and_then(Json::as_num)
        .ok_or_else(|| format!("{ctx}: thiếu hoặc sai kiểu số `{key}`"))?;
    if n < 0.0 || n.fract() != 0.0 || n > u32::MAX as f64 {
        return Err(format!("{ctx}: `{key}`={n} không phải u32 hợp lệ"));
    }
    Ok(n as u32)
}

pub fn parse_trace_jsonl(
    rel_case: &str,
    content: &str,
    strict_commit_before_hide: bool,
) -> Result<BTreeMap<u32, Vec<TraceCmd>>, String> {
    let mut by_step: BTreeMap<u32, Vec<TraceCmd>> = BTreeMap::new();
    for (idx, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let ctx = format!("{rel_case}:{}", idx + 1);
        let obj =
            check_mac_targets::parse_json(line).map_err(|e| format!("{ctx}: JSON lỗi: {e}"))?;
        let case_field = json_str_field(&obj, "case", &ctx)?;
        if case_field != rel_case {
            return Err(format!(
                "{ctx}: trường `case` (`{case_field}`) != tên tệp (`{rel_case}`)"
            ));
        }
        let step = json_u32_field(&obj, "step", &ctx)?;
        let cmd_name = json_str_field(&obj, "cmd", &ctx)?;
        let text = json_str_field(&obj, "text", &ctx)?;
        let n = json_u32_field(&obj, "n", &ctx)?;

        let cmd = match cmd_name {
            "delete_before" => {
                if n == 0 {
                    return Err(format!("{ctx}: `delete_before` phải có n > 0"));
                }
                TraceCmd::DeleteBefore(n)
            }
            "commit" => TraceCmd::Commit(text.to_string()),
            "set_preedit" => TraceCmd::SetPreedit(text.to_string(), n),
            "forward_key" => TraceCmd::ForwardKey,
            "eat_key" => TraceCmd::EatKey,
            other => return Err(format!("{ctx}: lệnh lạ `{other}` ngoài từ vựng 5 lệnh")),
        };
        by_step.entry(step).or_default().push(cmd);
    }
    if by_step.is_empty() {
        return Err(format!("{rel_case}: tệp vết rỗng"));
    }

    for (step, cmds) in &by_step {
        let dels = cmds
            .iter()
            .filter(|c| matches!(c, TraceCmd::DeleteBefore(_)))
            .count();
        let commits = cmds
            .iter()
            .filter(|c| matches!(c, TraceCmd::Commit(_)))
            .count();
        if dels > 1 {
            return Err(format!("{rel_case} step {step}: quá 1 delete_before"));
        }
        if commits > 1 {
            return Err(format!("{rel_case} step {step}: quá 1 commit"));
        }
        if strict_commit_before_hide {
            if let (Some(c_idx), Some(h_idx)) = (
                cmds.iter().position(|c| matches!(c, TraceCmd::Commit(_))),
                cmds.iter()
                    .position(|c| matches!(c, TraceCmd::SetPreedit(t, 0) if t.is_empty())),
            ) {
                if c_idx > h_idx {
                    return Err(format!(
                        "{rel_case} step {step}: `commit` phải đứng TRƯỚC ẩn preedit (P0-6 §3)"
                    ));
                }
            }
        }
    }
    Ok(by_step)
}

pub fn parse_allowed_diff_md(md: &str) -> Result<Vec<AllowedRule>, String> {
    let mut rules = Vec::new();
    let mut seen = BTreeSet::new();
    for (idx, line) in md.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cols.len() != 4 {
            continue;
        }
        let id = cols[0].trim_matches('`').trim();
        let pair = cols[1].trim_matches('`').trim();
        let scope = cols[2].trim();
        let approver = cols[3].trim_matches('`').trim();
        if id == "ID" || id.chars().all(|c| c == '-' || c == ':') {
            continue;
        }
        let Some((_, expected_pair)) = KNOWN_RULES.iter().find(|(k, _)| *k == id) else {
            return Err(format!(
                "ALLOWED_DIFF.md:{}: quy tắc lạ `{id}` chưa được matcher trong trace_diff.rs hỗ trợ",
                idx + 1
            ));
        };
        if pair != *expected_pair {
            return Err(format!(
                "ALLOWED_DIFF.md:{}: `{id}` phải có Pair=`{expected_pair}`, thấy `{pair}`",
                idx + 1
            ));
        }
        if scope.len() < 12 || approver.is_empty() {
            return Err(format!(
                "ALLOWED_DIFF.md:{}: `{id}` thiếu lý do kỹ thuật hoặc người duyệt",
                idx + 1
            ));
        }
        if !seen.insert(id.to_string()) {
            return Err(format!("ALLOWED_DIFF.md:{}: trùng ID `{id}`", idx + 1));
        }
        rules.push(AllowedRule {
            id: id.to_string(),
            pair: pair.to_string(),
        });
    }
    if rules.is_empty() {
        return Err("ALLOWED_DIFF.md: không tìm thấy bảng quy tắc nào".into());
    }
    Ok(rules)
}

/// Kiểm tra khác biệt TSF vs Linux trên cùng 1 bước: chỉ cho phép `CUAS_PRINTABLE_BOUNDARY`.
fn match_tsf_linux_rule(tsf: &[TraceCmd], linux: &[TraceCmd]) -> Option<&'static str> {
    match (tsf, linux) {
        (
            [TraceCmd::Commit(tc), TraceCmd::SetPreedit(tp, 0), TraceCmd::EatKey],
            [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::ForwardKey],
        ) if tp.is_empty() && lp.is_empty() => {
            let suffix = tc.strip_prefix(lc.as_str())?;
            if suffix.chars().count() == 1 {
                Some("CUAS_PRINTABLE_BOUNDARY")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Kiểm tra khác biệt Linux vs macOS trên cùng 1 bước theo bộ quy tắc hẹp P0-6 §6.2.
fn match_linux_mac_rule(linux: &[TraceCmd], mac: &[TraceCmd]) -> Option<&'static str> {
    // 1. MAC_IMK_PREEDIT_LIFECYCLE:
    //    - Ký tự thô chưa biến đổi được Mac cho `forward_key` tới khi bắt đầu biến đổi (Linux giữ preedit từ phím 1).
    //    - Khi bắt đầu biến đổi phần chữ đã pass, Mac `delete_before(d)` rồi `set_preedit(p, n)` với `p` trùng Linux.
    //    - Khi đổi preedit đang có (`preeditReplace`), Mac thu hồi `set_preedit("", 0)` rồi `set_preedit(p, n)` trùng Linux.
    if let [TraceCmd::SetPreedit(lp, ln), TraceCmd::EatKey] = linux {
        match mac {
            [TraceCmd::ForwardKey] => return Some("MAC_IMK_PREEDIT_LIFECYCLE"),
            [TraceCmd::Commit(mc), TraceCmd::EatKey]
                if mc == lp && *ln == mc.chars().count() as u32 =>
            {
                return Some("MAC_IMK_PREEDIT_LIFECYCLE");
            }
            [TraceCmd::DeleteBefore(dm), TraceCmd::SetPreedit(mp, mn), TraceCmd::EatKey]
                if *dm > 0 && mp == lp && mn == ln =>
            {
                return Some("MAC_IMK_PREEDIT_LIFECYCLE");
            }
            [TraceCmd::SetPreedit(mh, 0), TraceCmd::SetPreedit(mp, mn), TraceCmd::EatKey]
                if mh.is_empty() && mp == lp && mn == ln =>
            {
                return Some("MAC_IMK_PREEDIT_LIFECYCLE");
            }
            _ => {}
        }
    }
    if let [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lh, 0), TraceCmd::SetPreedit(np, 1), TraceCmd::EatKey] =
        linux
    {
        if lh.is_empty() && !np.is_empty() {
            match mac {
                [TraceCmd::Commit(mc), TraceCmd::SetPreedit(mh, 0), TraceCmd::ForwardKey]
                    if mh.is_empty() && lc == mc =>
                {
                    return Some("MAC_IMK_PREEDIT_LIFECYCLE");
                }
                [TraceCmd::ForwardKey] if lc.chars().count() == 1 && lc.is_ascii() => {
                    return Some("MAC_IMK_PREEDIT_LIFECYCLE");
                }
                _ => {}
            }
        }
    }

    // 2. MAC_PRINTABLE_BOUNDARY_COMMIT:
    //    - Ranh giới từ in được: Linux commit `word` + `forward_key`; Mac (giống TSF) commit `word + boundary` + `eat_key`
    //      (cả trường hợp `.commit` với `Commit + SetPreedit("")` lẫn `.restore` ở ranh giới với `SetPreedit("") + Commit`).
    //    - Hoặc khi từ hoàn toàn pass-through trên Mac (Linux giữ preedit rồi flush), Mac chỉ `forward_key`.
    match (linux, mac) {
        (
            [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::ForwardKey],
            [TraceCmd::Commit(mc), TraceCmd::SetPreedit(mp, 0), TraceCmd::EatKey],
        )
        | (
            [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::ForwardKey],
            [TraceCmd::SetPreedit(mp, 0), TraceCmd::Commit(mc), TraceCmd::EatKey],
        ) if lp.is_empty() && mp.is_empty() => {
            if let Some(suffix) = mc.strip_prefix(lc.as_str()) {
                if suffix.chars().count() == 1 {
                    return Some("MAC_PRINTABLE_BOUNDARY_COMMIT");
                }
            }
        }
        (
            [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::ForwardKey],
            [TraceCmd::ForwardKey],
        ) if lp.is_empty() && !lc.is_empty() => {
            return Some("MAC_PRINTABLE_BOUNDARY_COMMIT");
        }
        _ => {}
    }

    // 3. MAC_BACKSPACE_TYPE_CLEAR_ORDER:
    //    `ApplyReplace.backspaceType` (restore / macro / emoji / auto-cap / `--`) thu hồi `set_preedit("", 0)` TRƯỚC
    //    khi xoá phần đã pass + `commit(T)`; hoặc xoá ký tự đã pass khi không mở preedit. `T` phải trùng Linux 100%!
    match (linux, mac) {
        (
            [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::EatKey],
            [TraceCmd::SetPreedit(mp, 0), TraceCmd::Commit(mc), TraceCmd::EatKey],
        ) if lp.is_empty() && mp.is_empty() && lc == mc => {
            return Some("MAC_BACKSPACE_TYPE_CLEAR_ORDER");
        }
        (
            [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::EatKey],
            [TraceCmd::SetPreedit(mp, 0), TraceCmd::DeleteBefore(dm), TraceCmd::Commit(mc), TraceCmd::EatKey],
        ) if lp.is_empty() && mp.is_empty() && *dm > 0 && lc == mc => {
            return Some("MAC_BACKSPACE_TYPE_CLEAR_ORDER");
        }
        (
            [TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::EatKey],
            [TraceCmd::DeleteBefore(dm), TraceCmd::Commit(mc), TraceCmd::EatKey],
        ) if lp.is_empty() && *dm > 0 && lc == mc => {
            return Some("MAC_BACKSPACE_TYPE_CLEAR_ORDER");
        }
        (
            [TraceCmd::DeleteBefore(dl), TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0), TraceCmd::EatKey],
            [TraceCmd::DeleteBefore(dm), TraceCmd::Commit(mc), TraceCmd::EatKey],
        ) if lp.is_empty() && *dm >= *dl && lc == mc => {
            return Some("MAC_BACKSPACE_TYPE_CLEAR_ORDER");
        }
        (
            [TraceCmd::ForwardKey, TraceCmd::Commit(lc), TraceCmd::SetPreedit(lp, 0)],
            [TraceCmd::ForwardKey, TraceCmd::SetPreedit(mp, 0), TraceCmd::Commit(mc)],
        ) if lp.is_empty() && mp.is_empty() && lc == mc => {
            return Some("MAC_BACKSPACE_TYPE_CLEAR_ORDER");
        }
        _ => {}
    }

    None
}

/// Độ dài preedit sau một bước: `n` của `set_preedit` CUỐI trong bước, không có thì giữ nguyên.
fn preedit_len_after(prev: u32, cmds: &[TraceCmd]) -> u32 {
    cmds.iter().fold(prev, |acc, c| match c {
        TraceCmd::SetPreedit(_, n) => *n,
        _ => acc,
    })
}

fn delete_count(cmds: &[TraceCmd]) -> u32 {
    cmds.iter()
        .find_map(|c| match c {
            TraceCmd::DeleteBefore(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(0)
}

fn commit_text(cmds: &[TraceCmd]) -> Option<&str> {
    cmds.iter().find_map(|c| match c {
        TraceCmd::Commit(t) => Some(t.as_str()),
        _ => None,
    })
}

/// E96 — kiểm GIÁ TRỊ (không chỉ hình dạng) cho bước Linux↔macOS đã khớp một quy tắc — audit SF-1f
/// (claude-opus-7e41b2d9): matcher cũ nhận `delete_before` bất kỳ số > 0 và ký tự ranh giới bất kỳ.
/// - `delete_before(dm)` của macOS phải xoá ĐÚNG phần chữ nó đã cho đi thẳng: `dl` (Linux xoá ở bước
///   này) + (preedit Linux trước bước − preedit macOS trước bước). Lệch = xoá lẹm/thiếu chữ người dùng.
/// - Ranh giới từ: TSF (CUAS) và macOS cùng commit `word + boundary` cho CÙNG phím ⇒ hai chuỗi commit
///   phải giống hệt nhau.
fn check_rule_values(
    rule: &str,
    tv: &[TraceCmd],
    tsf_rule: Option<&str>,
    lv: &[TraceCmd],
    mv: &[TraceCmd],
    linux_prev_len: u32,
    mac_prev_len: u32,
) -> Result<(), String> {
    if mv.iter().any(|c| matches!(c, TraceCmd::DeleteBefore(_))) {
        let want = delete_count(lv) + linux_prev_len.saturating_sub(mac_prev_len);
        let got = delete_count(mv);
        if got != want {
            return Err(format!(
                "`{rule}`: macOS delete_before({got}) != {want} (Linux xoá {} + đã cho đi thẳng {} ký tự)",
                delete_count(lv),
                linux_prev_len.saturating_sub(mac_prev_len)
            ));
        }
    }
    if rule == "MAC_PRINTABLE_BOUNDARY_COMMIT" && tsf_rule == Some("CUAS_PRINTABLE_BOUNDARY") {
        if let (Some(tc), Some(mc)) = (commit_text(tv), commit_text(mv)) {
            if tc != mc {
                return Err(format!(
                    "`{rule}`: macOS commit {mc:?} != TSF commit {tc:?} (cùng phím ranh giới)"
                ));
            }
        }
    }
    Ok(())
}

fn collect_trace_map(dir: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    if !dir.is_dir() {
        return Err(format!("thiếu thư mục vết: {}", dir.display()));
    }
    let mut map = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let entries = fs::read_dir(&cur).map_err(|e| format!("đọc {}: {e}", cur.display()))?;
        for entry in entries {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                let rel = p
                    .strip_prefix(dir)
                    .map_err(|e| e.to_string())?
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/");
                map.insert(rel, p);
            }
        }
    }
    if map.is_empty() {
        return Err(format!("E94: {} không có tệp `.jsonl` nào", dir.display()));
    }
    Ok(map)
}

#[derive(Debug, Default)]
pub struct DiffSummary {
    pub cases: usize,
    pub steps_compared: usize,
    pub exact_tl_steps: usize,
    pub exact_lm_steps: usize,
    pub rule_hits: BTreeMap<String, usize>,
}

pub fn compare_roots(
    tsf_root: &Path,
    linux_root: &Path,
    mac_root: &Path,
    allowed_md_path: &Path,
) -> Result<DiffSummary, String> {
    let md = fs::read_to_string(allowed_md_path)
        .map_err(|e| format!("đọc {}: {e}", allowed_md_path.display()))?;
    let rules = parse_allowed_diff_md(&md)?;
    let approved_ids: BTreeSet<String> = rules.iter().map(|r| r.id.clone()).collect();

    let tsf_map = collect_trace_map(tsf_root)?;
    let linux_map = collect_trace_map(linux_root)?;
    let mac_map = collect_trace_map(mac_root)?;

    let tsf_keys: BTreeSet<&String> = tsf_map.keys().collect();
    let linux_keys: BTreeSet<&String> = linux_map.keys().collect();
    let mac_keys: BTreeSet<&String> = mac_map.keys().collect();
    if tsf_keys != linux_keys || linux_keys != mac_keys {
        return Err(format!(
            "bộ ca `shared` của 3 adapter không khớp nhau: tsf={} linux={} mac={}",
            tsf_keys.len(),
            linux_keys.len(),
            mac_keys.len()
        ));
    }

    let mut summary = DiffSummary {
        cases: tsf_keys.len(),
        ..Default::default()
    };
    for id in &approved_ids {
        summary.rule_hits.insert(id.clone(), 0);
    }
    let mut unapproved: Vec<String> = Vec::new();

    for name in tsf_keys {
        let case_rel = format!("shared/{name}");
        let t_raw = fs::read_to_string(&tsf_map[name]).map_err(|e| e.to_string())?;
        let l_raw = fs::read_to_string(&linux_map[name]).map_err(|e| e.to_string())?;
        let m_raw = fs::read_to_string(&mac_map[name]).map_err(|e| e.to_string())?;

        let t_steps = parse_trace_jsonl(&case_rel, &t_raw, true)?;
        let l_steps = parse_trace_jsonl(&case_rel, &l_raw, true)?;
        let m_steps = parse_trace_jsonl(&case_rel, &m_raw, false)?;

        let mut all_steps: BTreeSet<u32> = BTreeSet::new();
        all_steps.extend(t_steps.keys().copied());
        all_steps.extend(l_steps.keys().copied());
        all_steps.extend(m_steps.keys().copied());

        let (mut linux_len, mut mac_len) = (0u32, 0u32);
        for step in all_steps {
            summary.steps_compared += 1;
            let tv = t_steps.get(&step).map(Vec::as_slice).unwrap_or(&[]);
            let lv = l_steps.get(&step).map(Vec::as_slice).unwrap_or(&[]);
            let mv = m_steps.get(&step).map(Vec::as_slice).unwrap_or(&[]);

            let tsf_rule = if tv == lv {
                None
            } else {
                match_tsf_linux_rule(tv, lv)
            };
            if tv == lv {
                summary.exact_tl_steps += 1;
            } else if let Some(rule) = tsf_rule {
                if approved_ids.contains(rule) {
                    *summary.rule_hits.entry(rule.to_string()).or_insert(0) += 1;
                } else {
                    unapproved.push(format!(
                        "{case_rel}#step={step} [tsf vs linux]: khớp `{rule}` nhưng chưa được duyệt trong ALLOWED_DIFF.md"
                    ));
                }
            } else {
                unapproved.push(format!(
                    "{case_rel}#step={step} [tsf vs linux]: TSF={tv:?} vs Linux={lv:?}"
                ));
            }

            if lv == mv {
                summary.exact_lm_steps += 1;
            } else if let Some(rule) = match_linux_mac_rule(lv, mv) {
                if let Err(e) = check_rule_values(rule, tv, tsf_rule, lv, mv, linux_len, mac_len) {
                    unapproved.push(format!("{case_rel}#step={step} [linux vs mac]: {e}"));
                } else if approved_ids.contains(rule) {
                    *summary.rule_hits.entry(rule.to_string()).or_insert(0) += 1;
                } else {
                    unapproved.push(format!(
                        "{case_rel}#step={step} [linux vs mac]: khớp `{rule}` nhưng chưa được duyệt trong ALLOWED_DIFF.md"
                    ));
                }
            } else {
                unapproved.push(format!(
                    "{case_rel}#step={step} [linux vs mac]: Linux={lv:?} vs Mac={mv:?}"
                ));
            }
            linux_len = preedit_len_after(linux_len, lv);
            mac_len = preedit_len_after(mac_len, mv);
        }
    }

    if !unapproved.is_empty() {
        let sample = unapproved
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ");
        return Err(format!(
            "SF-1f trace-diff: phát hiện {} khác biệt chưa nằm trong ALLOWED_DIFF.md:\n  {sample}",
            unapproved.len()
        ));
    }

    let stale: Vec<&String> = summary
        .rule_hits
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(id, _)| id)
        .collect();
    if !stale.is_empty() {
        return Err(format!(
            "SF-1f ALLOWED_DIFF.md có quy tắc thừa (0 bước khớp): {:?}",
            stale
        ));
    }

    Ok(summary)
}

fn collect_relative_jsonl(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    if !dir.is_dir() {
        return Err(format!("thiếu thư mục {}", dir.display()));
    }
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        for entry in fs::read_dir(&cur).map_err(|e| format!("đọc {}: {e}", cur.display()))? {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                let rel = p
                    .strip_prefix(dir)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = fs::read_to_string(&p)
                    .map_err(|e| format!("đọc {}: {e}", p.display()))?
                    .replace("\r\n", "\n");
                out.insert(rel, text);
            }
        }
    }
    if out.is_empty() {
        return Err(format!("E94: {} rỗng", dir.display()));
    }
    Ok(out)
}

pub fn check_tsf_freshness(root: &Path) -> Result<usize, String> {
    check_replay_freshness(root, "tsf", &["corpus/shared", "corpus/win"], TSF_ALL_DIR)
}

/// SF-3 (SF-G1 qua session): `replay --adapter linux` chạy CHÍNH `textvn_session::compose::Composer`
/// (chính sách LINUX) phải tái tạo từng byte 58 vết Linux do `trace_compose.c` (mã C cũ) ghi.
pub fn check_linux_session_freshness(root: &Path) -> Result<usize, String> {
    check_replay_freshness(root, "linux", &["corpus/shared"], LINUX_ALL_DIR)
}

/// Sinh lại vết bằng `textvn-cli replay --adapter <adapter> --trace` rồi so byte với `committed_dir`.
fn check_replay_freshness(
    root: &Path,
    adapter: &str,
    corpus: &[&str],
    committed_dir: &str,
) -> Result<usize, String> {
    let tmp = std::env::temp_dir().join(format!("textvn-trace-{adapter}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| format!("tạo tmp trace dir: {e}"))?;

    let status = Command::new("cargo")
        .current_dir(root)
        .args(["run", "-q", "-p", "textvn-cli", "--", "replay"])
        .args(corpus)
        .args(["--adapter", adapter, "--trace"])
        .arg(&tmp)
        .status()
        .map_err(|e| format!("chạy textvn-cli replay --trace: {e}"))?;
    if !status.success() {
        let _ = fs::remove_dir_all(&tmp);
        return Err(format!("textvn-cli replay --trace thất bại: {status}"));
    }

    let committed = collect_relative_jsonl(&root.join(committed_dir));
    let fresh = collect_relative_jsonl(&tmp);
    let _ = fs::remove_dir_all(&tmp);

    let committed = committed?;
    let fresh = fresh?;
    if committed != fresh {
        let c_keys: BTreeSet<&String> = committed.keys().collect();
        let f_keys: BTreeSet<&String> = fresh.keys().collect();
        let mut diffs = Vec::new();
        for k in c_keys.symmetric_difference(&f_keys) {
            diffs.push(format!("thiếu/dư tệp: {k}"));
        }
        for k in c_keys.intersection(&f_keys) {
            if committed[*k] != fresh[*k] {
                diffs.push(format!("lệch nội dung: {k}"));
            }
        }
        return Err(format!(
            "SF-1c/1f/SF-3: {} tệp trong `{committed_dir}` lệch bản replay `--adapter {adapter}` hiện tại — TSF: chạy `cargo run -q -p textvn-cli -- replay {} --adapter {adapter} --trace {committed_dir}`; Linux: vết vàng do trace_compose.c ghi, lệch là session sai (không sinh lại):\n  {}",
            diffs.len(),
            corpus.join(" "),
            diffs.into_iter().take(15).collect::<Vec<_>>().join("\n  ")
        ));
    }
    Ok(committed.len())
}

pub fn run(args: &[String]) -> Result<(), String> {
    let mut skip_fresh = false;
    for a in args {
        match a.as_str() {
            "--skip-fresh" => skip_fresh = true,
            "--help" | "-h" => {
                println!(
                    "Usage: cargo xtask trace-diff [--skip-fresh]\n\
                     Đối chiếu 3 bộ vết corpus/traces/{{tsf,linux,mac}}/shared theo ALLOWED_DIFF.md\n\
                     và kiểm tra chống trôi 136 tệp corpus/traces/tsf/."
                );
                return Ok(());
            }
            other => return Err(format!("cờ lạ `{other}` (dùng `--skip-fresh` hoặc `-h`)")),
        }
    }

    let root = crate::assist_tools::repo_root();
    let summary = compare_roots(
        &root.join(TSF_DIR),
        &root.join(LINUX_DIR),
        &root.join(MAC_DIR),
        &root.join(ALLOWED_MD),
    )?;
    let rules_str = summary
        .rule_hits
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "trace-diff: {} ca shared · {} bước (TSF==Linux {}/{}, Linux==Mac {}/{}) · approved: [{}]",
        summary.cases,
        summary.steps_compared,
        summary.exact_tl_steps,
        summary.steps_compared,
        summary.exact_lm_steps,
        summary.steps_compared,
        rules_str
    );

    if !skip_fresh {
        let count = check_tsf_freshness(&root)?;
        println!("trace-diff: gate TSF freshness PASS ({count} tệp khớp byte-to-byte)");
        let count = check_linux_session_freshness(&root)?;
        println!(
            "trace-diff: gate Linux qua session (SF-3) PASS ({count} tệp khớp byte-to-byte với trace_compose.c)"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_case(dir: &Path, name: &str, body: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(format!("{name}.jsonl")), body).unwrap();
    }

    #[test]
    fn sf1f_repo_traces_and_allowed_diff_pass() {
        let root = crate::assist_tools::repo_root();
        let s = compare_roots(
            &root.join(TSF_DIR),
            &root.join(LINUX_DIR),
            &root.join(MAC_DIR),
            &root.join(ALLOWED_MD),
        )
        .unwrap();
        assert_eq!(s.cases, 58);
        assert!(s.steps_compared > 1000);
        assert!(s.rule_hits.values().all(|&c| c > 0));
    }

    #[test]
    fn sf1f_dot_bien_xoa_rule_trong_allowed_diff_phai_do() {
        let root = crate::assist_tools::repo_root();
        let tmp = std::env::temp_dir().join(format!("textvn-sf1f-drop-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let md = fs::read_to_string(root.join(ALLOWED_MD)).unwrap();
        let filtered: String = md
            .lines()
            .filter(|l| !l.contains("CUAS_PRINTABLE_BOUNDARY"))
            .collect::<Vec<_>>()
            .join("\n");
        let md_path = tmp.join("ALLOWED_DIFF.md");
        fs::write(&md_path, filtered).unwrap();
        let err = compare_roots(
            &root.join(TSF_DIR),
            &root.join(LINUX_DIR),
            &root.join(MAC_DIR),
            &md_path,
        )
        .unwrap_err();
        let _ = fs::remove_dir_all(&tmp);
        assert!(err.contains("chưa nằm trong ALLOWED_DIFF.md"), "{err}");
    }

    #[test]
    fn sf1f_dot_bien_sai_chu_commit_khong_duoc_rule_nuot() {
        let root = crate::assist_tools::repo_root();
        let tmp = std::env::temp_dir().join(format!("textvn-sf1f-corrupt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let t_dir = tmp.join("tsf");
        let l_dir = tmp.join("linux");
        let m_dir = tmp.join("mac");
        // Khác chữ commit (`chao.` vs `sai`) ⇒ CUAS_PRINTABLE_BOUNDARY KHÔNG được phép nuốt!
        write_case(
            &t_dir,
            "c1",
            "{\"case\":\"shared/c1\",\"step\":0,\"cmd\":\"commit\",\"text\":\"chao.\",\"n\":0}\n\
             {\"case\":\"shared/c1\",\"step\":0,\"cmd\":\"set_preedit\",\"text\":\"\",\"n\":0}\n\
             {\"case\":\"shared/c1\",\"step\":0,\"cmd\":\"eat_key\",\"text\":\"\",\"n\":0}\n",
        );
        write_case(
            &l_dir,
            "c1",
            "{\"case\":\"shared/c1\",\"step\":0,\"cmd\":\"commit\",\"text\":\"sai\",\"n\":0}\n\
             {\"case\":\"shared/c1\",\"step\":0,\"cmd\":\"set_preedit\",\"text\":\"\",\"n\":0}\n\
             {\"case\":\"shared/c1\",\"step\":0,\"cmd\":\"forward_key\",\"text\":\"\",\"n\":0}\n",
        );
        write_case(
            &m_dir,
            "c1",
            "{\"case\":\"shared/c1\",\"step\":0,\"cmd\":\"forward_key\",\"text\":\"\",\"n\":0}\n",
        );
        let err = compare_roots(&t_dir, &l_dir, &m_dir, &root.join(ALLOWED_MD)).unwrap_err();
        let _ = fs::remove_dir_all(&tmp);
        assert!(err.contains("[tsf vs linux]"), "{err}");
    }

    #[test]
    fn sf1f_fail_closed_khi_thieu_tep_hoac_lenh_la() {
        let root = crate::assist_tools::repo_root();
        let err = parse_trace_jsonl(
            "shared/x",
            "{\"case\":\"shared/x\",\"step\":0,\"cmd\":\"bogus\",\"text\":\"\",\"n\":0}\n",
            true,
        )
        .unwrap_err();
        assert!(err.contains("lệnh lạ"), "{err}");

        let tmp = std::env::temp_dir().join(format!("textvn-sf1f-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let err_dir = compare_roots(
            &tmp,
            &root.join(LINUX_DIR),
            &root.join(MAC_DIR),
            &root.join(ALLOWED_MD),
        )
        .unwrap_err();
        let _ = fs::remove_dir_all(&tmp);
        assert!(err_dir.contains("E94"), "{err_dir}");
    }

    /// Audit SF-1f: quy tắc cho phép phải kiểm GIÁ TRỊ — số xoá và ký tự ranh giới.
    #[test]
    fn sf1f_rule_kiem_gia_tri_so_xoa_va_ranh_gioi() {
        use TraceCmd::*;
        let lv = [SetPreedit("ba".into(), 2), EatKey];
        let ok = [DeleteBefore(1), SetPreedit("ba".into(), 2), EatKey];
        // Linux giữ preedit 1 ký tự, macOS đã cho đi thẳng nó ⇒ phải xoá đúng 1.
        assert!(check_rule_values("MAC_IMK_PREEDIT_LIFECYCLE", &[], None, &lv, &ok, 1, 0).is_ok());
        let over = [DeleteBefore(2), SetPreedit("ba".into(), 2), EatKey];
        let e = check_rule_values("MAC_IMK_PREEDIT_LIFECYCLE", &[], None, &lv, &over, 1, 0)
            .unwrap_err();
        assert!(e.contains("delete_before(2) != 1"), "{e}");
        // Ranh giới: macOS phải commit đúng chuỗi TSF commit cho cùng phím.
        let tv = [Commit("ba ".into()), SetPreedit(String::new(), 0), EatKey];
        let lb = [
            Commit("ba".into()),
            SetPreedit(String::new(), 0),
            ForwardKey,
        ];
        let mb_ok = [Commit("ba ".into()), SetPreedit(String::new(), 0), EatKey];
        let mb_bad = [Commit("baZ".into()), SetPreedit(String::new(), 0), EatKey];
        let r = "MAC_PRINTABLE_BOUNDARY_COMMIT";
        let tr = Some("CUAS_PRINTABLE_BOUNDARY");
        assert!(check_rule_values(r, &tv, tr, &lb, &mb_ok, 0, 0).is_ok());
        let e = check_rule_values(r, &tv, tr, &lb, &mb_bad, 0, 0).unwrap_err();
        assert!(e.contains("!= TSF commit"), "{e}");
        assert_eq!(preedit_len_after(3, &[EatKey]), 3);
        assert_eq!(
            preedit_len_after(3, &[Commit("x".into()), SetPreedit(String::new(), 0)]),
            0
        );
    }
}
