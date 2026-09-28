/* settings_window.h — Bảng điều khiển TextVN cho Linux (GTK4)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Cùng bố cục/nhãn với bảng điều khiển Windows (tray/src/settings_dialog.rs):
 *   Điều khiển:   Bảng mã | Kiểu gõ | Phím chuyển
 *   Tùy chọn gõ:  Bật gõ tiếng Việt · Dấu mới/Dấu cũ · Khôi phục từ tiếng Anh · Đặt dấu
 *                 tự do · Tự viết hoa chữ đầu câu · Quick Telex · Gõ tắt khi tắt tiếng Việt
 *   Nút:          Hướng dẫn · Thông tin · Gõ tắt... · Mặc định · Đóng
 * (Nhóm "Hệ thống" chỉ có trên Windows: IBus/Fcitx5 tự khởi động cùng phiên đăng nhập.)
 * Đặc tả: docs/release/ui-spec.md. Logic không-GTK: settings_model.h.
 */

#ifndef TEXTVN_SETTINGS_WINDOW_H
#define TEXTVN_SETTINGS_WINDOW_H

#include <gtk/gtk.h>

#include "settings_model.h"

G_BEGIN_DECLS

/* Tạo (hoặc trả cửa sổ đang có) bảng điều khiển. */
GtkWidget *textvn_settings_window_create(GtkApplication *app, const char *custom_config_path);

G_END_DECLS

#endif /* TEXTVN_SETTINGS_WINDOW_H */
