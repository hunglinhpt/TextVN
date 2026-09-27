/* test_env.c — Unit tests for environment matrix check
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "linux_common.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

static void test_env_report_generation(void) {
    lc_env_info info;
    char report[1024];

    assert(lc_env_check(&info, report, sizeof(report)) == 0);
    assert(strlen(report) > 0);
    assert(strstr(report, "GTK_IM_MODULE") != NULL);
    assert(strstr(report, "QT_IM_MODULE") != NULL);
    assert(strstr(report, "XMODIFIERS") != NULL);
}

int main(void) {
    test_env_report_generation();
    printf("All linux-common environment tests passed successfully!\n");
    return 0;
}
