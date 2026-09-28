cmake_minimum_required(VERSION 3.16)
project(textvn-settings C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

include(GNUInstallDirs)

find_package(PkgConfig QUIET)
if(PkgConfig_FOUND)
    pkg_check_modules(GTK4 gtk4)
    pkg_check_modules(GLIB glib-2.0 gobject-2.0)
endif()

# Locate libtextvn_ffi from Rust build output (P0-2 §1)
set(TEXTVN_FFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release" CACHE PATH "Path to Rust target/release directory")
find_library(TEXTVN_FFI_LIB NAMES textvn_ffi libtextvn_ffi PATHS ${TEXTVN_FFI_DIR} NO_DEFAULT_PATH)
if(NOT TEXTVN_FFI_LIB)
    find_library(TEXTVN_FFI_LIB NAMES textvn_ffi libtextvn_ffi)
endif()

# Subdirectory for linux-common
if(NOT TARGET textvn-linux-common)
    add_subdirectory(../linux-common ${CMAKE_CURRENT_BINARY_DIR}/linux-common)
endif()

# Main executable for Linux system with GTK4
if(GTK4_FOUND AND GLIB_FOUND)
    add_executable(textvn-settings
        src/main.c
        src/settings_window.c
    )

    target_include_directories(textvn-settings PRIVATE
        src
        ${CMAKE_CURRENT_SOURCE_DIR}/../linux-common/include
        ${CMAKE_CURRENT_SOURCE_DIR}/../../ffi/include
        ${GTK4_INCLUDE_DIRS}
        ${GLIB_INCLUDE_DIRS}
    )

    target_link_libraries(textvn-settings PRIVATE
        textvn-linux-common
        ${GTK4_LIBRARIES}
        ${GLIB_LIBRARIES}
        pthread
        m
    )

    install(TARGETS textvn-settings DESTINATION "${CMAKE_INSTALL_BINDIR}")
endif()

# Standalone UI logic test target with mock GTK4
add_executable(test_settings_ui
    tests/test_settings.c
    src/settings_window.c
)

target_compile_definitions(test_settings_ui PRIVATE TEXTVN_GTK_MOCK=1)
# Test phải giữ assert kể cả bản Release (NDEBUG).
target_compile_options(test_settings_ui PRIVATE -UNDEBUG)

target_include_directories(test_settings_ui PRIVATE
    src
    tests
    ${CMAKE_CURRENT_SOURCE_DIR}/../linux-common/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../../ffi/include
)

target_link_libraries(test_settings_ui PRIVATE
    textvn-linux-common
)

enable_testing()
add_test(NAME test_settings_ui COMMAND test_settings_ui)
