# Data Consistency Tests

This document describes the data consistency tests between local (D423F dataset) and remote (Taiwan Lottery API) query sources.

## Overview

Seven new granular integration tests have been added to `tests/parity.rs` to verify that local and remote queries return consistent data with proper boundary validation:

### Discontinued Games (4 tests)
- `discontinued_game_boundary_before_start` - Verify (start_month - 1) returns no data
- `discontinued_game_start_month_matches` - Verify start_month data matches exactly
- `discontinued_game_end_month_matches` - Verify end_month data matches exactly  
- `discontinued_game_boundary_after_end` - Verify (end_month + 1) returns no data

### Active Games (3 tests)
- `active_game_boundary_before_start` - Verify (start_month - 1) returns no data
- `active_game_start_month_matches` - Verify start_month data matches exactly
- `active_game_two_months_ago_matches` - Verify 2-month-old data matches exactly

All tests are marked with `#[ignore]` since they require:
- Network access to Taiwan Lottery API
- Local D423F dataset in `data/` directory

## Test Design Rationale

### Boundary Tests (Before/After Start/End)

For both discontinued and active games:

**Before Start** 
- Expected: Both local and remote return **no data**
- Validates: Game data doesn't exist before documented start month

**After End** (discontinued games only)
- Expected: Both local and remote return **no data**
- Validates: Game data doesn't exist after documented end month

### Data Matching Tests

**Start Month**
- Expected: Both sources return data, results match **exactly**
- Validates: Game data begins at documented start month with consistent values

**End Month** (discontinued games only)
- Expected: Both sources return data, results match **exactly**
- Validates: Game data exists through documented end month with consistent values

**Two Months Ago** (active games only)
- Query month: Current UTC month - 2 months (dynamically calculated)
- Expected: Results match **exactly** between local and remote
- Rationale: Avoids latency issues with very recent draws where local data might not have caught up to remote

## Running the Tests

### List all parity tests:

```bash
cd tests && cargo test --test parity -- --list
```

### Run specific granular test:

```bash
cargo test --test parity discontinued_game_start_month_matches -- --nocapture --ignored
cargo test --test parity active_game_two_months_ago_matches -- --nocapture --ignored
```

### Run all granular parity tests:

```bash
cargo test --test parity -- --nocapture --ignored | grep -E "discontinued_game|active_game"
```

### Run all parity tests including the comprehensive check:

```bash
cargo test --test parity -- --nocapture --ignored
```

## Setup Requirements

Ensure `data/` directory contains downloaded D423F dataset:

```bash
# Download D423F dataset first
cargo run --example download -- history-draw data/

# Verify data directory exists
ls -la data/D423F/
```

## Test Data Coverage

### Discontinued Games

| Game | Start Month | End Month | Boundary Tests |
|------|-------------|-----------|-----------------|
| Lotto38M6 | 2007-07 | 2023-12 | Check 2006-06 (before), 2024-01 (after) |
| Lotto638 | 2007-07 | (varies) | Check 2006-06 (before) |
| TicTacToe | 2009-02 | 2013-12 | Check 2008-12 (before), 2014-01 (after) |
| Lotto1224 | 2018-04 | 2023-12 | Check 2018-03 (before), 2024-01 (after) |
| Lotto740 | 2015-06 | 2019-12 | Check 2015-04 (before), 2020-01 (after) |

### Active Games

| Game | Start Month | Current Status | Boundary Test |
|------|-------------|-----------------|-----------------|
| SuperLotto638 | 2008-01 | Still active | Check 2007-12 (before) |
| Lotto649 | 2007-01 | Still active | Check 2006-12 (before) |
| Daily539 | 2007-01 | Still active | Check 2006-12 (before) |
| Lotto3D | 2007-01 | Still active | Check 2006-12 (before) |
| Lotto4D | 2007-01 | Still active | Check 2006-12 (before) |
| Lotto49M6 | 2007-01 | Still active | Check 2006-12 (before) |
| Lotto39M5 | 2010-01 | Still active | Check 2009-12 (before) |
| BingoBingo | 2024-01 | Still active | Check 2023-12 (before) |

## Potential Failures and Solutions

### Boundary test fails: Unexpected data at out-of-range month

**Cause**: Date ranges may be incorrectly documented

**Debug**:
```bash
# Manually query the month that should be empty
curl https://api.taiwanlottery.com/TLCAPIWeB/Lottery/Lotto649Result \
  -G -d "month=2006-12" -d "endMonth=2006-12"

# Check local data
cargo run --example query -- data/ Lotto649 2006-12
```

**Fix**: Update date ranges in `GAME_MAPPING.md` if needed

### Start/End month test fails: Data mismatch

**Cause**: Inconsistency between local D423F dataset and remote API

**Debug**:
```bash
# Compare remote API response
curl https://api.taiwanlottery.com/TLCAPIWeB/Lottery/Lotto649Result \
  -G -d "month=2007-01" -d "endMonth=2007-01" | jq .

# Check local data
cargo run --example query -- data/ Lotto649 2007-01
```

**Fix**:
- Re-download latest data: `cargo run --example download -- history-draw data/`
- If issue persists, may indicate data source inconsistency requiring investigation

### Two months ago test fails for active game

**Cause**: Recent data synchronization lag or local data not updated

**Debug**:
```bash
# Check current month
date +%Y-%m

# Re-download latest data
cargo run --example download -- history-draw data/

# Query two months ago manually
MONTH=$(date -d "2 months ago" +%Y-%m)
curl https://api.taiwanlottery.com/TLCAPIWeB/Lottery/Lotto649Result \
  -G -d "month=${MONTH}" -d "endMonth=${MONTH}"
```

**Fix**: Re-download latest data to synchronize with remote API

### Network timeout

**Cause**: Taiwan Lottery API is slow or unreachable

**Debug**:
```bash
# Test API connectivity
curl -v https://api.taiwanlottery.com/TLCAPIWeB/Lottery/Lotto649Result \
  -G -d "month=2026-01" -d "endMonth=2026-01" --max-time 10
```

**Fix**: Check network connection or retry later

## Implementation Details

### Helper Functions (parity.rs)

- `compare_one_month()` - Compares local vs remote for single month, handles errors
- `canonicalize()` - Normalizes draw data for comparison (sorted vs base numbers)
- `execute_query()` - Wraps query execution, maps errors to QueryOutcome enum
- `shift_month()` - Calculates month offsets for dynamic date calculations
- `utc_current_yyyy_mm()` - Gets current UTC month for active game tests

### Test Outcome Types

- `QueryOutcome::Data` - Query returned results (period → sorted numbers mapping)
- `QueryOutcome::Empty` - Query returned 0 results
- `QueryOutcome::Rejected` - Query was rejected as out-of-range

## Future Improvements

- Add CI/CD pipeline with scheduled runs
- Implement data quality metrics dashboard
- Add performance benchmarks comparing query speeds
- Track historical data lag patterns
- Generate monthly parity reports
- Add snapshot-based regression testing
