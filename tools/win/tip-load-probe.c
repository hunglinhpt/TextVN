// SPDX-License-Identifier: GPL-3.0-or-later
// tip-load-probe.c - ARM-W5b: chung minh TIP TextVN DA CAI duoc nap DUNG kien truc trong tien trinh
// thuc - khong can route input (E89: runner windows-11-arm khong dua focus/phim toi cua so).
//
// CoCreateInstance(CLSID_TEXTVN_TIP, CLSCTX_INPROC_SERVER) di dung duong COM that: HKCR\CLSID\...\
// InprocServer32 do bo cai ghi (64-bit view = forwarder ARM64X textvn-tsf.dll; WOW6432Node =
// textvn-tsf-x86.dll). Sau do kiem DLL dich nao dang nam trong tien trinh:
//   ARM64 native  -> textvn-tsf-arm64.dll (qua forwarder)
//   x64 gia lap   -> textvn-tsf-x64.dll   (qua forwarder, view EC)
//   x86 (WOW64)   -> textvn-tsf-x86.dll
// Dung: tip-load-probe.exe <ten-dll-mong-doi>   exit 0 = dung; 1 = CoCreateInstance loi; 2 = sai DLL.
// Khong ghi registry, khong go phim, khong log chu (G8). Build: cl /nologo /W4 tip-load-probe.c ole32.lib
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <objbase.h>
#include <stdio.h>
#include <wchar.h>

// adapters/windows-tsf/src/guids.rs CLSID_TEXTVN_TIP = {6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}
static const CLSID kClsidTip = {0x6F2B9C31, 0x8E47, 0x4D2A, {0x9C, 0x84, 0x1D, 0x5A, 0x3E, 0x70, 0xF9, 0xB8}};
// ITfTextInputProcessor = {AA80E7F7-2021-11D2-93E0-0060B067B86E}
static const IID kIidTip = {0xAA80E7F7, 0x2021, 0x11D2, {0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E}};

static const wchar_t *kTargets[] = {L"textvn-tsf-arm64.dll", L"textvn-tsf-x64.dll", L"textvn-tsf-x86.dll"};

static const char *arch(void) {
#if defined(_M_ARM64)
    return "arm64";
#elif defined(_M_X64)
    return "x64";
#elif defined(_M_IX86)
    return "x86";
#else
    return "other";
#endif
}

int wmain(int argc, wchar_t **argv) {
    if (argc != 2) {
        fwprintf(stderr, L"dung: tip-load-probe.exe <textvn-tsf-<arch>.dll>\n");
        return 64;
    }
    const wchar_t *want = argv[1];
    HRESULT hr = CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);
    if (FAILED(hr)) {
        printf("tip-load-probe arch=%s CoInitializeEx hr=0x%08lX => FAIL\n", arch(), (unsigned long)hr);
        return 1;
    }
    IUnknown *tip = NULL;
    hr = CoCreateInstance(&kClsidTip, NULL, CLSCTX_INPROC_SERVER, &kIidTip, (void **)&tip);
    if (FAILED(hr) || tip == NULL) {
        printf("tip-load-probe arch=%s CoCreateInstance hr=0x%08lX => FAIL\n", arch(), (unsigned long)hr);
        CoUninitialize();
        return 1;
    }
    int wanted_loaded = 0, other_loaded = 0;
    printf("tip-load-probe arch=%s CoCreateInstance=ok loaded:", arch());
    for (size_t i = 0; i < sizeof(kTargets) / sizeof(kTargets[0]); i++) {
        if (GetModuleHandleW(kTargets[i]) != NULL) {
            printf(" %ls", kTargets[i]);
            if (_wcsicmp(kTargets[i], want) == 0) {
                wanted_loaded = 1;
            } else {
                other_loaded = 1;
            }
        }
    }
    int ok = wanted_loaded && !other_loaded;
    printf(" (mong doi %ls) => %s\n", want, ok ? "PASS" : "FAIL");
    tip->lpVtbl->Release(tip);
    CoUninitialize();
    return ok ? 0 : 2;
}
