use std::ffi::CStr;
use std::os::raw::c_char;

use crate::{HistoryDrawQuery, LotteryDisplayLanguage, LotteryGame};

use super::ffi_status::DownloadStatus;

pub(crate) fn c_str_arg_to_string(
    ptr: *const c_char,
    null_status: i32,
    invalid_utf8_status: i32,
) -> Result<String, i32> {
    if ptr.is_null() {
        return Err(null_status);
    }

    // SAFETY: ptr is checked for null above and expected to point to a valid C string.
    let c_str = unsafe { CStr::from_ptr(ptr) };
    match c_str.to_str() {
        Ok(value) => Ok(value.to_string()),
        Err(_) => Err(invalid_utf8_status),
    }
}

fn optional_c_str_arg_to_string(
    ptr: *const c_char,
    invalid_utf8_status: i32,
) -> Result<Option<String>, i32> {
    if ptr.is_null() {
        return Ok(None);
    }

    // SAFETY: pointer is either null (handled above) or expected to point to a valid C string.
    let c_str = unsafe { CStr::from_ptr(ptr) };
    match c_str.to_str() {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Err(_) => Err(invalid_utf8_status),
    }
}

pub(crate) fn int_to_lottery_game(value: i32) -> Result<LotteryGame, i32> {
    LotteryGame::from_code(value).ok_or(DownloadStatus::InvalidGame as i32)
}

pub(crate) fn int_to_display_language(value: i32) -> Result<LotteryDisplayLanguage, i32> {
    match value {
        0 => Ok(LotteryDisplayLanguage::English),
        1 => Ok(LotteryDisplayLanguage::Chinese),
        _ => Err(DownloadStatus::InvalidLanguage as i32),
    }
}

pub(crate) fn build_history_draw_query(
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
    open_date: *const c_char,
) -> Result<HistoryDrawQuery, i32> {
    let period = optional_c_str_arg_to_string(period, DownloadStatus::InvalidQueryUtf8 as i32)?;
    let month = optional_c_str_arg_to_string(month, DownloadStatus::InvalidQueryUtf8 as i32)?;
    let end_month =
        optional_c_str_arg_to_string(end_month, DownloadStatus::InvalidQueryUtf8 as i32)?;
    let open_date =
        optional_c_str_arg_to_string(open_date, DownloadStatus::InvalidQueryUtf8 as i32)?;

    Ok(HistoryDrawQuery {
        period,
        month,
        end_month,
        open_date,
    })
}
