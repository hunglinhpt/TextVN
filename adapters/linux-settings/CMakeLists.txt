cmake_minimum_required(VERSION 3.16)
project(textvn-settings C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

include(GNUInstallDirs)

find_package(PkgConfig QUIET)
if(PkgConfig_FOUND)
    pkg_check_modules(GTK4 gtk4)
endif()

# Engine Rust (staticlib): chứa cả C API cấu hình (ffi/include/textvn_settings.h).
if(NOT TEXTVN_FFI_LIB)
    set(TEXTVN_FFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release" CACHE PATH "Rust target/release")
    find_library(TEXTVN_FFI_LIB NAMES libtextvn_ffi.a textvn_ffi PATHS ${TEXTVN_FFI_DIR} NO_DEFAULT_PATH)
endif()
if(NOT TEXTVN_FFI_LIB)
    message(FATAL_ERROR "Chưa có libtextvn_ffi.a — chạy: cargo build --release -p textvn-ffi")
endif()

if(NOT TARGET textvn-linux-common)
    set(TEXTVN_LINUX_COMMON_NO_TESTS ON)
    add_subdirectory(../linux-common ${CMAKE_CURRENT_BINARY_DIR}/linux-common)
endif()

# Phiên bản hiển thị ở "Thông tin" = version workspace Cargo (một nguồn sự thật).
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/../../Cargo.toml" _ver_line REGEX "^version = \"[^\"]+\"" LIMIT_COUNT 1)
string(REGEX REPLACE "^version = \"([^\"]+)\"" "\\1" TEXTVN_VERSION "${_ver_line}")

set(TEXTVN_SETTINGS_WARN -Wall -Wextra -Werror)

# Logic không-GTK (test được trên CI không có màn hình).
add_library(textvn-settings-model STATIC src/settings_model.c)
target_include_directories(textvn-settings-model PUBLIC
    src
    ${CMAKE_CURRENT_SOURCE_DIR}/../linux-common/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../../ffi/include
)
target_compile_options(textvn-settings-model PRIVATE ${TEXTVN_SETTINGS_WARN})
target_link_libraries(textvn-settings-model PUBLIC textvn-linux-common ${TEXTVN_FFI_LIB} pthread dl m)

if(GTK4_FOUND)
    add_executable(textvn-settings src/main.c src/settings_window.c)
    target_include_directories(textvn-settings PRIVATE ${GTK4_INCLUDE_DIRS})
    target_compile_definitions(textvn-settings PRIVATE TEXTVN_VERSION="${TEXTVN_VERSION}")
    target_compile_options(textvn-settings PRIVATE ${TEXTVN_SETTINGS_WARN} ${GTK4_CFLAGS_OTHER})
    target_link_libraries(textvn-settings PRIVATE textvn-settings-model ${GTK4_LDFLAGS})
    install(TARGETS textvn-settings DESTINATION "${CMAKE_INSTALL_BINDIR}")
else()
    message(STATUS "gtk4 không có — chỉ build logic + test (không có textvn-settings)")
endif()

enable_testing()
add_executable(test_settings_model tests/test_settings_model.c)
# Test phải giữ assert kể cả bản Release (NDEBUG).
target_compile_options(test_settings_model PRIVATE ${TEXTVN_SETTINGS_WARN} -UNDEBUG)
target_link_libraries(test_settings_model PRIVATE textvn-settings-model)
add_test(NAME test_settings_model COMMAND test_settings_model)
