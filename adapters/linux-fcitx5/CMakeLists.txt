cmake_minimum_required(VERSION 3.16)
project(fcitx5-textvn LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Optional Extra CMake Modules (ECM)
find_package(ECM 1.0.0 QUIET NO_MODULE)
if(ECM_FOUND)
    set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH} "${CMAKE_CURRENT_SOURCE_DIR}/cmake" ${CMAKE_MODULE_PATH})
endif()

# Fcitx5 Core
find_package(Fcitx5Core 5.0.14 REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(XKBCommon REQUIRED xkbcommon)

include(GNUInstallDirs)

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

# Fcitx5 addon module
add_library(textvn-fcitx5 MODULE
    src/addon.cpp
    src/engine.cpp
    src/keymap.cpp
    src/apply.cpp
)

target_include_directories(textvn-fcitx5 PRIVATE
    src
    ${CMAKE_CURRENT_SOURCE_DIR}/../linux-common/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../../ffi/include
    ${XKBCommon_INCLUDE_DIRS}
)

target_link_libraries(textvn-fcitx5 PRIVATE
    Fcitx5::Core
    textvn-linux-common
    ${TEXTVN_FFI_LIB}
    ${XKBCommon_LIBRARIES}
    pthread
    dl
)

set_target_properties(textvn-fcitx5 PROPERTIES
    OUTPUT_NAME "textvn-fcitx5"
)

# Install addon descriptors
if(DEFINED FCITX_INSTALL_ADDONDIR)
    install(TARGETS textvn-fcitx5 DESTINATION "${FCITX_INSTALL_ADDONDIR}")
else()
    install(TARGETS textvn-fcitx5 DESTINATION "${CMAKE_INSTALL_LIBDIR}/fcitx5")
endif()

configure_file(conf/textvn.conf.in conf/textvn.conf @ONLY)
configure_file(conf/textvn-addon.conf.in conf/textvn-im.conf @ONLY)

install(FILES "${CMAKE_CURRENT_BINARY_DIR}/conf/textvn.conf" DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/addon")
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/conf/textvn-im.conf" DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod" RENAME textvn.conf)
