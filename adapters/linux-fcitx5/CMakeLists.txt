cmake_minimum_required(VERSION 3.16)
project(fcitx5-textvn LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(GNUInstallDirs)

find_package(Fcitx5Core 5.0.14 REQUIRED)
find_package(Fcitx5Utils REQUIRED)

# Engine Rust (staticlib, P0-2 §1): `cargo build --release -p textvn-ffi`.
set(TEXTVN_FFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release" CACHE PATH "Rust target/release")
find_library(TEXTVN_FFI_LIB NAMES libtextvn_ffi.a textvn_ffi PATHS ${TEXTVN_FFI_DIR} NO_DEFAULT_PATH)
if(NOT TEXTVN_FFI_LIB)
    message(FATAL_ERROR "libtextvn_ffi chưa build: chạy `cargo build --release -p textvn-ffi`")
endif()

if(NOT TARGET textvn-linux-common)
    add_subdirectory(../linux-common ${CMAKE_CURRENT_BINARY_DIR}/linux-common)
endif()

set(TEXTVN_FCITX5_WARN -Wall -Wextra -Werror)

add_library(textvn-fcitx5-core STATIC src/engine.cpp)
set_target_properties(textvn-fcitx5-core PROPERTIES POSITION_INDEPENDENT_CODE ON)
target_include_directories(textvn-fcitx5-core PUBLIC src)
target_compile_options(textvn-fcitx5-core PRIVATE ${TEXTVN_FCITX5_WARN})
target_link_libraries(textvn-fcitx5-core PUBLIC
    Fcitx5::Core Fcitx5::Utils textvn-linux-common ${TEXTVN_FFI_LIB} pthread dl m)

# Module nạp bởi fcitx5: libtextvn-fcitx5.so (khớp `Library=libtextvn-fcitx5`).
add_library(textvn-fcitx5 MODULE src/addon.cpp)
set_target_properties(textvn-fcitx5 PROPERTIES PREFIX "lib" OUTPUT_NAME "textvn-fcitx5")
target_compile_options(textvn-fcitx5 PRIVATE ${TEXTVN_FCITX5_WARN})
target_link_libraries(textvn-fcitx5 PRIVATE textvn-fcitx5-core)
# Staticlib Rust nằm trong .so: chỉ export điểm vào của Fcitx5.
target_link_options(textvn-fcitx5 PRIVATE -Wl,--exclude-libs,ALL -Wl,--no-undefined)

if(DEFINED FCITX_INSTALL_ADDONDIR)
    install(TARGETS textvn-fcitx5 DESTINATION "${FCITX_INSTALL_ADDONDIR}")
else()
    install(TARGETS textvn-fcitx5 DESTINATION "${CMAKE_INSTALL_LIBDIR}/fcitx5")
endif()
install(FILES ../../packaging/linux/fcitx5/addon/textvn.conf
        DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/addon")
install(FILES ../../packaging/linux/fcitx5/inputmethod/textvn.conf
        DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod")

enable_testing()
add_executable(test_fcitx5_keymap tests/test_keymap.cpp)
target_compile_options(test_fcitx5_keymap PRIVATE ${TEXTVN_FCITX5_WARN} -UNDEBUG)
target_link_libraries(test_fcitx5_keymap PRIVATE textvn-fcitx5-core)
add_test(NAME test_fcitx5_keymap COMMAND test_fcitx5_keymap)
