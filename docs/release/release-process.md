# Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor

> **Trạng thái:** v1 (2026-10-01) — viết lại sau khi v0.2.0 → v0.2.2 phát hành
> thành công bằng đúng quy trình này (ban đầu chạy theo kinh nghiệm, nay chuẩn
> hoá thành checklist). Quy tắc **G16** trong `docs/00-WORKFLOW.md` trỏ tới file
> này: **không ai được phép tag version mới nếu chưa đi hết Checklist A (máy
> local) và Checklist B (repo/CI) bên dưới.**
>
> Bản phát hành ở repo này là **release candidate chưa ký số**. Production cần
> thêm Authenticode (Windows), Developer ID + notarization (macOS) và smoke GUI
> trên máy thật — luôn ghi rõ điều đó trong CHANGELOG và release notes, không
> bao giờ tự nâng một candidate thành production.

## 0. Nguyên tắc (đã xác minh qua 3 lần phát hành)

| # | Nguyên tắc | Vì sao |
|---|---|---|
| 1 | **Tag là nguồn sự thật.** Version nằm ở `[workspace.package]` Cargo.toml + 14 chỗ ghi tay; `git tag vN.N.N` kích hoạt workflow `release-candidate` build 3 nền tảng và publish. Không publish binary build tay từ máy local lên release. | `source_tree_clean=true` + `git_commit` = tag được validate máy trước khi publish; binary phải tái lập được từ commit |
| 2 | **Publish là draft-first** (release.yml): validate payload → tạo draft → upload hết asset → mới `--draft=false`. Upload fail giữa chừng = rerun được, không bao giờ public release thiếu asset. | v0.2.2 audit M2: `gh release create` upload nửa chừng từng khoá luôn khả năng chạy lại |
| 3 | **Không bỏ gate local dù CI sẽ chạy lại.** CI đỏ tốn 20–90 phút; gate local tốn 3–5 phút và bắt lỗi trước khi push. | `tránh tối đa build fail` — bắt ở vòng rẻ nhất |
| 4 | **Mọi phiên bản phát hành đều phải có đủ bộ docs** (§ Checklist B.4) — thiếu một mục trong số đó = chưa Done. | release note/changelog/version/readme thiếu là lỗi lặp lại nhiều nhất của agent |
| 5 | **CI đỏ thì xử lý trong phiên** (G13) — nhưng phân biệt nhiễu runner đã ghi sổ (B5) với hồi quy thật: cùng commit rerun xanh + release run cùng bộ test xanh = nhiễu; đỏ ≥2 lần liên tiếp trên cùng commit = điều tra thật. | job typing smoke "Windows package" đỏ chớp trên runner dùng chung đã được ghi nhận là nhiễu có bằng chứng |

## Checklist A — máy local (TRƯỚC khi commit/tag)

### A0. `cargo xtask preflight` — BẮT BUỘC exit 0 (thay cho chạy tay từng lệnh)

Một lệnh chạy đủ 17 gate theo đúng thứ tự CI, fail-fast, exit code thật
(không bao giờ pipe `| tail` che exit code — sự cố 2 lần 2026-10-02: clippy
`unused-mut` đẩy code rồi mới phát hiện). Preflight xanh = CI thấy đúng cây
đang test ⇒ khả năng đỏ chỉ còn nhiễu runner (perf, typing smoke — rerun).
Chi tiết từng bước + điều nó chặn: `xtask/src/preflight.rs` (mảng STEPS).

