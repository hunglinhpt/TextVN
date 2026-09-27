cmake_minimum_required(VERSION 3.16)
project(textvn-ibus C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

include(GNUInstallDirs)

find_package(PkgConfig QUIET)
if(PkgConfig_FOUND)
    pkg_check_modules(IBUS ibus-1.0)
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

# Main executable target for Linux system with IBus
if(IBUS_FOUND AND GLIB_FOUND)
    add_executable(textvn-ibus-engine
        src/main.c
        src/engine.c
        src/keymap.c
        src/apply.c
    )

    target_include_directories(textvn-ibus-engine PRIVATE
        include
        src
        ${CMAKE_CURRENT_SOURCE_DIR}/../linux-common/include
        ${CMAKE_CURRENT_SOURCE_DIR}/../../ffi/include
        ${IBUS_INCLUDE_DIRS}
        ${GLIB_INCLUDE_DIRS}
    )

    target_link_libraries(textvn-ibus-engine PRIVATE
        textvn-linux-common
        ${TEXTVN_FFI_LIB}
        ${IBUS_LIBRARIES}
        ${GLIB_LIBRARIES}
        pthread
        m
    )

    install(TARGETS textvn-ibus-engine DESTINATION "${CMAKE_INSTALL_LIBDIR}/textvn")
    install(FILES conf/textvn.xml DESTINATION "${CMAKE_INSTALL_DATADIR}/ibus/component")
endif()

# Unit test target with mock IBus/GLib headers (runnable in any host environment)
add_executable(test_ibus_engine
    tests/test_ibus_engine.c
    src/engine.c
    src/keymap.c
    src/apply.c
)

target_compile_definitions(test_ibus_engine PRIVATE TEXTVN_IBUS_MOCK=1)

target_include_directories(test_ibus_engine PRIVATE
    include
    src
    tests
    ${CMAKE_CURRENT_SOURCE_DIR}/../linux-common/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../../ffi/include
)

target_link_libraries(test_ibus_engine PRIVATE
    textvn-linux-common
    ${TEXTVN_FFI_LIB}
)

enable_testing()
add_test(NAME test_ibus_engine COMMAND test_ibus_engine)
