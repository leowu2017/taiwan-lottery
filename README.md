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

`download_all` now performs two steps:

1. Download all CSV files from FinancialPlanning OpenData into `data/`.
2. Parse each CSV and download all `http/https` links found in rows into a per-code directory.
3. If a downloaded file is a ZIP archive, extract it into its own subfolder under that code directory.

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
	Shortcut of `download_dataset(output_dir, "D423F")` and includes ZIP extraction.
- `download_all(output_dir)`:
	Downloads API docs and all datasets listed in the docs.

Examples:

- `cargo run --example download -- all`
- `cargo run --example download -- api-doc`
- `cargo run --example download -- dataset D416F`
- `cargo run --example download -- history-draw`