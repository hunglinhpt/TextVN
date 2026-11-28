/* SPDX-License-Identifier: GPL-3.0-or-later */
/* e2e-editor.c — ARM-W5: bo sua Win32 toi gian de E2E go chu TSF tren app KHA CUNG
 * KIEN TRUC (x64 gia lap / x86 tren may ARM64). Mot cua so chinh chua MOT control
 * Edit phu hon client — dung class "Edit" chuan de EditOf cua test-typing.ps1 tim
 * thay ngay. Khong menu, khong file, khong clipboard: chi nhan ban phim.
 *
 * Build (trong job CI hoac may dev, tu thu muc tools/win):
 *   x64: cl /nologo /W3 /O1 /Fe:e2e-editor-x64.exe e2e-editor.c user32.lib
 *   x86: cl /nologo /W3 /O1 /Fe:e2e-editor-x86.exe e2e-editor.c user32.lib
 *          (can bo toolset x86 — treo WARNING + bo qua neu runner khong co)
 */
#include <windows.h>

static LRESULT CALLBACK WndProc(HWND h, UINT m, WPARAM w, LPARAM l) {
    switch (m) {
    case WM_SIZE: {
        HWND edit = GetDlgItem(h, 1);
        if (edit) MoveWindow(edit, 0, 0, LOWORD(l), HIWORD(l), TRUE);
        return 0;
    }
    case WM_DESTROY:
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(h, m, w, l);
}

int WINAPI wWinMain(HINSTANCE inst, HINSTANCE prev, PWSTR cmd, int show) {
    WNDCLASSW wc;
    HWND w, edit;
    MSG msg;
    (void)prev; (void)cmd;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = inst;
    wc.hCursor = LoadCursorW(NULL, (LPCWSTR)IDC_ARROW);
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wc.lpszClassName = L"TextVNE2EEditor";
    if (!RegisterClassW(&wc)) return 1;
    w = CreateWindowExW(0, wc.lpszClassName, L"TextVN E2E Editor",
                        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                        CW_USEDEFAULT, CW_USEDEFAULT, 640, 420,
                        NULL, NULL, inst, NULL);
    if (!w) return 1;
    edit = CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", L"",
                           WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE
                               | ES_AUTOVSCROLL,
                           0, 0, 640, 420, w, (HMENU)1, inst, NULL);
    if (!edit) return 1;
    SetFocus(edit);
    while (GetMessageW(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    return 0;
}
