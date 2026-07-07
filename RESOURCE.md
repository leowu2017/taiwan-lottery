# Taiwan Lottery Resources

This document lists external public resources related to Taiwan Lottery data.

It is intentionally limited to public pages, public documentation entry points, and externally visible URL formats. Project-specific behavior and implementation details belong in `README.md`.

---

## Resource 1: Government Open Data Dataset Page

Public page:

- Taiwan Public Welfare Lottery winning numbers and prize information  
  https://data.gov.tw/dataset/72921

Public URL formats:

- API docs JSON  
  `https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning`
- CSV download pattern  
  `https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=<CODE>`

Examples:

- `https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=D423F`

Notes:

- This is the government open data entry page.
- It is the public-facing source that points to the FinancialPlanning open-data documents and dataset codes.

---

## Resource 2: Taiwan Lottery Historical Result Download Page

Public page:

- Taiwan Lottery historical result download page  
  https://www.taiwanlottery.com/lotto/history/result_download

Public URL format:

- Year-based download metadata API  
  `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/ResultDownload?year=<YYYY>`

Examples:

- `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/ResultDownload?year=2026`

Notes:

- This is the public page for downloadable historical result archives.
- The API form exposes year-based download metadata.

---

## Resource 3: Taiwan Lottery Historical Winning Numbers and Results

Public page:

- Taiwan Lottery historical winning numbers and results  
  https://www.taiwanlottery.com/lotto/history/history_result

Public URL format:

- General history/result API pattern for most non-Bingo games  
  `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/<GameResultEndpoint>?period=<PERIOD>&month=<YYYY-MM>&endMonth=<YYYY-MM>&pageNum=<N>&pageSize=<N>`

Examples:

- `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/Lotto649Result?period=&month=2026-01&endMonth=2026-01&pageNum=1&pageSize=20`
- `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/SuperLotto638Result?period=&month=2026-01&endMonth=2026-01&pageNum=1&pageSize=20`
- `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/3DResult?period=&month=2026-01&endMonth=2026-01&pageNum=1&pageSize=20`

Related public endpoints:

- `/Lottery/3DHistoryResult`
- `/Lottery/4DHistoryResult`

Notes:

- This page is the public history query entry point.
- Most games visible from this page follow the same `period/month/endMonth` query shape.

---

## Resource 4: Taiwan Lottery Winning Numbers and Results

Public page:

- Taiwan Lottery winning numbers and results (Super Lotto 638 example page)  
  https://www.taiwanlottery.com/lotto/result/super_lotto638

Public URL format:

- General result API pattern for most non-Bingo games  
  `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/<GameResultEndpoint>?period=<PERIOD>&month=<YYYY-MM>&endMonth=<YYYY-MM>&pageNum=<N>&pageSize=<N>`

Examples:

- Page: `https://www.taiwanlottery.com/lotto/result/super_lotto638`
- API: `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/SuperLotto638Result?period=&month=2026-01&endMonth=2026-01&pageNum=1&pageSize=20`

Notes:

- These are public per-game result pages.
- They are useful as human-facing reference points for the corresponding result APIs.

---

## Resource 5: Taiwan Lottery Winning Numbers and Results: Bingo Bingo

Public page:

- Taiwan Lottery winning numbers and results (Bingo Bingo)  
  https://www.taiwanlottery.com/lotto/result/bingo_bingo

Public URL format:

- Bingo-specific API pattern  
  `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?openDate=<YYYY-MM-DD>&pageNum=<N>&pageSize=<N>`

Examples:

- `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?openDate=2026-07-07&pageNum=1&pageSize=20`
- `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?openDate=2026-06-01&pageNum=1&pageSize=20`

Special note:

- Bingo Bingo uses a different public query shape from most other games.
- Most other games use `period`, `month`, and `endMonth`.
- Bingo Bingo uses `openDate`.

Public format that may look similar to other games but should be treated carefully:

- `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?period=&month=2026-06&endMonth=2026-06&pageNum=1&pageSize=20`

---

## Empirical Validation (Public API Calls)

Validation date:

- 2026-07-08

Validation scope:

- Called the public URLs directly via HTTPS (no project-internal logic used).
- Every game endpoint was tested with all four query shapes:
  - `month`
  - `month + endMonth`
  - `openDate`
  - `period`
- For each game, sampled `month`/`period` values were selected from remote API windows that returned data in live calls.

