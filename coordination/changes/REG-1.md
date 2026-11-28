# REG-1 — Sổ hồi quy + gate (claude-opus-c4c5a567)

## VI

- **Sổ hồi quy `docs/specs/regression-index.md`** + gate `cargo xtask regression-index` (bước preflight 26/26 và
  `repo-hygiene` 5b): mỗi mã lỗi đã ghi — E* (`project-common-errors.md`), `win/*` (`win-test-common-errors.md`),
  R* (`release-process.md`), mọi R2-* trong `CHANGELOG.md` — phải có ≥ 1 bằng chứng chống tái diễn:
  `test:<tệp>::<hàm>`, `file:<corpus>`, `gate:<tệp xtask/script/workflow còn nhắc mã>`, `lesson:` (chỉ `win/*`),
  `process:<mô tả>`, hoặc nợ `todo:REG-2` (số dòng todo ≤ `todo-max`, chỉ được giảm).
- **Đỏ khi**: xoá/đổi tên test đang chặn một mã; thêm mã lỗi mới mà không có dòng; tăng số `todo:REG-2`; gate bị gỡ
  khỏi tệp (tệp không còn nhắc mã). Đã đột biến cả 3 trên cây thật — đều đỏ.
- **Khởi tạo**: `--bootstrap` tự tìm test nhắc mã (tên `r2_174_*`, chú thích/thân hàm nhắc `E82`), corpus, gate —
  229 mã: 135 tham chiếu test, 46 gate, 9 corpus, 56 lesson, 8 process, **81 `todo:REG-2`** (việc của REG-2). Mã
  một chữ số (E1–E9, R1–R8) không tự tìm (quá chung: `E1 BB 99` là byte UTF-8).
- Mã mới: `cargo xtask regression-index --bootstrap --merge` rồi sửa tay cho đúng.

## EN

- **Regression index `docs/specs/regression-index.md`** + gate `cargo xtask regression-index` (preflight step 26/26 and
  `repo-hygiene` 5b): every recorded error code — E* (`project-common-errors.md`), `win/*` (`win-test-common-errors.md`),
  R* (`release-process.md`), every R2-* in `CHANGELOG.md` — must point to ≥ 1 anti-regression evidence:
  `test:<file>::<fn>`, `file:<corpus>`, `gate:<xtask/script/workflow file still naming the code>`, `lesson:` (only `win/*`),
  `process:<description>`, or the debt marker `todo:REG-2` (todo rows ≤ `todo-max`, which may only go down).
- **Red when**: a test guarding a code is deleted/renamed; a new error code has no row; the `todo:REG-2` count grows; a
  gate is removed from its file (file no longer names the code). All 3 mutations tried on the real tree — all red.
- **Bootstrap**: `--bootstrap` finds tests naming the code (`r2_174_*` names, `E82` in comments/body), corpus files and
  gates — 229 codes: 135 test refs, 46 gates, 9 corpus, 56 lessons, 8 process, **81 `todo:REG-2`** (REG-2's work).
  Single-digit codes (E1–E9, R1–R8) are not auto-discovered (too generic: `E1 BB 99` is a UTF-8 byte).
- New codes: `cargo xtask regression-index --bootstrap --merge`, then correct the row by hand.

## Tài liệu / Docs

`docs/specs/regression-index.md` (mới), `docs/release/release-process.md` (26 bước + hàng gate),
`docs/10-shared/P0-7-TASKS-ARM-SF.md` (D2 26/26).

## Kiểm thử / Testing

`cargo test -p xtask` 96/96 (4 test mới của `regression_index`: ranh giới mã, tên hàm, đọc bảng/CHANGELOG,
tham chiếu test bị xoá ⇒ lỗi); đột biến trên cây thật ×3 đỏ; preflight 26/26.
