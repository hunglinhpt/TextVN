cmake_minimum_required(VERSION 3.16)
project(textvn-ibus C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

include(GNUInstallDirs)

find_package(PkgConfig REQUIRED)
pkg_check_modules(IBUS REQUIRED IMPORTED_TARGET ibus-1.0)

# Engine Rust (staticlib, P0-2 §1): `cargo build --release -p textvn-ffi`.
set(TEXTVN_FFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release" CACHE PATH "Rust target/release")
find_library(TEXTVN_FFI_LIB NAMES libtextvn_ffi.a textvn_ffi PATHS ${TEXTVN_FFI_DIR} NO_DEFAULT_PATH)
if(NOT TEXTVN_FFI_LIB)
    message(FATAL_ERROR "libtextvn_ffi chưa build: chạy `cargo build --release -p textvn-ffi`")
endif()

if(NOT TARGET textvn-linux-common)
    add_subdirectory(../linux-common ${CMAKE_CURRENT_BINARY_DIR}/linux-common)
endif()

set(TEXTVN_IBUS_WARN -Wall -Wextra -Werror)

add_library(textvn-ibus-core STATIC src/engine.c src/keymap.c)
target_include_directories(textvn-ibus-core PUBLIC include src)
target_compile_options(textvn-ibus-core PRIVATE ${TEXTVN_IBUS_WARN})
target_link_libraries(textvn-ibus-core PUBLIC
    textvn-linux-common ${TEXTVN_FFI_LIB} PkgConfig::IBUS pthread dl m)

add_executable(textvn-ibus-engine src/main.c)
target_compile_options(textvn-ibus-engine PRIVATE ${TEXTVN_IBUS_WARN})
target_link_libraries(textvn-ibus-engine PRIVATE textvn-ibus-core)

install(TARGETS textvn-ibus-engine DESTINATION "${CMAKE_INSTALL_LIBDIR}/textvn")
install(FILES ../../packaging/linux/ibus/textvn.xml DESTINATION "${CMAKE_INSTALL_DATADIR}/ibus/component")

enable_testing()
add_executable(test_ibus_keymap tests/test_keymap.c)
target_compile_options(test_ibus_keymap PRIVATE ${TEXTVN_IBUS_WARN} -UNDEBUG)
target_link_libraries(test_ibus_keymap PRIVATE textvn-ibus-core)
add_test(NAME test_ibus_keymap COMMAND test_ibus_keymap)

# Kiểm thử đầu-cuối với ibus-daemon thật — chạy qua scripts/e2e-linux.sh.
add_executable(e2e_ibus tests/e2e_ibus.c)
target_link_libraries(e2e_ibus PRIVATE PkgConfig::IBUS)
