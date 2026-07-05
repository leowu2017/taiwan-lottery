# taiwan-lottery

## Overview

`taiwan-lottery` provides Taiwan lottery download, query, and random draw capabilities.

This repository exposes two public interfaces:

- A Rust API for the primary implementation and developer-facing usage.
- A C API for FFI consumers, examples, and SDK-style integration.

The project includes tests for both interfaces and they should stay aligned.

## Developer Guidance

This section is intended for both human maintainers and AI agents.

Guidelines:

- Treat the Rust API as the primary implementation surface.
- Treat the C API as a supported public interface, not a throwaway wrapper.
- Keep Rust and C data models, exported behavior, examples, and tests as consistent as practical.
- When a public behavior changes, update both Rust-side and C-side coverage.
- Do not consider API-affecting work complete if only the Rust side passes.

Recommended workflow:

1. Implement or update the Rust API.
2. Add or update Rust tests.
3. Add or update the C interface in `ffi.rs`, headers, and C examples/tests.
4. Add or update C tests.
5. Run both Rust and C validation.

## Pre-Release Review Checklist

This section is intended to be followed by both human maintainers and AI agents when asked to do a crate release readiness check.

### Review Goal

Before publishing a new crate version, verify that the codebase is consistent, public APIs are documented, tests match the current behavior, and obvious maintainability issues are addressed or explicitly left alone with a reason.

### Required Review Flow

1. Scan the diff or current repo state for public API changes.
2. Check whether Rust API, C API, examples, tests, and documentation still describe the same behavior.
3. Check whether recently added code introduced avoidable duplication or oversized logic that should be refactored before release.
4. Check whether tests are missing, redundant, misleading, or still pinned to outdated behavior.
5. Check whether README, C README, public headers, doc comments, and examples need updates.
6. Run the narrowest relevant validations first, then run the broader release validation commands.
7. Summarize release blockers, recommended fixes, and lower-priority cleanup separately.

### Code Review Checklist

- Check for duplicated parsing logic, duplicated mappings, duplicated constants, and repeated branching that should be centralized.
- Check for large functions that have grown beyond a reasonable single responsibility and should be split into helpers.
- Check whether refactoring would reduce maintenance cost without creating unnecessary churn right before release.
- Do not refactor just for style; prioritize duplicated logic, misleading names, and code that is hard to validate.
- If code is large but stable and a refactor would be high-risk, call it out as follow-up instead of forcing it into the release.

### Test Review Checklist

- Add tests for any new public API, new branch, bug fix, error mapping, or compatibility path.
- Update tests that still assert old names, old fields, or old behavior after an API cleanup.
- Remove or rewrite tests that only duplicate coverage without validating a distinct behavior.
- Prefer behavior-focused tests over implementation-detail tests.
- When Rust and C expose the same public behavior, check whether both sides have coverage where practical.
- For FFI changes, verify success paths, error paths, and ownership/freeing behavior.

### Documentation Review Checklist

- Update `README.md` when public Rust APIs, naming, return shapes, examples, or release expectations change.
- Update `c/README.md` and public headers when the C surface changes.
- Update doc comments on public Rust types and functions when names or semantics change.
- Check examples for stale imports, stale field names, stale function names, and outdated CLI behavior.
- Add migration notes when a change is additive-but-not-obvious or breaking.

### Comment Review Checklist

- Update comments when code behavior changes and the old comment is now misleading.
- Remove comments that merely restate obvious code.
- Add short comments only where a non-obvious rule, fallback path, ownership rule, or data-shape rule needs explanation.

### Validation Checklist

- Run `cargo test`.
- Run `cargo check --examples`.
- Run `cargo build --release` if the Rust library is part of the release artifact.
- Run the relevant C build/test commands when the C API is part of the supported release surface.
- If only one area changed, run the narrow validation for that area before broader validation.

### Release Validation Commands

Use this command sequence for a full pre-publish validation pass:

```powershell
cargo test
cargo check --examples
cargo build --release
cmake -S c -B c/build
cmake --build c/build --config Release
ctest --test-dir c/build --build-config Release --output-on-failure
```

If only Rust code changed and the C surface was not affected, the minimum validation set is:

