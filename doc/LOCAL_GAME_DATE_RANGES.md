# Taiwan Lottery Game Date Ranges (Local — D423F)

This document lists the earliest and latest draw dates available in the local D423F dataset for each known game.

Data sourced from: `D423F`

Retrieval date: 2026-07-16

---

## Directory Structure

The D423F dataset organizes files by year, but the internal layout changed over time:

| Year range | Path pattern | File naming pattern |
|---|---|---|
| 2007–2013 | `D423F/<YEAR>/<ROC_YEAR>/` | `<game>_<YEAR>.csv` |
| 2014–2017, 2019, 2021–2024 | `D423F/<YEAR>/<YEAR>/` | `<game>_<YEAR>.csv` |
| 2018, 2020 | `D423F/<YEAR>/<YEAR>/` | `<game>_<YEAR>01_<YEAR>12.csv` |
| 2025–2026 | `D423F/<YEAR>/` | `<game>_<YEAR>.csv` |

ROC year = Gregorian year − 1911. Example: 2007 → ROC 96, so path is `D423F/2007/96/`.

---

## CSV Format

Each CSV file has the following structure:

- Row 1: header — `遊戲名稱,期別,開獎日期,...`
  - Exception: `大樂透加開獎項` files use `活動名稱` as the first column header, and the value per row is the specific event name (e.g., `春節加碼加開獎項`), not a fixed game name.
- Rows 2+: data rows, one row per draw
- `開獎日期` column (col 3): draw date in `YYYY/MM/DD` format
- `期別` column (col 2): draw period in `<ROC_YEAR><6-digit draw number>` format

---

## Active Games

Games present in the 2026 year folder.

| Local CSV name | First Draw Date | Latest in D423F |
|---|---|---|
| `大樂透` | 2007/01/02 | 2026/06/30 |
| `威力彩` | 2008/01/24 | 2026/06/29 |
| `今彩539` | 2007/01/01 | 2026/06/30 |
| `3星彩` | 2007/01/01 | 2026/06/30 |
| `4星彩` | 2007/01/01 | 2026/06/30 |
| `49樂合彩` | 2007/01/02 | 2026/06/30 |
| `39樂合彩` | 2010/09/06 | 2026/06/30 |
| `賓果賓果` | 2008/04/30 | 2026/06/30 |
| `大樂透加開獎項` | 2011/02/01 | 2026/06/30 |

Notes:

- The `大樂透加開獎項` files cover special bonus draw events tied to the 大樂透 game. They are not a standalone regular game.
- The `賓果賓果` local data starts from 2008/04/30. The D423F dataset has significantly older Bingo Bingo records than what is available through the Taiwan Lottery remote API.
- The latest date in D423F is 2026/06/30 across most games, reflecting a snapshot roughly 2 weeks behind the retrieval date.

---

## Discontinued Games

Games absent from the 2026 year folder.

| Local CSV name | First Draw Date | Last Draw Date |
|---|---|---|
| `6/38樂透彩` | 2007/01/01 | 2008/01/21 |
| `38樂合彩` | 2007/01/01 | 2023/12/28 |
| `樂線九宮格` | 2009/07/27 | 2013/12/31 |
| `大福彩` | 2015/04/22 | 2019/04/27 |
| `雙贏彩` | 2018/04/23 | 2023/12/30 |

---

## How to Re-verify (User Instructions)

To re-run this verification using an AI agent with file access, paste the following prompt:

---

> Re-verify the first and last draw dates for each game listed in `taiwan-lottery/doc/LOCAL_GAME_DATE_RANGES.md` using the local D423F dataset.
>
> Follow the verification procedure in the "Verification Procedure" section of that file. For each game, confirm the first draw date and the latest available draw date from the local files, then update the table and retrieval date if anything has changed.

---

## Verification Procedure

### Finding the earliest year for a game

1. Start from the earliest year folder (`D423F/2007/`) and check whether a CSV file for that game exists in the year subfolder (see directory structure table above).
2. Move forward year by year until the CSV first appears. That year's file contains the first draw.

### Finding the first draw date

1. Navigate to the earliest year folder containing the game's CSV.
2. Open the CSV and read row 2 (first data row after the header).
3. The `開獎日期` column (col 3) is the first draw date.

Example path for 大樂透 in its first year:
```
D423F/2007/96/大樂透_2007.csv
```

### Finding the latest draw date

1. Navigate to the most recent year folder containing the game's CSV.
2. Open the CSV and read the last data row.
3. The `開獎日期` column (col 3) is the latest draw date.

Example path for 大樂透 latest data:
```
D423F/2026/大樂透_2026.csv
```

### Confirming a game is discontinued

1. Identify the last year folder that contains the game's CSV.
2. Confirm the next year folder has no CSV for that game.
3. In the last year's CSV, the last data row's `開獎日期` is the final draw date.

### Constructing the file path

Use the directory structure table at the top of this document to build the correct path for a given year:

- For 2007–2013: `D423F/<YEAR>/<YEAR - 1911>/<game>_<YEAR>.csv`
- For 2014–2017, 2019, 2021–2024: `D423F/<YEAR>/<YEAR>/<game>_<YEAR>.csv`
- For 2018, 2020: `D423F/<YEAR>/<YEAR>/<game>_<YEAR>01_<YEAR>12.csv`
- For 2025–2026: `D423F/<YEAR>/<game>_<YEAR>.csv`

---

## Scope Note

- The dates listed here reflect what is present in the local D423F snapshot at the retrieval date.
- The D423F dataset may lag behind the current date; the latest available draw date is not necessarily today.
