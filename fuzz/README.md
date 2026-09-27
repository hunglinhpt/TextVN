# fuzz — L4 (P0-4 §1)

Target fuzz cho các **đường input không tin được**: ABI engine, parser `config.v1`, parser preset `appdb.v1`.
Crate này **không thuộc workspace chính** (`fuzz/Cargo.toml` có `[workspace]` riêng) vì `cargo-fuzz`
cần nightly + sanitizer, còn repo build bằng stable (P0-1 §3/§4).

## Chạy

```bash
cargo install cargo-fuzz                 # 1 lần
rustup toolchain install nightly

cargo +nightly fuzz run ffi_key --features ffi-fuzz      -- -max_total_time=60   # Unix CI
cargo +nightly fuzz run config_parse --features ffi-fuzz -- -max_total_time=60   # Unix CI
cargo +nightly fuzz run appdb_parse     -- -max_total_time=60
cargo +nightly fuzz run ffi_key         -- -max_total_time=600  # nightly.yml
```

Chạy song song target: `cargo +nightly fuzz run --jobs 8 -- -max_total_time=600`.
Crash tìm được nằm ở `fuzz/artifacts/` (git-ignore) — **phải** dịch thành unit test
hoặc corpus `.keys` trước khi fix (Handbook §9: reproduces trước, fix sau).

## Bất biến mỗi target assert (không chỉ "không panic")

| Target | Bất biến |
|---|---|
| `ffi_key` | `insert_len`/`preedit_len`/`delete_count` ≤ `IME_MAX_TEXT` (adapter đọc tràn?); `action`/`rc` thuộc tập công bố; **fail-open**: `rc ≠ OK ⇒ action = PASS` (S4); phím `is_injected` luôn PASS (chống loop); abi lệch ⇒ `IME_ERR_ABI`; NULL args ⇒ `IME_ERR_INVALID_ARG` |
| `config_parse` | config sai vẫn tạo instance (không panic, không NULL); **FFI và `vietime-config` cùng kết luận**; `ime_reload_config` nhất quán; `ime_last_error` không echo nội dung config (S2); `ime_suggest` (feature tắt) không trả item |
| `appdb_parse` | `AppDb::parse` không panic với rác; strategy id luôn trong tập header công bố. Trên Windows target này không link FFI vì libFuzzer MSVC không link được dependency `cdylib`; invariant FFI tương ứng chạy stable trong `ffi/tests/abi_invariants.rs`, còn Unix CI fuzz `ffi_key`/`config_parse` với `ffi-fuzz`. |

## Kiểm chứng trên máy không có cargo-fuzz

`ffi/tests/abi_invariants.rs` chạy **cùng bộ invariant** trên **stable** với PRNG tự cài
(xorshift64*, 200 vòng × 64 phím + ~250 config input) → chạy trong `cargo test --workspace`
ở **mọi PR**, 3 OS. Fuzz bổ sung phần *input thật* của libFuzzer; stress test bảo đảm
invariant không bị rơi khi ai đó sửa FFI mà quên fuzz.

## Seed corpus

```bash
mkdir -p fuzz/corpus/ffi_key
printf '\x02\x01\x64\x75\x6f\x63\x6a' > fuzz/corpus/ffi_key/key_duocj
cargo +nightly fuzz run ffi_key fuzz/corpus/ffi_key   # chạy đúng các seed này
```