| Bước preflight | Validate cái gì | Sự cố thật đã chặn/lẽ ra chặn |
|---|---|---|
| fmt | rustfmt job | — |
| clippy `-D warnings` | mọi lint trong workspace, gồm target test | `unused mut` × 2 (be2489f); thiếu import ở target non-Windows (E11) |
| check-linux | cross-compile không phá Linux/macOS | E11: nhánh non-Windows sai chữ ký sau khi Windows xanh |
| test --workspace | 349+ unit/integration | break behavior corpus (dedupe layout, EN-detect) |
| verify + sizes | ABI header C + struct 20/532 | lệch FFI khi đổi struct |
| replay × 3 adapter | hành vi gõ thật mọi nền tảng | regression 5269ae1/fecf245: raw typing sau đổi layout |
| check-tables/win-corpus/mac-corpus | dữ liệu sinh khớp generator | corpus quên regenerate |
| version-sync | 14 chỗ version | bump sót chỗ |
| perf | hồi quy so baseline | baseline chết theo runner → phải re-record từ CI (7f069f0) |
| hygiene-apis / hygiene-docs | API giống-malware, link hỏng | — |
| homebrew | ruby cask hợp lệ + sha256 format | formula sai hash 0.2.5–0.2.7 |

Chạy từ gốc repo, tất cả phải xanh. Lệnh đúng như dưới đây (đã là lệnh của
build-release.ps1 và CI — cùng một gate, không thêm không bớt):

```powershell
# A1. Format + lint (0 warning). Lưu ý: ci-shared chạy clippy --workspace
# (KHÔNG exclude win-hook) — CI nghiêm ngặt hơn dòng dưới đây; nếu sửa gì
# trong adapters/win-hook thì chạy cả `cargo clippy --workspace --all-targets`.
cargo fmt --all -- --check
cargo clippy --workspace --exclude textvn-win-hook --all-targets -- -D warnings

# A2. Test toàn workspace (kỳ vọng ≥343 pass — số test tăng thì ghi con số MỚI vào report)
cargo test --workspace

# A3. Version đồng bộ 14 chỗ = version đang phát hành
cargo run -q -p xtask -- check-version-sync

# A4. ABI + corpus (số pass ghi vào build-release-report)
cargo run -q -p textvn-cli -- verify
cargo run -q -p xtask -- check-tables
cargo run -q -p textvn-cli -- replay corpus/shared corpus/mac --adapter mac
cargo run -q -p textvn-cli -- replay corpus/shared corpus/win --adapter win
cargo run -q -p textvn-cli -- replay corpus/shared corpus/win --adapter tsf

# A5. (nếu đụng packaging/CI) syntax + schema
bash -n scripts/*.sh packaging/linux/*.sh        # shell
python -c "import yaml,glob; [yaml.safe_load(open(f,encoding='utf-8')) for f in glob.glob('.github/workflows/*.yml')]"

# A6. (nếu đụng docs) link + inject check như repo-hygiene
python .github/scripts/check_doc_links.py
python .github/scripts/check_no_injection_apis.py

# A7. Cross-check Linux trên host Windows (E9): bắt lỗi cfg(windows) mà clippy
# Windows không nhìn thấy — BẮT BUỘC khi đụng cfg-gate/import liên platform.
cargo check --workspace --exclude textvn-win-hook --all-targets   --target x86_64-unknown-linux-gnu
```

**Kịch bản build release trên Windows (tuỳ chọn nhưng khuyến nghị cho bản có
thay đổi packaging):** `powershell -NoProfile -ExecutionPolicy Bypass -File
.\build-release.ps1 -BuildInstaller` — nó chạy lại A1–A4 + runtime smoke + đóng
gói ZIP/installer. Exit 2 với `-Channel production` là chủ đích (production
blockers chưa được chứng minh).

**Sạch trước khi commit (G14):** không process test còn chạy, không file
`dbg_*`/`*.tmp`, `git status` chỉ chứa đúng file mình sửa (`git diff
--cached --name-only` đối chiếu trước khi commit), probe registry/spike registry
đã xoá sạch.

## Checklist B — repo/CI (SAU khi local xanh)