### Per-Game Remote Validation Windows

| Game | Validation month window used for remote calls |
|---|---|
| SuperLotto638 | 2007-01 to current UTC month (capped at 2033-12) |
| Lotto649 | 2007-01 to current UTC month (capped at 2033-12) |
| Daily539 | 2007-01 to current UTC month (capped at 2033-12) |
| 3D | 2007-01 to current UTC month (capped at 2033-12) |
| 4D | 2007-01 to current UTC month (capped at 2033-12) |
| 49M6 | 2007-01 to current UTC month (capped at 2033-12) |
| 39M5 | 2007-01 to current UTC month (capped at 2033-12) |
| 38M6 | 2007-01 to 2023-12 |
| 1224 | 2014-01 to 2023-12 |
| 740 | 2014-01 to 2023-12 |
| TicTacToe | 2007-01 to 2013-12 |
| 638 | 2007-01 to 2013-12 |
| BingoBingo | 2024-01 to current UTC month (capped at 2033-12) |

### All-Game Matrix (Same 4 Query Shapes)

Rule used in this matrix:

- `OK` means sampled request returned non-empty result data.
- `FAIL` means sampled request returned empty result data, even when `HTTP 200` and `rtCode=0`.

Sample selection rule:

- For each game, use a month inside the validation window that returned non-empty data to derive sampled `month` and `period` values.
- For `openDate`, use the sampled draw date from that game when available.

| Game | Endpoint | month | month+endMonth | openDate | period |
|---|---|---|---|---|---|
| SuperLotto638 | `SuperLotto638Result` | OK | OK | FAIL | OK |
| Lotto649 | `Lotto649Result` | OK | OK | FAIL | OK |
| Daily539 | `Daily539Result` | OK | OK | FAIL | OK |
| 3D | `3DResult` | OK | OK | FAIL | OK |
| 4D | `4DResult` | OK | OK | FAIL | OK |
| 49M6 | `49M6Result` | OK | OK | FAIL | OK |
| 39M5 | `39M5Result` | OK | OK | FAIL | OK |
| 38M6 | `38M6Result` | OK | OK | FAIL | OK |
| 1224 | `Lotto1224Result` | OK | OK | FAIL | OK |
| 740 | `Lotto740Result` | OK | OK | FAIL | OK |
| TicTacToe | `TicTacToeResult` | OK | OK | FAIL | OK |
| 638 | `Lotto638Result` | OK | OK | FAIL | OK |
| BingoBingo | `BingoResult` | FAIL | FAIL | OK | FAIL |

### Summary (Bingo vs Non-Bingo)

Non-Bingo summary:

- Using in-range sampled months/periods, all non-Bingo endpoints returned data for `month`, `month+endMonth`, and `period`.
- In sampled checks, `openDate` did not produce data for non-Bingo endpoints.

Bingo summary:

- `BingoResult` with `openDate` returned sampled data (`OK`).
- `BingoResult` with `month`, `month+endMonth`, or `period` returned empty sampled data (`FAIL`).

---

## Summary Table

| Resource | Public page or API | URL / format | Notes |
|---|---|---|---|
| Government dataset page | data.gov.tw | `https://data.gov.tw/dataset/72921` | Human-facing dataset page |
| Government API docs | NTA OpenAPI | `https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning` | Lists `/restful/<CODE>` entries |
| Government CSV download | OpenData CSV | `https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=<CODE>` | Direct CSV pattern |
| Taiwan Lottery download page | history result download page | `https://www.taiwanlottery.com/lotto/history/result_download` | Human-facing archive page |
| Taiwan Lottery yearly metadata | ResultDownload API | `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/ResultDownload?year=<YYYY>` | Yearly download info |
| Taiwan Lottery history page | history result page | `https://www.taiwanlottery.com/lotto/history/history_result` | Human-facing history entry |
| General game API | Result API | `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/<GameResultEndpoint>?period=<PERIOD>&month=<YYYY-MM>&endMonth=<YYYY-MM>&pageNum=<N>&pageSize=<N>` | Works for most games |
| Bingo-specific API | BingoResult API | `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?openDate=<YYYY-MM-DD>&pageNum=<N>&pageSize=<N>` | Different public query shape |

---

## Scope Note

- This file is intentionally limited to external public resources and public URL shapes.
- Internal implementation choices, validation rules, fallback behavior, and project-specific handling should stay in `README.md`.
