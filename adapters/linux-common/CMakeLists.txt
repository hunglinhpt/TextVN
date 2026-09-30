cmake_minimum_required(VERSION 3.16)
project(textvn-linux-common C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

set(SOURCES
    src/compose.c
    src/field_detect.c
    src/ipc_client.c
    src/log.c
    src/env.c
    src/utf.c
)

add_library(textvn-linux-common STATIC ${SOURCES})

# Version handshake/IBus component = version workspace Cargo (một nguồn sự thật,
# giống linux-settings). PUBLIC để ibus/fcitx5 link vào cũng thấy TEXTVN_VERSION.
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/../../Cargo.toml" _ver_line REGEX "^version = \"[^\"]+\"" LIMIT_COUNT 1)
string(REGEX REPLACE "^version = \"([^\"]+)\"" "\\1" TEXTVN_VERSION "${_ver_line}")
if(NOT TEXTVN_VERSION)
    message(FATAL_ERROR "Không đọc được version từ Cargo.toml")
endif()
target_compile_definitions(textvn-linux-common PUBLIC TEXTVN_VERSION="${TEXTVN_VERSION}")

target_include_directories(textvn-linux-common PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../../ffi/include>
    $<INSTALL_INTERFACE:include>
)

if(MSVC)
    target_compile_options(textvn-linux-common PRIVATE /W4)
else()
    target_compile_options(textvn-linux-common PRIVATE -Wall -Wextra -Werror -pedantic -fPIC)
endif()

# Engine Rust (staticlib) cho test chạy engine thật: `cargo build --release -p textvn-ffi`.
if(NOT TEXTVN_FFI_LIB)
    set(TEXTVN_FFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release" CACHE PATH "Rust target/release")
    find_library(TEXTVN_FFI_LIB NAMES libtextvn_ffi.a textvn_ffi PATHS ${TEXTVN_FFI_DIR} NO_DEFAULT_PATH)
endif()

if(TEXTVN_FFI_LIB AND NOT TEXTVN_LINUX_COMMON_NO_TESTS)
    enable_testing()
    foreach(t test_compose test_env test_ipc_client test_field_detect)
        add_executable(${t} tests/${t}.c)
        # Test phải giữ assert kể cả bản Release (NDEBUG).
        target_compile_options(${t} PRIVATE -Wall -Wextra -Werror -UNDEBUG)
        target_link_libraries(${t} PRIVATE textvn-linux-common ${TEXTVN_FFI_LIB} pthread dl m)
        add_test(NAME ${t} COMMAND ${t})
    endforeach()
endif()
