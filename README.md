# taiwan-lottery

## Data Source Notes (FinancialPlanning)

Taiwan lottery related open data is exposed from NTA's OpenAPI docs:

- API docs JSON:
	https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning

The key mapping rule is:

1. Read all entries under `paths` in the API docs JSON.
2. Each path is like `/restful/D423F`.
3. Extract the code after `/restful/` (for example, `D423F`).
4. Build CSV download URL with:
	 `https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=<CODE>`

Example:

- `/restful/D423F`
	-> `https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=D423F`

## Validation Result (2026-07-02)

Validation was run against all 20 codes found in the FinancialPlanning API-docs `paths` section.

- HTTP accessibility: all 20 links returned status 200.
- Non-empty CSV content: 17 links.
- Empty CSV content (length = 0): 3 links.

Empty CSV codes:

- `D409F`
- `D421F`
- `D422F`

So the URL pattern is correct and downloadable, but not every code currently has non-empty CSV content.

## Download Behavior

`download_all` performs these steps:

1. Download API docs JSON into `output_dir`.
2. Parse all dataset codes from docs.
3. Download each dataset CSV from FinancialPlanning OpenData into `output_dir`.
4. Parse each CSV and download all `http/https` links found in rows into a per-code directory.
5. If a downloaded file is a ZIP archive, extract it into its own subfolder under that code directory.

`download_history_draw` behavior:

1. Primary path: download `D423F` from FinancialPlanning OpenData (same behavior as `download_dataset(output_dir, "D423F")`).
2. Fallback path: only when the primary path fails with an HTTP/network error, use Taiwan Lottery API (`/Lottery/ResultDownload`) to download yearly ZIP files from 2007 onward until a year has no downloadable path.
3. Fallback ZIP files are saved under `output_dir/D423F/` and extracted automatically.

Example output layout:

- `data/D410F.csv`
- `data/D410F/<downloaded files from links in D410F.csv>`
- `data/D410F/<zip-file-stem>/<extracted files from that zip>`

## Public APIs

- `download_api_doc(output_dir)`:
	Downloads `financialplanning_api_docs.json` into `output_dir`.
- `download_dataset(output_dir, dataset_code)`:
	Downloads one dataset CSV (for example `D416F.csv`), then downloads all links in that CSV and extracts ZIP files automatically.
- `download_history_draw(output_dir)`:
	Downloads history draw data with a two-path strategy:
	1) try `D423F` via FinancialPlanning OpenData first,
	2) fallback to Taiwan Lottery yearly ZIP API (from 2007 onward) only when the primary path returns an HTTP/network error.
- `download_history_draw_from_gov_data(output_dir)`:
	Downloads history draw data only from FinancialPlanning OpenData (`D423F`).
- `download_history_draw_from_taiwan_lottery(output_dir)`:
	Downloads history draw data only from Taiwan Lottery yearly ZIP API.

- `query_history_draw(output_dir, game, query)`:
	Reads history draw data from files that were downloaded by `download_history_draw` (under `output_dir/D423F/`).
	This source only guarantees sorted draw numbers (`numbers_sorted`).
	`numbers_draw` is returned as `null` to avoid misleading interpretation.
- `query_history_draw_from_taiwan_lottery(game, query)`:
	Calls Taiwan Lottery website API directly. This source can provide both:
	1) draw order (`numbers_draw`),
	2) sorted order (`numbers_sorted`).

- `download_all(output_dir)`:
	Downloads API docs and all datasets listed in the docs.

Example (`Lotto649`, local query from downloaded files):

```rust
use taiwan_lottery::{
	download_history_draw, query_history_draw, HistoryDrawQuery, HistoryGame,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	download_history_draw("data")?;

	let query = HistoryDrawQuery::by_month("2023-12");

	let page = query_history_draw("data", HistoryGame::Lotto649, query)?;
	for item in page.items {
		println!("period={}", item.period);
		println!("numbers_draw={:?}", item.numbers_draw); // local source is None
		println!("numbers_sorted={:?}", item.numbers_sorted);
	}

	Ok(())
}
```

Examples:

- `cargo run --example download -- all`
- `cargo run --example download -- api-doc`
- `cargo run --example download -- dataset D416F`
- `cargo run --example download -- history-draw`
- `cargo run --example download -- history-draw-gov`
- `cargo run --example download -- history-draw-taiwan-lottery`
- `cargo run --example query -- local lotto649 period 115000001 data`
- `cargo run --example query -- remote lotto649 month 2026-01`

## C Example

The C SDK layout is under `c/`.

The C example mirrors the same modes as the Rust example:

- `all [output_dir]`
- `api-doc [output_dir]`
- `dataset <DATASET_CODE> [output_dir]`
- `history-draw [output_dir]`
- `history-draw-gov [output_dir]`
- `history-draw-taiwan-lottery [output_dir]`

Source file:

- `c/examples/download.c`
- `c/examples/query.c`

Example build (CMake):

- `cargo build --release`
- `cmake -S c -B c/build`
- `cmake --build c/build --config Release`

Example run:

- `c/build/download all data`
- `c/build/download api-doc data`
- `c/build/download dataset D416F data`
- `c/build/download history-draw data`
- `c/build/download history-draw-gov data`
- `c/build/download history-draw-taiwan-lottery data`
- `c/build/query local lotto649 period 115000001 data`
- `c/build/query remote lotto649 month 2026-01`