```powershell
cargo test
cargo check --examples
cargo build --release
```

### Expected Release Review Output

When an agent performs a pre-release check, the result should separate findings into these categories:

- Release blockers: must fix before publishing.
- Should fix now: not strictly blocking, but high-value before release.
- Follow-up cleanup: safe to defer.
- Validation run: exactly what was executed and what could not be executed.

## Project Structure

- `src/`: Rust library implementation.
- `examples/`: Rust example CLIs.
- `include/taiwan_lottery/`: public C headers.
- `c/examples/`: C example CLIs.
- `c/tests/`: C tests.
- `data/`: downloaded and checked-in sample/history data used by local query flows.

## Build And Test

### Rust

- Build: `cargo build`
- Test: `cargo test`
- Release build for C consumers: `cargo build --release`

### C

Build the Rust release library first, then configure and build the C project:

- `cargo build --release`
- `cmake -S c -B c/build`
- `cmake --build c/build --config Release`

Run C tests from the repository root:

- `ctest --test-dir c/build --build-config Release --output-on-failure`

Run C tests from inside `c/build`:

- `ctest -C Release --output-on-failure`

`-C Release` is required in the `c/build` directory because the Visual Studio generator is multi-config.

## Public APIs

### Download APIs

- `download_api_doc(output_dir)`
  Downloads `financialplanning_api_docs.json` into `output_dir`.
- `download_dataset(output_dir, dataset_code)`
  Downloads one dataset CSV (for example `D416F.csv`), then downloads all links in that CSV and extracts ZIP files automatically.
- `download_history_draw(output_dir)`
  Downloads history draw data with a two-path strategy:
  1. try `D423F` via FinancialPlanning OpenData first.
  2. fallback to the Taiwan Lottery yearly ZIP API only when the primary path fails with an HTTP/network error.
- `download_history_draw_from_gov_data(output_dir)`
  Downloads history draw data only from FinancialPlanning OpenData (`D423F`).
- `download_history_draw_from_taiwan_lottery(output_dir)`
  Downloads history draw data only from the Taiwan Lottery yearly ZIP API.
- `download_all(output_dir)`
  Downloads API docs and all datasets listed in the docs.

### Query APIs

- `query_history_draw(output_dir, game, query)`
  Reads history draw data from files downloaded by `download_history_draw` under `output_dir/D423F/`.
  Primary numbers are exposed in `HistoryDrawItem.numbers.base.numbers`.
  When a sorted view is available, it is exposed in `HistoryDrawItem.numbers.sorted`.
- `query_history_draw_from_taiwan_lottery(game, query)`
  Calls the Taiwan Lottery web API directly.
  Draw-order numbers are exposed in `HistoryDrawItem.numbers.base.numbers`, and sorted numbers are exposed in `HistoryDrawItem.numbers.sorted` when available.

### Draw APIs

- `draw_by_game(game)`
  Performs a random draw for a specific game and returns `DrawResult`.
  Primary numbers are stored in `DrawResult.base.numbers`, and optional bonus data is stored in `DrawResult.bonus`.

### Game APIs

- `LotteryGame::ALL`
  Enumerates every supported game for UI lists and validation.
- `LotteryGame::metadata()`
  Returns UI-oriented static metadata including display name, number rule text, and number ranges.
- `LotteryGame::parse(value)`
  Parses CLI/user aliases such as `lotto649`, `5118`, or `tic-tac-toe`.
- `LotteryGame::from_code(code)`
  Maps the integer codes used by the C API and FFI layer back to `LotteryGame`.

Compatibility note:

- `HistoryGame`, `HistoryGameMetadata`, and `HistoryGameNumberRule` are deprecated compatibility aliases for the old naming and may be removed in a future release.

### Error Type

- `DownloadError`
  Implements `Display` and `std::error::Error`, so it can be printed directly and chained through standard Rust error handling.

## Rust Usage

Example local query for `Lotto649`:

