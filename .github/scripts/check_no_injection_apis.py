#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Check API inject / giong malware bi cam (Farch-3).

Chay local:  python .github/scripts/check_no_injection_apis.py
Dung trong:  .github/workflows/repo-hygiene.yml (check #8)
Chinh sach:  docs/specs/antivirus-false-positive.md §8 — TSF/IME chinh thong la
             duong gom mac dinh; global hook chi WH_KEYBOARD_LL + SendInput
             (opt-in 2 lop); khong dung process injection, DLL inject qua hook
             non-LL, hay doc/ghi vao bo nho process khac. Khong co IME nao can
             cac API nay — nguoi dung 2026-09-28 dat bat buoc "khong viet code
             giong malware" de tranh heuristic AV (Kaspersky/Defender).
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SELF = pathlib.Path(__file__).resolve().relative_to(ROOT).as_posix()
CODE_SUFFIX = {
    ".rs", ".c", ".h", ".cpp", ".hpp", ".cc",
    ".py", ".ps1", ".sh", ".iss",
}
# spikes/ = ma nhap probe khong phat hanh (hook-spike workflow, WIN-005):
# khai bao P/Invoke va goi `SetWindowsHookExW(13, ...)` (13 = WH_KEYBOARD_LL)
# la hop phai; guard ap cho code phat hanh. Noi dung grep spikes/ van xay
# khi copy vao adapters/ (bi chan tai do).
SKIP_PREFIX = ("spikes/",)

# Ten API nhap van — khong co ly do hop phap trong mot IME.
BANNED = (
    # thread/process injection
    "CreateRemoteThread",
    "RtlCreateUserThread",
    "QueueUserAPC",
    "SetThreadContext",
    "Wow64SetContextThread",
    "NtSetContextThread",
    # doc/ghi bo nho process khac
    "WriteProcessMemory",
    "VirtualAllocEx",
    "NtWriteVirtualMemory",
    "process_vm_writev",
    "PTRACE_ATTACH",
)

# Hook non-LL (WH_KEYBOARD/WH_MOUSE) can inject DLL vao process dich — chi
# ban *_LL duoc phep (khong inject, khong doc phim cua app khac qua DLL).
NON_LL_HOOK_RE = re.compile(r"\b(?:WH_KEYBOARD|WH_MOUSE)\b")
# SetWindowsHookEx con lai phai ghi ro mot LL hook tren cung dong.
SET_HOOK_RE = re.compile(r"\bSetWindowsHookEx\w*\s*\(")
LL_OK_RE = re.compile(r"\bWH_(?:KEYBOARD|MOUSE)_LL\b")


def main() -> int:
    listed = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout.decode("utf-8", "replace")

    violations = []
    scanned = 0
    for rel in listed.split("\0"):
        if not rel or rel == SELF:
            continue
        if rel.startswith(SKIP_PREFIX):
            continue
        if pathlib.PurePosixPath(rel).suffix.lower() not in CODE_SUFFIX:
            continue
        path = ROOT / rel
        if not path.is_file():
            continue
        scanned += 1
        text = path.read_text(encoding="utf-8", errors="replace")
        for lineno, line in enumerate(text.splitlines(), 1):
            for name in BANNED:
                if name in line:
                    violations.append(f"{rel}:{lineno}: {name}")
            if NON_LL_HOOK_RE.search(line):
                violations.append(
                    f"{rel}:{lineno}: hook non-LL (chi WH_KEYBOARD_LL/WH_MOUSE_LL duoc phep)"
                )
            elif SET_HOOK_RE.search(line) and not LL_OK_RE.search(line):
                violations.append(
                    f"{rel}:{lineno}: SetWindowsHookEx khong ghi ro LL hook tren cung dong"
                )

    if violations:
        print("API inject/giong malware bi cam (antivirus-false-positive.md §8):")
        for item in violations:
            print(f"  {item}")
        return 1
    print(f"check_no_injection_apis: OK ({scanned} file code, khong co API bi cam)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
