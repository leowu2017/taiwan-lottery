cmake_minimum_required(VERSION 3.16)
project(taiwan_lottery_c_examples C)

# Build Rust library first:
#   cargo build --release
# Then configure C examples:
#   cmake -S c -B c/build
#   cmake --build c/build --config Release

set(TAIWAN_LOTTERY_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../include" CACHE PATH "Path to taiwan_lottery headers")
set(TAIWAN_LOTTERY_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../target/release" CACHE PATH "Path to built Rust library directory")

find_library(TAIWAN_LOTTERY_LIB
    NAMES taiwan_lottery
    PATHS "${TAIWAN_LOTTERY_LIB_DIR}"
    NO_DEFAULT_PATH
)

if(NOT TAIWAN_LOTTERY_LIB)
    message(FATAL_ERROR "Cannot find taiwan_lottery library in ${TAIWAN_LOTTERY_LIB_DIR}. Run 'cargo build --release' first.")
endif()

add_executable(download examples/download.c)
target_include_directories(download PRIVATE "${TAIWAN_LOTTERY_INCLUDE_DIR}")
target_link_libraries(download PRIVATE "${TAIWAN_LOTTERY_LIB}")
