cmake_minimum_required (VERSION 2.8.5)
cmake_policy (SET CMP0054 NEW)
project (rust-sponge)

#include (etc/build_defs.cmake)
#include (etc/build_type.cmake)
#include (etc/cflags.cmake)

add_subdirectory ("${PROJECT_SOURCE_DIR}/apps")
include (etc/tests.cmake)
