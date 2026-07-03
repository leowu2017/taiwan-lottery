# C SDK Layout

This directory contains C-facing usage assets for the Rust library:

- `examples/download.c`: C example CLI matching Rust download modes.
- `src/`: reserved for future C wrapper/helper code.
- `CMakeLists.txt`: build script for C examples.

## Query APIs (C)

The main entry header is:

- `include/taiwan_lottery/taiwan_lottery.h`

It exposes:

- `query_history_draw(...)`: query from downloaded local history files (`output_dir/D423F`).
- `query_history_draw_from_taiwan_lottery(...)`: query directly from Taiwan Lottery web API.
- `free_history_draw_page(...)`: release memory allocated by the two query APIs.

Prefer split headers by concern:

- `include/taiwan_lottery/download.h`
- `include/taiwan_lottery/query.h`

Both query functions return a `taiwan_lottery_history_draw_page*` via output pointer.
The caller owns this memory and must call `free_history_draw_page(...)` when done.

Each `taiwan_lottery_history_draw_item` exposes:

- `numbers_sorted`: numbers in sorted order
- `numbers_draw`: numbers in draw order when available from the remote API

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
- `c/build/query local lotto649 period 115000001 data`
- `c/build/query remote lotto649 month 2026-01`
