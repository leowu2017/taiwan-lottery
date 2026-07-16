# Taiwan Lottery Game Date Ranges

This document lists the earliest and latest draw dates available through the public Taiwan Lottery API for each known game.

Data sourced from:
`https://api.taiwanlottery.com/TLCAPIWeB/Lottery/<GameResultEndpoint>`

Retrieval date: 2026-07-16

---

## Active Games

Games confirmed to have draws available up to the retrieval date.

| Game | Endpoint | First Draw Date | Verified Latest Draw |
|---|---|---|---|
| Lotto 6/49 | `Lotto649Result` | 2007-01-02 | 2026-07-14 |
| Super Lotto 6/38 | `SuperLotto638Result` | 2008-01-24 | 2026-07-16 |
| Daily 539 | `Daily539Result` | 2007-01-01 | 2026-07-16 |
| 3D | `3DResult` | 2007-01-01 | 2026-07-16 |
| 4D | `4DResult` | 2007-01-01 | 2026-07-16 |
| 49M6 | `49M6Result` | 2007-01-02 | 2026-07-14 |
| 39M5 | `39M5Result` | 2010-09-06 | 2026-07-16 |
| Bingo Bingo | `BingoResult` | 2024-01-01 | 2026-07-16 |

Notes:

- The first draw dates above are the earliest period (draw #1 of the year) confirmed to return data from the API.
- The API returns no data for any of these games for dates prior to the listed first draw date.
- Bingo Bingo uses a different public query parameter: `openDate=<YYYY-MM-DD>`. The other games use `month` / `endMonth`.

---

## Discontinued Games

Games for which the API returns no data beyond the listed last draw date.

| Game | Endpoint | First Draw Date | Last Draw Date |
|---|---|---|---|
| Lotto 638 (old) | `Lotto638Result` | 2007-01-01 | 2008-01-21 |
| 38M6 | `38M6Result` | 2007-01-01 | 2023-12-28 |
| TicTacToe | `TicTacToeResult` | 2009-07-27 | 2013-12-31 |
| 1224 | `Lotto1224Result` | 2018-04-23 | 2023-12-30 |
| 740 | `Lotto740Result` | 2015-04-22 | 2019-04-27 |

Notes:

- The API returns no data before the listed first draw date, and no data after the listed last draw date.
- "Lotto 638 (old)" (`Lotto638Result`) is distinct from "Super Lotto 6/38" (`SuperLotto638Result`). Both endpoints are separate in the API.

---

## How to Re-verify (User Instructions)

To re-run this verification using an AI agent with web access, paste the following prompt:

---

> Re-verify the first and last draw dates for each Taiwan Lottery game listed in `taiwan-lottery/doc/GAME_DATE_RANGES.md` using the public API at `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/`.
>
> Follow the verification procedure in the "Verification Procedure" section of that file. For each game, confirm the first draw date and the latest available draw date, then update the table if anything has changed. Note the retrieval date when done.

---

## Verification Procedure

This section describes the exact method used to produce the date ranges above, so an agent can reproduce or update them.

### General (non-Bingo) games

**To find the first draw date:**

1. Compute the ROC year for the suspected start year (ROC year = Gregorian year − 1911). For example, 2007 → ROC 96.
2. Call: `GET .../Lottery/<Endpoint>?period=<ROC_YEAR>000001`
   - Example: `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/Daily539Result?period=96000001`
3. If `totalSize=1`, read `lotteryDate` from the result — that is the first draw date.
4. If `totalSize=0`, the game did not start in that ROC year. Try the next year (ROC year + 1 → Gregorian + 1).

**To find the latest available draw date:**

1. Call with a broad range from the known first year to the current month:
   - Example: `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/Lotto649Result?period=&month=2026-01&endMonth=2026-07&pageNum=1&pageSize=1`
2. The API returns results in descending order; the first result is the most recent draw. Read its `lotteryDate`.

**To confirm a game is discontinued (no data after a certain date):**

1. Call with `month` set to the year following the last known draw year:
   - Example for 38M6 (last known draw 2023-12-28): `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/38M6Result?period=&month=2024-01&endMonth=2024-12&pageNum=1&pageSize=1`
2. If `totalSize=0`, the game has no data in that year, confirming the previous year was the last.

### Bingo Bingo

Bingo Bingo uses a different public query parameter (`openDate` instead of `month`/`endMonth`).

**To confirm start date:**

1. Call with a date one day before the suspected start:
   - `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?openDate=2023-12-01&pageNum=1&pageSize=1`
   - Expect: `totalSize=null` and `bingoQueryResult=[]` (no data).
2. Call with the suspected start date:
   - `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?openDate=2024-01-01&pageNum=1&pageSize=1`
   - Expect: `totalSize > 0` (data present).

**To find the latest available draw date:**

1. Call with today's date:
   - `https://api.taiwanlottery.com/TLCAPIWeB/Lottery/BingoResult?openDate=<YYYY-MM-DD>&pageNum=1&pageSize=1`
2. If `totalSize > 0`, the game has draws on that date.

### Period format reference

For non-Bingo games, the `period` field follows this format:

```
<ROC_YEAR><6-digit draw number within that year>
```

Examples:
- `96000001` = ROC year 96 (2007), 1st draw of the year
- `115000057` = ROC year 115 (2026), 57th draw of the year

ROC year = Gregorian year − 1911.

---

## Scope Note

- The dates listed here are based on the earliest and latest draw records returned by the public API at the retrieval date.
- These are API-observable boundaries only. Actual real-world game history may differ if older records are not exposed through the API.
