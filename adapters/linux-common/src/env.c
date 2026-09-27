/* env.c — Environment variable check and framework detection (P3-4 §7)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Checks GTK_IM_MODULE, QT_IM_MODULE, XMODIFIERS for textvn doctor diagnostics.
 */

#include "linux_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int lc_detect_active_framework(void) {
    const char *gtk_im = getenv("GTK_IM_MODULE");
    const char *qt_im = getenv("QT_IM_MODULE");
    const char *xmod = getenv("XMODIFIERS");
    const char *desktop = getenv("XDG_CURRENT_DESKTOP");

    /* Prioritize explicit fcitx settings */
    if ((gtk_im && strstr(gtk_im, "fcitx")) ||
        (qt_im && strstr(qt_im, "fcitx"))   ||
        (xmod && strstr(xmod, "fcitx"))) {
        return LC_FRAMEWORK_FCITX5;
    }

    /* Check explicit ibus settings */
    if ((gtk_im && strstr(gtk_im, "ibus")) ||
        (qt_im && strstr(qt_im, "ibus"))   ||
        (xmod && strstr(xmod, "ibus"))) {
        return LC_FRAMEWORK_IBUS;
    }

    /* Fallback by desktop environment default */
    if (desktop) {
        if (strstr(desktop, "KDE")) return LC_FRAMEWORK_FCITX5;
        if (strstr(desktop, "GNOME") || strstr(desktop, "Ubuntu")) return LC_FRAMEWORK_IBUS;
    }

    return LC_FRAMEWORK_UNKNOWN;
}

int lc_env_check(lc_env_info *out_info, char *out_report, size_t report_len) {
    if (!out_info) return -1;
    memset(out_info, 0, sizeof(*out_info));

    const char *gtk_im = getenv("GTK_IM_MODULE");
    const char *qt_im = getenv("QT_IM_MODULE");
    const char *xmod = getenv("XMODIFIERS");
    const char *desktop = getenv("XDG_CURRENT_DESKTOP");

    if (gtk_im) strncpy(out_info->gtk_im_module, gtk_im, sizeof(out_info->gtk_im_module) - 1);
    if (qt_im) strncpy(out_info->qt_im_module, qt_im, sizeof(out_info->qt_im_module) - 1);
    if (xmod) strncpy(out_info->xmodifiers, xmod, sizeof(out_info->xmodifiers) - 1);
    if (desktop) strncpy(out_info->xdg_current_desktop, desktop, sizeof(out_info->xdg_current_desktop) - 1);

    out_info->detected_framework = lc_detect_active_framework();

    /* Check consistency */
    int is_fcitx = (out_info->detected_framework == LC_FRAMEWORK_FCITX5);
    int is_ibus = (out_info->detected_framework == LC_FRAMEWORK_IBUS);

    bool gtk_ok = false;
    bool qt_ok = false;
    bool xmod_ok = false;

    if (is_fcitx) {
        gtk_ok = (gtk_im && strstr(gtk_im, "fcitx"));
        qt_ok = (qt_im && strstr(qt_im, "fcitx"));
        xmod_ok = (xmod && strstr(xmod, "fcitx"));
    } else if (is_ibus) {
        gtk_ok = (gtk_im && strstr(gtk_im, "ibus"));
        qt_ok = (qt_im && strstr(qt_im, "ibus"));
        xmod_ok = (xmod && strstr(xmod, "ibus"));
    } else {
        gtk_ok = qt_ok = xmod_ok = true;
    }

    out_info->is_consistent = (gtk_ok && qt_ok && xmod_ok) ? 1 : 0;

    if (out_report && report_len > 0) {
        const char *fw_name = (is_fcitx ? "Fcitx5" : (is_ibus ? "IBus" : "Unknown"));
        char rec[512] = {0};

        if (!out_info->is_consistent) {
            if (is_fcitx) {
                snprintf(rec, sizeof(rec),
                         "Inconsistent IM environment for Fcitx5! Recommended exports in ~/.profile:\n"
                         "  export GTK_IM_MODULE=fcitx\n"
                         "  export QT_IM_MODULE=fcitx\n"
                         "  export XMODIFIERS=@im=fcitx");
            } else if (is_ibus) {
                snprintf(rec, sizeof(rec),
                         "Inconsistent IM environment for IBus! Recommended exports in ~/.profile:\n"
                         "  export GTK_IM_MODULE=ibus\n"
                         "  export QT_IM_MODULE=ibus\n"
                         "  export XMODIFIERS=@im=ibus");
            }
        } else {
            snprintf(rec, sizeof(rec), "Environment consistent for %s.", fw_name);
        }

        snprintf(out_report, report_len,
                 "Desktop: %s | Active Framework: %s\n"
                 "  GTK_IM_MODULE = %s (%s)\n"
                 "  QT_IM_MODULE  = %s (%s)\n"
                 "  XMODIFIERS    = %s (%s)\n"
                 "Status: %s\n%s",
                 out_info->xdg_current_desktop[0] ? out_info->xdg_current_desktop : "n/a",
                 fw_name,
                 out_info->gtk_im_module[0] ? out_info->gtk_im_module : "<unset>", gtk_ok ? "OK" : "WARN",
                 out_info->qt_im_module[0] ? out_info->qt_im_module : "<unset>", qt_ok ? "OK" : "WARN",
                 out_info->xmodifiers[0] ? out_info->xmodifiers : "<unset>", xmod_ok ? "OK" : "WARN",
                 out_info->is_consistent ? "PASS" : "ACTION REQUIRED",
                 rec);
    }

    return 0;
}
