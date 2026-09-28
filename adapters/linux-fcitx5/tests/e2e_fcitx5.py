#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Kiểm thử đầu-cuối với fcitx5 THẬT qua DBus frontend (org.fcitx.Fcitx.InputMethod1).

Chạy qua scripts/e2e-linux.sh: fcitx5 nạp libtextvn-fcitx5.so từ thư mục build.
Đóng vai app: tạo input context, gửi phím, nhận CommitString / UpdateFormattedPreedit;
phím engine không nuốt thì "app" tự xử lý.
"""

import json
import os
import sys
import time

import dbus
import dbus.mainloop.glib
from gi.repository import GLib

KEY = {
    "\n": 0xFF0D,
    "\b": 0xFF08,
    "\t": 0xFF09,
    " ": 0x20,
}
CTRL_L, SHIFT_L = 0xFFE3, 0xFFE1
STATE_SHIFT, STATE_CTRL = 1 << 0, 1 << 2
CAP_PREEDIT, CAP_FORMATTED, CAP_PASSWORD = 1 << 1, 1 << 4, 1 << 3
CAP_UNFOCUS_COMMIT = 1 << 5

dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus()
loop_ctx = GLib.MainContext.default()

doc = []
preedit = [""]


def pump(sec=0.05):
    end = time.time() + sec
    while time.time() < end:
        while loop_ctx.iteration(False):
            pass
        time.sleep(0.005)


def wait_service():
    for _ in range(100):
        if bus.name_has_owner("org.fcitx.Fcitx5"):
            return True
        time.sleep(0.1)
    return False


if not wait_service():
    print("fcitx5 không lên DBus", file=sys.stderr)
    sys.exit(1)

im = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/org/freedesktop/portal/inputmethod"),
                    "org.fcitx.Fcitx.InputMethod1")
path, _uuid = im.CreateInputContext([("program", "textvn-e2e"), ("display", "x11:")])
icobj = bus.get_object("org.fcitx.Fcitx5", path)
ic = dbus.Interface(icobj, "org.fcitx.Fcitx.InputContext1")
controller = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/controller"),
                            "org.fcitx.Fcitx.Controller1")


def on_commit(text):
    doc.append(str(text))


def on_preedit(segments, cursor):
    preedit[0] = "".join(str(s[0]) for s in segments)


icobj.connect_to_signal("CommitString", on_commit, dbus_interface="org.fcitx.Fcitx.InputContext1")
icobj.connect_to_signal("UpdateFormattedPreedit", on_preedit,
                        dbus_interface="org.fcitx.Fcitx.InputContext1")

ic.SetCapability(dbus.UInt64(CAP_PREEDIT | CAP_FORMATTED))
ic.FocusIn()
pump(0.2)
controller.SetCurrentIM("textvn")
pump(0.3)
if str(controller.CurrentInputMethod()) != "textvn":
    print("không kích hoạt được IM textvn:", controller.CurrentInputMethod(), file=sys.stderr)
    sys.exit(1)


def app_handles(ch):
    if ch == "\b":
        if doc and doc[-1]:
            doc[-1] = doc[-1][:-1]
    else:
        doc.append(ch)


def send(keysym, state, release):
    return bool(ic.ProcessKeyEvent(dbus.UInt32(keysym), dbus.UInt32(0), dbus.UInt32(state),
                                   dbus.Boolean(release), dbus.UInt32(0)))


def press_char(ch):
    keysym = KEY.get(ch, ord(ch))
    state = STATE_SHIFT if ch.isupper() else 0
    handled = send(keysym, state, False)
    pump()
    if not handled:
        if preedit[0]:
            print("B2 vi phạm: app nhận phím khi preedit còn mở:", preedit[0], file=sys.stderr)
            sys.exit(2)
        app_handles(ch)
    send(keysym, state, True)
    pump()


def type_(s):
    for ch in s:
        press_char(ch)


def text():
    return "".join(doc) + preedit[0]


ok = True


def check(label, want):
    global ok
    got = text()
    good = got == want
    ok &= good
    print(f"{'PASS' if good else 'FAIL'} {label:28} want={want!r} got={got!r}")


def clear():
    ic.Reset()
    pump(0.1)
    doc.clear()
    preedit[0] = ""


def ctrl_shift_tap():
    send(CTRL_L, 0, False)
    pump()
    send(SHIFT_L, STATE_CTRL, False)
    pump()
    send(SHIFT_L, STATE_CTRL | STATE_SHIFT, True)
    pump()
    send(CTRL_L, STATE_CTRL, True)
    pump()


type_("dduocj ")
check("telex dduocj+space", "được ")
clear()
type_("Vieetj Nam")
check("preedit giữ cả từ", "Việt Nam")
clear()
type_("chaof banj\n")
check("Enter commit (B2)", "chào bạn\n")
clear()
type_("dduocj\b ")
check("Backspace trong từ", "đươc ")
clear()
type_("hello world ")
check("tiếng Anh giữ nguyên", "hello world ")
clear()

type_("tieengs")
ic.FocusOut()
pump(0.2)
check("focus-out commit", "tiếng")
ic.FocusIn()
pump(0.2)
clear()

# Client tự commit khi mất focus (fcitx5-gtk/qt đặt ClientUnfocusCommit): không lặp.
ic.SetCapability(dbus.UInt64(CAP_PREEDIT | CAP_FORMATTED | CAP_UNFOCUS_COMMIT))
pump(0.1)
type_("tieengs")
pending = preedit[0]
ic.FocusOut()
pump(0.2)
doc.append(pending)  # client commit preedit của nó
preedit[0] = ""
check("focus-out (client commit)", "tiếng")
ic.FocusIn()
pump(0.2)
clear()
ic.SetCapability(dbus.UInt64(CAP_PREEDIT | CAP_FORMATTED))
pump(0.1)

type_("tieengs")
ic.Reset()
pump(0.2)
check("reset commit 1 lần", "tiếng")
doc.clear()
preedit[0] = ""

CONFIG_DIR = os.path.join(os.environ.get("XDG_CONFIG_HOME") or os.path.expanduser("~/.config"),
                          "TextVN")


def write_textvn_file(name, data):
    """Ghi như bảng cài đặt (tmp → rename); chờ để mtime chắc chắn khác lần trước."""
    os.makedirs(CONFIG_DIR, exist_ok=True)
    time.sleep(0.02)
    tmp = os.path.join(CONFIG_DIR, f".{name}.e2e.tmp")
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False)
    os.replace(tmp, os.path.join(CONFIG_DIR, name))


def state_says(enabled):
    global ok
    try:
        with open(os.path.join(CONFIG_DIR, "state.json"), encoding="utf-8") as f:
            got = json.load(f).get("global_enabled")
    except OSError:
        got = None
    good = got is enabled
    ok &= good
    print(f"{'PASS' if good else 'FAIL'} state.json global_enabled={enabled}")


# Ctrl+Shift đổi VN/EN và lưu vào state.json (nhớ qua lần khởi động sau).
ctrl_shift_tap()
type_("as ")
check("Ctrl+Shift → EN", "as ")
state_says(False)
clear()
ctrl_shift_tap()
type_("as ")
check("Ctrl+Shift → VN", "á ")
state_says(True)
clear()

# Bảng cài đặt đổi state.json khi IME đang chạy → theo ngay ở phím kế tiếp.
write_textvn_file("state.json", {"global_enabled": False})
type_("as ")
check("state.json → EN", "as ")
write_textvn_file("state.json", {"global_enabled": True})
type_("as ")
check("state.json → VN", "as á ")
clear()

MACROS = [{"trigger": "vn", "expand": "Việt Nam"}]
write_textvn_file("config.json", {"config_version": 1, "auto_capitalize": False, "macros": MACROS})
type_("xin vn\t")
check("gõ tắt khi bật VN", "xin Việt Nam")
clear()
write_textvn_file("state.json", {"global_enabled": False})
type_("xin vn\t")
check("tắt VN: không gõ tắt", "xin vn\t")
clear()
write_textvn_file("config.json", {"config_version": 1, "auto_capitalize": False,
                                  "allow_macro_when_vi_off": True, "macros": MACROS})
type_("xin vieetj vn\t")
check("tắt VN + gõ tắt", "xin vieetj Việt Nam")
clear()
type_("hello world\n")
check("tắt VN: chữ Anh nguyên vẹn", "hello world\n")
clear()
write_textvn_file("state.json", {"global_enabled": True})
write_textvn_file("config.json", {"config_version": 1})
type_("as ")
check("trở lại VN", "á ")
clear()

ic.SetCapability(dbus.UInt64(CAP_PREEDIT | CAP_FORMATTED | CAP_PASSWORD))
pump(0.1)
type_("duocj")
check("ô mật khẩu passthrough", "duocj")

print("e2e_fcitx5: OK" if ok else "e2e_fcitx5: FAILED")
sys.exit(0 if ok else 1)
