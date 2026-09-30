// SPDX-License-Identifier: GPL-3.0-or-later
// shim.c — CTextVNFFI là module header-only. Build universal (`swift build
// --arch arm64 --arch x86_64` → backend XCBuild) đòi mỗi target có object
// `CTextVNFFI_Module.o`; thiếu file nguồn thì link báo "Build input file cannot
// be found" (MAC-032, lần đầu job package-candidate chạy). File này chỉ để
// target sinh object — không định nghĩa symbol nào.
#include "textvn_ffi.h"
