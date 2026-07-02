# C SDK Layout

This directory contains C-facing usage assets for the Rust library:

- `examples/download.c`: C example CLI matching Rust download modes.
- `src/`: reserved for future C wrapper/helper code.
- `CMakeLists.txt`: build script for C examples.

## Query APIs (C)

The header `include/taiwan_lottery/data.h` now exposes:

- `get_history_draw(...)`: query from downloaded local history files (`output_dir/D423F`).
- `get_history_draw_from_taiwan_lottory(...)`: query directly from Taiwan Lottery web API.
- `free_history_draw_page(...)`: release memory allocated by the two query APIs.

Both query functions return a `taiwan_lottery_history_draw_page*` via output pointer.
The caller owns this memory and must call `free_history_draw_page(...)` when done.

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
- `c/build/download history-draw-gov data`
- `c/build/download history-draw-taiwan-lottery data`
