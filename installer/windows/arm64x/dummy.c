// SPDX-License-Identifier: GPL-3.0-or-later
// ARM-W2: intentionally empty translation unit compiled twice by MSVC
// (`cl /c` for ARM64 native and `cl /c /arm64EC` for ARM64EC) so
// `link /dll /noentry /machine:arm64x` produces a pure ARM64X forwarder DLL
// with zero code and zero CRT imports.