```rust
use taiwan_lottery::{
  download_history_draw, query_history_draw, HistoryDrawQuery, LotteryGame,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    download_history_draw("data")?;

    let query = HistoryDrawQuery::by_month("2023-12");
    let page = query_history_draw("data", LotteryGame::Lotto649, query)?;

    for item in page.items {
        println!("period={}", item.period);
        println!("numbers={:?}", item.numbers.base.numbers);
        println!("numbers_sorted={:?}", item.numbers.sorted);
    }

    Ok(())
}
```

Example UI metadata lookup:

```rust
use taiwan_lottery::LotteryGame;

let metadata = LotteryGame::Lotto649.metadata();
assert_eq!(metadata.display_name, "大樂透");
assert_eq!(metadata.number_ranges[0].picks, 6);
```

Rust example commands:

- `cargo run --example download -- all`
- `cargo run --example download -- api-doc`
- `cargo run --example download -- dataset D416F`
- `cargo run --example download -- history-draw`
- `cargo run --example download -- history-draw-gov`
- `cargo run --example download -- history-draw-taiwan-lottery`
- `cargo run --example draw -- lotto649`
- `cargo run --example query -- local lotto649 period 115000001 data`
- `cargo run --example query -- remote lotto649 month 2026-01`

## C SDK

The C SDK layout is under `c/`.

Main entry header:

- `include/taiwan_lottery/taiwan_lottery.h`

Split headers by concern:

- `include/taiwan_lottery/download.h`
- `include/taiwan_lottery/draw.h`
- `include/taiwan_lottery/numbers.h`
- `include/taiwan_lottery/query.h`

C examples mirror the Rust example modes.

Source files:

- `c/examples/download.c`
- `c/examples/draw.c`
- `c/examples/query.c`

C example commands:

- `c/build/draw lotto649`
- `c/build/download all data`
- `c/build/download api-doc data`
- `c/build/download dataset D416F data`
- `c/build/download history-draw data`
- `c/build/download history-draw-gov data`
- `c/build/download history-draw-taiwan-lottery data`
- `c/build/query local lotto649 period 115000001 data`
- `c/build/query remote lotto649 month 2026-01`

## Data Models

Rust shared number models:

- `DrawNumbers`: base number container.
- `BonusDrawNumbers`: base numbers plus optional bonus.
- `SortedDrawNumbers`: base numbers plus optional sorted view.

C shared number models:

- `draw_numbers`: base number array.
- `bonus_draw_numbers`: base numbers plus optional bonus.
- `sorted_draw_numbers`: base numbers plus optional sorted view.

## Download Behavior

`download_all` performs these steps:

1. Download API docs JSON into `output_dir`.
2. Parse all dataset codes from the docs.
3. Download each dataset CSV from FinancialPlanning OpenData into `output_dir`.
4. Parse each CSV and download all `http/https` links found in rows into a per-code directory.
5. If a downloaded file is a ZIP archive, extract it into its own subfolder under that code directory.

`download_history_draw` behavior:

1. Primary path: download `D423F` from FinancialPlanning OpenData.
2. Fallback path: only when the primary path fails with an HTTP/network error, use the Taiwan Lottery API (`/Lottery/ResultDownload`) to download yearly ZIP files from 2007 onward until a year has no downloadable path.
3. Fallback ZIP files are saved under `output_dir/D423F/` and extracted automatically.

Example output layout:

- `data/D410F.csv`
- `data/D410F/<downloaded files from links in D410F.csv>`
- `data/D410F/<zip-file-stem>/<extracted files from that zip>`

## Data Source Notes

Taiwan lottery related open data is exposed from NTA's OpenAPI docs:

- API docs JSON:
  `https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning`

Mapping rule:

1. Read all entries under `paths` in the API docs JSON.
2. Each path looks like `/restful/D423F`.
3. Extract the code after `/restful/`.
4. Build the CSV download URL with `https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=<CODE>`.

Example:

- `/restful/D423F` -> `https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=D423F`

## Validation Result (2026-07-02)

Validation was run against all 20 codes found in the FinancialPlanning API docs `paths` section.

- HTTP accessibility: all 20 links returned status 200.
- Non-empty CSV content: 17 links.
- Empty CSV content: 3 links.

Empty CSV codes:

- `D409F`
- `D421F`
- `D422F`

The URL pattern is correct and downloadable, but not every code currently has non-empty CSV content.