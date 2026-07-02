# C SDK Layout

This directory contains C-facing usage assets for the Rust library:

- `examples/download.c`: C example CLI matching Rust download modes.
- `src/`: reserved for future C wrapper/helper code.
- `CMakeLists.txt`: build script for C examples.

## Build

1. Build the Rust library:
   - `cargo build --release`
2. Build C example with CMake:
   - `cmake -S c -B c/build`
   - `cmake --build c/build --config Release`

## Run

From repository root:

- `c/build/download all data`
- `c/build/download api-doc data`
- `c/build/download dataset D416F data`
- `c/build/download history-draw data`
