cmake_minimum_required(VERSION 3.16)
project(taiwan_lottery_c_examples C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

if(MSVC)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

enable_testing()

# Build Rust library first:
# cargo build --release
# Then configure C examples:
# cmake -S c -B c/build
# cmake --build c/build --config Release
set(TAIWAN_LOTTERY_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../include" CACHE PATH "Path to taiwan_lottery headers")
set(TAIWAN_LOTTERY_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../target/release" CACHE PATH "Path to built Rust library directory")

# System libraries needed when linking the Rust library (see `rustc --print native-static-libs`).
set(TAIWAN_LOTTERY_SYSTEM_LIBS "")

if(WIN32)
    list(APPEND TAIWAN_LOTTERY_SYSTEM_LIBS bcrypt advapi32 kernel32 ntdll userenv ws2_32 dbghelp)
elseif(APPLE)
    find_package(Threads REQUIRED)
    list(APPEND TAIWAN_LOTTERY_SYSTEM_LIBS Threads::Threads m "-framework Security" "-framework CoreFoundation")
else()
    find_package(Threads REQUIRED)
    list(APPEND TAIWAN_LOTTERY_SYSTEM_LIBS Threads::Threads ${CMAKE_DL_LIBS} m)
endif()

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
target_link_libraries(download PRIVATE "${TAIWAN_LOTTERY_LIB}" ${TAIWAN_LOTTERY_SYSTEM_LIBS})

add_executable(query examples/query.c)
target_include_directories(query PRIVATE "${TAIWAN_LOTTERY_INCLUDE_DIR}")
target_sources(query PRIVATE src/game_args.c)
target_link_libraries(query PRIVATE "${TAIWAN_LOTTERY_LIB}" ${TAIWAN_LOTTERY_SYSTEM_LIBS})

add_executable(draw examples/draw.c)
target_include_directories(draw PRIVATE "${TAIWAN_LOTTERY_INCLUDE_DIR}")
target_sources(draw PRIVATE src/game_args.c)
target_link_libraries(draw PRIVATE "${TAIWAN_LOTTERY_LIB}" ${TAIWAN_LOTTERY_SYSTEM_LIBS})

add_executable(numbers_test tests/numbers_test.c)
target_include_directories(numbers_test PRIVATE "${TAIWAN_LOTTERY_INCLUDE_DIR}")

add_executable(draw_test tests/draw_test.c)
target_include_directories(draw_test PRIVATE "${TAIWAN_LOTTERY_INCLUDE_DIR}")
target_link_libraries(draw_test PRIVATE "${TAIWAN_LOTTERY_LIB}" ${TAIWAN_LOTTERY_SYSTEM_LIBS})

add_executable(query_test tests/query_test.c)
target_include_directories(query_test PRIVATE "${TAIWAN_LOTTERY_INCLUDE_DIR}")
target_compile_definitions(query_test PRIVATE TEST_FIXTURE_DIR="${CMAKE_CURRENT_SOURCE_DIR}/tests/fixtures")
target_link_libraries(query_test PRIVATE "${TAIWAN_LOTTERY_LIB}" ${TAIWAN_LOTTERY_SYSTEM_LIBS})

add_test(NAME numbers_test COMMAND numbers_test)
add_test(NAME draw_test COMMAND draw_test)
add_test(NAME query_test COMMAND query_test)

if(WIN32)
    set(TAIWAN_LOTTERY_TEST_PATH "PATH=${TAIWAN_LOTTERY_LIB_DIR};$ENV{PATH}")
    set_tests_properties(draw_test PROPERTIES ENVIRONMENT "${TAIWAN_LOTTERY_TEST_PATH}")
    set_tests_properties(query_test PROPERTIES ENVIRONMENT "${TAIWAN_LOTTERY_TEST_PATH}")
endif()
