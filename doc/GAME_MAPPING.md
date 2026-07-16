# Taiwan Lottery Game Mapping

This document maps official game names (Chinese and English) to their identifiers in the remote Taiwan Lottery API and the local D423F dataset, and compares the queryable date ranges between the two sources.

---

## Active Games

Games currently offered as of the document date.

| 中文名稱 | English Name | Remote API Endpoint | D423F CSV Name | Remote: First Draw | Remote: Latest | D423F: First Draw | D423F: Latest |
|---|---|---|---|---|---|---|---|
| 大樂透 | Lotto 6/49 | `Lotto649Result` | `大樂透` | 2007-01-02 | active | 2007-01-02 | active |
| 威力彩 | Super Lotto 6/38 | `SuperLotto638Result` | `威力彩` | 2008-01-24 | active | 2008-01-24 | active |
| 今彩539 | Daily Cash 539 | `Daily539Result` | `今彩539` | 2007-01-01 | active | 2007-01-01 | active |
| 3星彩 | 3D | `3DResult` | `3星彩` | 2007-01-01 | active | 2007-01-01 | active |
| 4星彩 | 4D | `4DResult` | `4星彩` | 2007-01-01 | active | 2007-01-01 | active |
| 49樂合彩 | 49 M6 | `49M6Result` | `49樂合彩` | 2007-01-02 | active | 2007-01-02 | active |
| 39樂合彩 | 39 M5 | `39M5Result` | `39樂合彩` | 2010-09-06 | active | 2010-09-06 | active |
| 賓果賓果 | Bingo Bingo | `BingoResult` | `賓果賓果` | 2024-01-01 | active | 2008-04-30 | active |
| 大樂透加開獎項 | Lotto 6/49 Special Draws | *(none)* | `大樂透加開獎項` | — | — | 2011-02-01 | active |

---

## Discontinued Games

Games no longer offered, but still present in one or both sources.

| 中文名稱 | English Name | Remote API Endpoint | D423F CSV Name | Remote: First Draw | Remote: Last Draw | D423F: First Draw | D423F: Last Draw |
|---|---|---|---|---|---|---|---|
| 6/38樂透彩 | Lotto 6/38 | `Lotto638Result` | `6_38樂透彩` | 2007-01-01 | 2008-01-21 | 2007-01-01 | 2008-01-21 |
| 38樂合彩 | 38 M6 | `38M6Result` | `38樂合彩` | 2007-01-01 | 2023-12-28 | 2007-01-01 | 2023-12-28 |
| 樂線九宮格 | TicTacToe | `TicTacToeResult` | `樂線九宮格` | 2009-07-27 | 2013-12-31 | 2009-07-27 | 2013-12-31 |
| 大福彩 | Lotto 7/40 | `Lotto740Result` | `大福彩` | 2015-04-22 | 2019-04-27 | 2015-04-22 | 2019-04-27 |
| 雙贏彩 | Lotto 12/24 | `Lotto1224Result` | `雙贏彩` | 2018-04-23 | 2023-12-30 | 2018-04-23 | 2023-12-30 |

---

## Coverage Differences

| Game | Difference |
|---|---|
| 賓果賓果 | D423F has data from 2008-04-30. Remote API only exposes data from 2024-01-01. D423F has ~16 years of additional history not available through the remote API. |
| 大樂透加開獎項 | D423F only. No corresponding remote API endpoint. These are special bonus draw events, not a regular standalone game. |
| All active games | D423F is a snapshot that may lag behind the current date. Remote API always reflects current draws. The D423F snapshot used for this document extends to around 2026/06/30. |

---

## Notes

- **Date format**: `YYYY-MM-DD` (ISO 8601) throughout this document. Note that D423F CSV files store dates as `YYYY/MM/DD` internally.
- **D423F CSV name** refers to the value in the `遊戲名稱` column (first column) of the CSV file, which is also used as the filename prefix.
  - Exception: `大樂透加開獎項` files use `活動名稱` as the first column header, and each row records the specific event name (e.g., `春節加碼加開獎項`) rather than a fixed game name.
- **Remote "active"** means the game has draws continuing up to and including the document date (2026-07-16).

---

## Data Source Reference

| Source | Description |
|---|---|
| Remote API | `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/<Endpoint>` |
| D423F | Government open data dataset — public welfare lottery winning numbers and prize information |
