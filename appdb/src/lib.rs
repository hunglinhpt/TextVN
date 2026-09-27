// SPDX-License-Identifier: GPL-3.0-or-later
//! App preset database — parser và lookup thuần, không I/O/network.
//!
//! Adapter phải verify chữ ký trước khi đưa JSON không tin cậy vào `AppDb`.
//! Resolver chỉ nhận database đã được caller tin cậy và luôn giữ gate secure/
//! disabled/capability trong `vietime-strategy`.

use serde::Deserialize;
use vietime_strategy::{ResolveInput, Strategy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppDbError {
    Schema,
    Version,
}

/// Chủ sở hữu engine mà adapter chọn trước khi dispatch một key. Giá trị này
/// không thay thế security/enabled gate; nó chỉ ngăn TSF và hook cùng xử lý.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineOwner {
    Tsf,
    Hook,
    Imk,
    Tap,
    Ibus,
    Fcitx5,
    X11,
}

/// Preset đầu tiên khớp app + role. Các field `None` nghĩa là để adapter dùng
/// default, không phải ghi đè bằng một giá trị ngầm định.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preset {
    pub strategy: Option<Strategy>,
    pub engine_owner: Option<EngineOwner>,
    pub enabled_default: Option<bool>,
}

impl std::fmt::Display for AppDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppDbError::Schema => write!(f, "appdb: invalid schema"),
            AppDbError::Version => write!(f, "appdb: unsupported appdb_version"),
        }
    }
}

