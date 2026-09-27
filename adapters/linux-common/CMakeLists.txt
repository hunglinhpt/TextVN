cmake_minimum_required(VERSION 3.16)
project(textvn-linux-common C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

set(SOURCES
    src/field_detect.c
    src/ipc_client.c
    src/log.c
    src/env.c
    src/utf.c
)

add_library(textvn-linux-common STATIC ${SOURCES})

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