| Bước | Việc | Bằng chứng |
|---|---|---|
| B1 | Commit theo `loai(scope): nd` (G5) — **tách commit code và commit docs-bằng-phát** để tag chỉ vào commit code đã xanh. `git push` ngay. | link commit |
| B2 | `graphify update .` → commit `graphify-out/` → push (G5 bước 7). | link commit graphify |
| B3 | Chờ CI xanh trên commit **cuối cùng** trước khi tag: `ci-shared` (perf `continue-on-error` là trừ, đỏ ≠ chặn), `repo-hygiene`, `ci-macos` (nếu đụng macOS). Red trên typing smoke → áp B5-checklist (rerun) rồi mới kết luận. | link run xanh |
| B4 | **Bộ docs bắt buộc của bản phát hành** — kiểm từng mục: ① `CHANGELOG.md` có entry `[N.N.N]` + link compare `[N.N.N]: .../compare/v(N-1)...vN` ở cuối file (thiếu link = render chết); ② `README.md` "Bản mới nhất" trỏ release mới; ③ `packaging/linux/appstream/*.metainfo.xml` có `<release version="N.N.N">` đứng đầu; ④ `docs/release/build-release-report.md` có mục bản mới với bảng gate A1–A4; ⑤ lỗi mới gặp trong phiên → `docs/specs/win-test-common-errors.md` hoặc `specs/project-common-errors.md` (G3). | từng mục |
| B5 | Tag: `git tag -a vN.N.N -m "..." && git push origin vN.N.N`. **Tag phải trỏ vào commit đã có CI xanh (B3).** | link tag |
| B6 | Theo dõi workflow `release-candidate` (4 job: windows/linux/macos/publish). Publish job tự validate: số asset, `RELEASE_REPORT.json` (`version` = tag, `source_tree_clean=true`, `feature_profile=tsf-only`), không có legacy hook. | link run |
| B7 | **Verify release sau khi publish:** `gh release view vN.N.N --json assets` đủ 7 asset (portable ZIP, setup exe, linux tar.gz, mac pkg, macos universal zip + tar.gz, SHA256SUMS.txt); tải `SHA256SUMS.txt` ghim hash vào build-release-report; mở trang release kiểm notes. | hash pin trong report |
| B7b | **Cập nhật `packaging/homebrew/textvn.rb`: `sha256` = hash của `TextVN-macos-universal-v<ver>.zip`** (lấy từ `SHA256SUMS.txt`, cùng lần tải ở B7). Bước này từng bị bỏ sót từ 0.2.5→0.2.7 (formula giữ hash cũ — cask cài sẽ lỗi checksum). | diff formula + hash khớp SHA256SUMS |
| B8 | Bổ sung số liệu CI + link run vào `build-release-report.md` (mục bản mới), commit docs + graphify, push. Kiểm `ci-shared`/`repo-hygiene` xanh trên commit cuối. | link run + commit |
| B9 | Dọn dẹp (G14): không process còn lại, `git status` sạch, memory dự án cập nhật nếu có bài học mới. | — |

## Bộ docs của một bản phát hành — chú thích từng file

| File | Nội dung bắt buộc |
|---|---|
| `CHANGELOG.md` | Entry `## [N.N.N] — date` theo Keep a Changelog (Fixed/Added/Changed/Housekeeping), nêu rõ "release candidate, chưa ký số". **Không xóa entry cũ.** |
| Link compare cuối `CHANGELOG.md` | `[N.N.N]: https://github.com/hunglinhpt/TextVN/compare/v(N-1)...vN` — quên là link render chết (đã xảy ra). |
| `README.md` | "Bản mới nhất: [GitHub Release vN.N.N]" + 1 dòng tóm tắt đắt giá nhất của bản + trạng thái chưa ký. |
| `packaging/linux/appstream/*.metainfo.xml` | `<release version="N.N.N" date="...">` đứng đầu `<releases>` (check-version-sync gate). |
| `docs/release/build-release-report.md` | Mục "Bản N.N.N": mục tiêu, bảng gate local (A1–A4 + số test + replay), bảng perf nếu đụng hot path, mục "Phát hành" (link release run, checksums pin, trạng thái CI), Trạng thái (còn thiếu gì để production). |
| Common errors (`docs/specs/*`) | Lỗi mới gặp khi phát hành (build fail, CI flake mới, lỗi docs) → entry mới, không xoá entry cũ (G3). |