/// Database preset đã parse. Thứ tự `entries` được giữ nguyên: match đầu tiên thắng.
#[derive(Debug, Clone)]
pub struct AppDb {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
struct Entry {
    matcher: Matcher,
    field_roles: Option<Vec<u32>>,
    preset: Preset,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Matcher {
    #[serde(default)]
    any: Option<Vec<MatchClause>>,
    #[serde(default)]
    exe: Option<String>,
    #[serde(default)]
    bundle: Option<String>,
    #[serde(rename = "class", default)]
    class_name: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchClause {
    #[serde(default)]
    exe: Option<String>,
    #[serde(default)]
    bundle: Option<String>,
    #[serde(rename = "class", default)]
    class_name: Option<String>,
    #[serde(rename = "and_subprocess", default)]
    and_subprocess: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDb {
    appdb_version: u32,
    #[serde(default)]
    min_engine_version: Option<String>,
    #[serde(rename = "updated_at", default)]
    _updated_at: Option<String>,
    #[serde(default)]
    entries: Vec<WireEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEntry {
    #[serde(rename = "id", default)]
    _id: Option<String>,
    #[serde(rename = "match")]
    matcher: Matcher,
    #[serde(default)]
    when: WireWhen,
    #[serde(default)]
    strategy: Option<String>,
    #[serde(default)]
    min_engine_version: Option<String>,
    #[serde(default)]
    engine_owner: Option<String>,
    #[serde(default)]
    enabled_default: Option<bool>,
    #[serde(rename = "inject_mode", default)]
    _inject_mode: Option<String>,
    #[serde(rename = "min_os", default)]
    _min_os: Option<String>,
    #[serde(rename = "notes", default)]
    _notes: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireWhen {
    #[serde(default)]
    field_role: FieldRoles,
}

#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
enum FieldRoles {
    #[default]
    Missing,
    One(String),
    Many(Vec<String>),
}

impl AppDb {
    pub fn parse(json: &str) -> Result<Self, AppDbError> {
        let wire: WireDb = serde_json::from_str(json).map_err(|_| AppDbError::Schema)?;
        if wire.appdb_version != 1 {
            return Err(AppDbError::Version);
        }
        if !engine_version_is_supported(wire.min_engine_version.as_deref())? {
            // Database thuộc release mới hơn: giữ schema hợp lệ nhưng không áp
            // preset nào. Đây là fail-safe, adapter vẫn dùng field default.
            return Ok(Self {
                entries: Vec::new(),
            });
        }
        let mut entries = Vec::with_capacity(wire.entries.len());
        for entry in wire.entries {
            entry.matcher.validate()?;
            if !engine_version_is_supported(entry.min_engine_version.as_deref())? {
                continue;
            }
            let field_roles = match entry.when.field_role {
                FieldRoles::Missing => None,
                FieldRoles::One(role) => Some(vec![parse_role(&role)?]),
                FieldRoles::Many(roles) => {
                    if roles.is_empty() {
                        return Err(AppDbError::Schema);
                    }
                    Some(
                        roles
                            .iter()
                            .map(|role| parse_role(role))
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                }
            };
            let strategy = entry.strategy.as_deref().map(parse_strategy).transpose()?;
            entries.push(Entry {
                matcher: entry.matcher,
                field_roles,
                preset: Preset {
                    strategy,
                    engine_owner: entry
                        .engine_owner
                        .as_deref()
                        .map(parse_engine_owner)
                        .transpose()?,
                    enabled_default: entry.enabled_default,
                },
            });
        }
        Ok(Self { entries })
    }

    /// Số entry hữu hiệu sau khi lọc `min_engine_version`; tiện cho doctor và
    /// test gói preset, không lộ representation nội bộ.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Strategy preset hệ thống đầu tiên khớp `app_id` + role.
    pub fn strategy_for(&self, app_id: &str, field_role: u32) -> Option<Strategy> {
        self.preset_for(app_id, field_role)
            .and_then(|preset| preset.strategy)
    }

    /// Lookup không merge field từ entry sau: entry user hẹp có thể ghi đè
    /// owner/enabled/strategy cùng lúc, và entry system khác role vẫn còn hiệu lực.
    pub fn preset_for(&self, app_id: &str, field_role: u32) -> Option<Preset> {
        self.entries
            .iter()
            .find(|entry| entry.matches(app_id, field_role))
            .map(|entry| entry.preset)
    }

    /// User database có precedence tuyệt đối theo thứ tự entry. Không merge
    /// từng field rồi suy đoán: nối user trước system giữ đúng rule "first
    /// matching entry wins" và cho phép user đặt override hẹp theo field role.
    pub fn with_user_overrides(system: &Self, user: &Self) -> Self {
        let mut entries = Vec::with_capacity(user.entries.len() + system.entries.len());
        entries.extend(user.entries.iter().cloned());
        entries.extend(system.entries.iter().cloned());
        Self { entries }
    }

    /// Resolve với toàn bộ gate của strategy crate; appdb chỉ cấp bước preset hệ thống.
    pub fn resolve(&self, mut input: ResolveInput, app_id: &str) -> Strategy {
        input.system_preset = self.strategy_for(app_id, input.field_role);
        vietime_strategy::resolve(input)
    }
}

impl Entry {
    fn matches(&self, app_id: &str, field_role: u32) -> bool {
        let role_matches = self
            .field_roles
            .as_ref()
            .is_none_or(|roles| roles.contains(&field_role));
        role_matches && self.matcher.matches(app_id)
    }
}

impl Matcher {
    /// Ánh xạ đúng `oneOf` của schema: hoặc `any` không rỗng, hoặc ít nhất
    /// một matcher trực tiếp. Không để `{}` / `{"any":[]}` thành wildcard.
    fn validate(&self) -> Result<(), AppDbError> {
        let has_direct = self.exe.is_some() || self.bundle.is_some() || self.class_name.is_some();
        match &self.any {
            Some(clauses) if !clauses.is_empty() && !has_direct => {
                if clauses.iter().all(MatchClause::is_valid) {
                    Ok(())
                } else {
                    Err(AppDbError::Schema)
                }
            }
            None if has_direct => Ok(()),
            _ => Err(AppDbError::Schema),
        }
    }

    fn matches(&self, app_id: &str) -> bool {
        if let Some(clauses) = &self.any {
            return clauses.iter().any(|clause| clause.matches(app_id));
        }
        MatchClause {
            exe: self.exe.clone(),
            bundle: self.bundle.clone(),
            class_name: self.class_name.clone(),
            and_subprocess: None,
        }
        .matches(app_id)
    }
}

impl MatchClause {
    fn is_valid(&self) -> bool {
        // Quan hệ parent/subprocess cần PID/process-tree mà core parser không
        // có. Adapter OS phải lọc app_id trước khi gọi resolver; đọc field ở
        // đây để giữ wire schema v1 tương thích mà không giả lập quan hệ đó.
        let _and_subprocess_is_adapter_constraint = self.and_subprocess;
        self.exe.is_some() || self.bundle.is_some() || self.class_name.is_some()
    }

    fn matches(&self, app_id: &str) -> bool {
        [
            self.exe.as_deref(),
            self.bundle.as_deref(),
            self.class_name.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|candidate| candidate.eq_ignore_ascii_case(app_id))
    }
}

fn parse_strategy(value: &str) -> Result<Strategy, AppDbError> {
    match value {
        "Preedit" => Ok(Strategy::Preedit),
        "BackspaceType" => Ok(Strategy::BackspaceType),
        "SelectionReplace" => Ok(Strategy::SelectionReplace),
        "ForwardAsCommit" => Ok(Strategy::ForwardAsCommit),
        "Passthrough" => Ok(Strategy::Passthrough),
        _ => Err(AppDbError::Schema),
    }
}

fn parse_engine_owner(value: &str) -> Result<EngineOwner, AppDbError> {
    match value {
        "tsf" => Ok(EngineOwner::Tsf),
        "hook" => Ok(EngineOwner::Hook),
        "imk" => Ok(EngineOwner::Imk),
        "tap" => Ok(EngineOwner::Tap),
        "ibus" => Ok(EngineOwner::Ibus),
        "fcitx5" => Ok(EngineOwner::Fcitx5),
        "x11" => Ok(EngineOwner::X11),
        _ => Err(AppDbError::Schema),
    }
}

fn parse_role(value: &str) -> Result<u32, AppDbError> {
    vietime_strategy::parse_field_role(value).ok_or(AppDbError::Schema)
}

/// AppDB phải không đổi hành vi của engine cũ. Dùng version của workspace để
/// tránh hard-code hai nguồn version khác nhau; chỉ nhận `major.minor.patch`
/// vì release artifact đã định dạng version trước khi ký.
fn engine_version_is_supported(required: Option<&str>) -> Result<bool, AppDbError> {
    let Some(required) = required else {
        return Ok(true);
    };
    let required = parse_version(required)?;
    let current = parse_version(env!("CARGO_PKG_VERSION"))?;
    Ok(required <= current)
}

fn parse_version(value: &str) -> Result<(u32, u32, u32), AppDbError> {
    let mut parts = value.split('.');
    let mut parse_part = || {
        parts
            .next()
            .ok_or(AppDbError::Schema)?
            .parse::<u32>()
            .map_err(|_| AppDbError::Schema)
    };
    let major = parse_part()?;
    let minor = parse_part()?;
    let patch = parse_part()?;
    if parts.next().is_some() {
        return Err(AppDbError::Schema);
    }
    Ok((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vietime_strategy::{
        IME_CAP_FIELD_DETECT, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR,
        IME_FIELD_BODY,
    };

    const DB: &str = r#"{
      "appdb_version": 1,
      "entries": [
        {"match":{"any":[{"exe":"chrome.exe"},{"bundle":"com.google.Chrome"}]},
         "when":{"field_role":["address_bar","search"]}, "strategy":"SelectionReplace"},
        {"match":{"exe":"chrome.exe"}, "when":{"field_role":"body"}, "strategy":"Preedit"}
      ]
    }"#;

    fn input(role: u32) -> ResolveInput {
        ResolveInput {
            secure: false,
            enabled: true,
            user_preset: None,
            system_preset: None,
            hint: -1,
            field_role: role,
            caps: IME_CAP_PREEDIT | IME_CAP_SELECTION | IME_CAP_FIELD_DETECT,
        }
    }

    #[test]
    fn first_matching_entry_and_role_win() {
        let db = AppDb::parse(DB).unwrap();
        assert_eq!(
            db.strategy_for("chrome.exe", IME_FIELD_ADDRESS_BAR),
            Some(Strategy::SelectionReplace)
        );
        assert_eq!(
            db.strategy_for("chrome.exe", IME_FIELD_BODY),
            Some(Strategy::Preedit)
        );
        assert_eq!(db.strategy_for("firefox.exe", IME_FIELD_BODY), None);
    }

    #[test]
    fn resolver_keeps_secure_and_capability_gates() {
        let db = AppDb::parse(DB).unwrap();
        let mut secure = input(IME_FIELD_ADDRESS_BAR);
        secure.secure = true;
        assert_eq!(db.resolve(secure, "chrome.exe"), Strategy::Passthrough);

        let mut no_selection = input(IME_FIELD_ADDRESS_BAR);
        no_selection.caps = IME_CAP_FIELD_DETECT;
        assert_eq!(
            db.resolve(no_selection, "chrome.exe"),
            Strategy::BackspaceType
        );
    }

    #[test]
    fn malformed_or_unknown_values_are_rejected() {
        assert!(matches!(AppDb::parse("{}"), Err(AppDbError::Schema)));
        assert!(matches!(
            AppDb::parse(r#"{"appdb_version":1,"entries":[{"match":{},"strategy":"Popup"}]}"#),
            Err(AppDbError::Schema)
        ));
        for invalid_match in [r#"{}"#, r#"{"any":[]}"#, r#"{"any":[{}]}"#] {
            let db = format!(
                r#"{{"appdb_version":1,"entries":[{{"match":{invalid_match},"strategy":"Preedit"}}]}}"#
            );
            assert!(matches!(AppDb::parse(&db), Err(AppDbError::Schema)));
        }
        // Ký được nội dung không có nghĩa là parser được im lặng bỏ field lạ:
        // schema v1 phải là contract chặt cho mọi adapter.
        assert!(matches!(
            AppDb::parse(r#"{"appdb_version":1,"surprise":true,"entries":[]}"#),
            Err(AppDbError::Schema)
        ));
    }

    #[test]
    fn newer_engine_requirement_skips_only_affected_presets() {
        let db = AppDb::parse(
            r#"{
              "appdb_version": 1,
              "entries": [
                {"match":{"exe":"chrome.exe"}, "strategy":"Passthrough",
                 "min_engine_version":"9.0.0"},
                {"match":{"exe":"chrome.exe"}, "strategy":"Preedit"}
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(
            db.strategy_for("chrome.exe", IME_FIELD_BODY),
            Some(Strategy::Preedit)
        );
        assert!(matches!(
            AppDb::parse(r#"{"appdb_version":1,"min_engine_version":"1.x.0"}"#),
            Err(AppDbError::Schema)
        ));
    }

    #[test]
    fn documented_metadata_and_match_clause_parse() {
        let db = AppDb::parse(
            r#"{
              "appdb_version": 1,
              "min_engine_version": "0.1.0",
              "updated_at": "2026-09-27T00:00:00Z",
              "entries": [{
                "id": "terminal",
                "match": {"any":[{"exe":"wezterm-gui.exe","and_subprocess":false}]},
                "strategy": "ForwardAsCommit",
                "engine_owner": "tsf",
                "inject_mode": "unicode",
                "enabled_default": true,
                "min_os": null,
                "notes": "B8"
              }]
            }"#,
        )
        .unwrap();
        assert_eq!(
            db.strategy_for("wezterm-gui.exe", IME_FIELD_BODY),
            Some(Strategy::ForwardAsCommit)
        );
        assert_eq!(
            db.preset_for("wezterm-gui.exe", IME_FIELD_BODY),
            Some(Preset {
                strategy: Some(Strategy::ForwardAsCommit),
                engine_owner: Some(EngineOwner::Tsf),
                enabled_default: Some(true),
            })
        );
    }

    #[test]
    fn user_override_precedes_system_without_hiding_unrelated_roles() {
        let system = AppDb::parse(
            r#"{"appdb_version":1,"entries":[
              {"match":{"exe":"chrome.exe"},"when":{"field_role":"address_bar"},"strategy":"SelectionReplace"},
              {"match":{"exe":"chrome.exe"},"when":{"field_role":"body"},"strategy":"Preedit"}
            ]}"#,
        )
        .unwrap();
        let user = AppDb::parse(
            r#"{"appdb_version":1,"entries":[
              {"match":{"exe":"chrome.exe"},"when":{"field_role":"body"},"strategy":"ForwardAsCommit"}
            ]}"#,
        )
        .unwrap();
        let merged = AppDb::with_user_overrides(&system, &user);
        assert_eq!(
            merged.strategy_for("chrome.exe", IME_FIELD_BODY),
            Some(Strategy::ForwardAsCommit)
        );
        assert_eq!(
            merged.strategy_for("chrome.exe", IME_FIELD_ADDRESS_BAR),
            Some(Strategy::SelectionReplace)
        );
    }

    #[test]
    fn user_owner_and_enabled_override_are_atomic_per_matching_entry() {
        let system = AppDb::parse(
            r#"{"appdb_version":1,"entries":[
              {"match":{"exe":"pwsh.exe"},"when":{"field_role":"terminal"},"strategy":"ForwardAsCommit","engine_owner":"hook","enabled_default":true},
              {"match":{"exe":"pwsh.exe"},"when":{"field_role":"body"},"strategy":"Preedit","engine_owner":"tsf","enabled_default":true}
            ]}"#,
        )
        .unwrap();
        let user = AppDb::parse(
            r#"{"appdb_version":1,"entries":[
              {"match":{"exe":"pwsh.exe"},"when":{"field_role":"terminal"},"strategy":"Passthrough","engine_owner":"hook","enabled_default":false}
            ]}"#,
        )
        .unwrap();
        let merged = AppDb::with_user_overrides(&system, &user);
        assert_eq!(
            merged.preset_for("pwsh.exe", vietime_strategy::IME_FIELD_TERMINAL),
            Some(Preset {
                strategy: Some(Strategy::Passthrough),
                engine_owner: Some(EngineOwner::Hook),
                enabled_default: Some(false),
            })
        );
        assert_eq!(
            merged
                .preset_for("pwsh.exe", IME_FIELD_BODY)
                .unwrap()
                .engine_owner,
            Some(EngineOwner::Tsf)
        );
    }

    #[test]
    fn shipped_windows_preset_has_twenty_ordered_entries() {
        let db = AppDb::parse(include_str!("../../data/appdb.default.json")).unwrap();
        assert_eq!(db.entry_count(), 20);
        assert_eq!(
            db.preset_for("chrome.exe", IME_FIELD_ADDRESS_BAR),
            Some(Preset {
                strategy: Some(Strategy::SelectionReplace),
                engine_owner: Some(EngineOwner::Tsf),
                enabled_default: Some(true),
            })
        );
        assert_eq!(
            db.preset_for("pwsh.exe", vietime_strategy::IME_FIELD_TERMINAL)
                .unwrap()
                .engine_owner,
            Some(EngineOwner::Hook)
        );
        assert_eq!(
            db.preset_for("code.exe", IME_FIELD_BODY)
                .unwrap()
                .enabled_default,
            Some(false)
        );
    }
}