## Sự cố đã gặp khi phát hành — tra trước khi xử lý

| Mã | Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|---|
| R1 | clippy đỏ trên CI nhưng local "đã chạy" | chạy `cargo clippy … \| tail` trong chuỗi `&&` — pipe che exit code, chuỗi vẫn tiếp tục | KHÔNG pipe các lệnh gate; dùng `cargo xtask preflight` (Command::status, exit thật) hoặc `set -o pipefail` |
| R2 | job perf đỏ trên mọi commit | baseline-win.json đo bằng runner image cũ — runner mới chậm hơn trên micro-bench | re-record baseline TỪ SỐ ĐO CI (không dùng số máy local — hardware khác); bisect worktree commit cũ để chứng minh không phải code mới |
| R3 | `cargo xtask` báo "no such command" | chưa có alias | alias đã thêm ở `.cargo/config.toml`; nếu clone mới mà thiếu, dùng `cargo run -q -p xtask -- <cmd>` |

| Triệu chứng | Nguyên nhân | Xử lý |
|---|---|---|
| `ci-shared` job "Windows package" đỏ: notepad nhận `[]` hoặc wordpad thiếu/dư ký tự | Nhiễu runner GHA dùng chung (B5, `win-test-common-errors.md`) | `gh run rerun <id> --failed`; đỏ ≥2 lần liên tiếp trên cùng commit mới điều tra thật |
| Job `perf regression` đỏ | `continue-on-error: true` theo thiết kế — nhiễu runner 2 vCPU; gate cứng chạy self-hosted (RW5) | Không chặn release; đo local nếu muốn số sạch |
| Run CI của commit code bị `cancelled` | Push docs/graphify sau commit code (concurrency) | Kể từ 2026-10-01: ci-shared không còn cancel-in-progress trên main + bỏ qua push chỉ-docs; nếu vẫn dính, push commit rỗng `git commit --allow-empty -m "chore: retrigger CI"` là cách cuối |
| Publish fail lúc upload asset | Mạng/quota runner | Rerun workflow — release ở trạng thái draft được phép upload tiếp (`--clobber`); chỉ release đã public mới bị chặn ghi đè |
| `check-version-sync` fail | Bump thiếu chỗ (14 chỗ ghi tay) | Sửa đúng file báo lỗi; nguyên tắc: Cargo.toml trước, xtask báo tên từng file |
| `source_tree_clean=false` bị publish job chặn | Đã tag khi tree dirty | Tag lại: xóa tag (`git push origin :vN.N.N` + `git tag -d`), commit sạch, tag lại |
| Release run fail ở job macos: `library 'textvn_ffi' not found` | `swift test` chạy trước `scripts/build-macos.sh` — test của macos-app link `libtextvn_ffi` từ `adapters/macos-imk/lib` do build-macos.sh dựng | Thứ tự bắt buộc job macos của release.yml: **build-macos.sh → swift test → package-macos-pkg.sh** (test vẫn trước package) |

## Bằng chứng quy trình chạy thành công

| Bản | Tag → run | Kết quả |
|---|---|---|
| v0.2.0 | `4f01cc1` → release run 2026-09-30 | publish đủ asset Windows, pre-release |
| v0.2.1 | `8216070` → [release-candidate #36719431897](https://github.com/hunglinhpt/TextVN/actions/runs/36719431897) | 7 asset, CI 3 nền tảng xanh, checksums khớp |
| v0.2.2 | `96083d5` → [release-candidate #36745978568](https://github.com/hunglinhpt/TextVN/actions/runs/36745978568) | 4/4 job xanh, 7 asset, validate `RELEASE_REPORT.json` pass, checksums pin trong build-release-report |

v0.2.3 trở đi phát hành theo đúng checklist này (G16).
