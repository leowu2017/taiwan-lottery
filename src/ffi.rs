use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::{
    download_all, download_api_doc, download_dataset, download_history_draw,
    download_history_draw_from_gov_data, download_history_draw_from_taiwan_lottery, draw_by_game,
    query_history_draw, query_history_draw_from_taiwan_lottery, DownloadError, HistoryDrawQuery,
    LotteryGame,
};

#[repr(i32)]
enum DownloadStatus {
    Success = 0,
    NullPath = 1,
    InvalidPathUtf8 = 2,
    Io = 3,
    Network = 4,
    Parse = 5,
    NullDatasetCode = 6,
    InvalidDatasetCodeUtf8 = 7,
    InvalidGame = 8,
    InvalidQueryUtf8 = 9,
    NullResultPointer = 10,
}

#[repr(C)]
pub struct DrawNumbersC {
    numbers: *mut i32,
    numbers_len: usize,
}

#[repr(C)]
pub struct BonusDrawNumbersC {
    base: DrawNumbersC,
    has_bonus: i32,
    bonus: i32,
}

#[repr(C)]
pub struct SortedDrawNumbersC {
    base: DrawNumbersC,
    sorted_numbers: *mut i32,
    sorted_numbers_len: usize,
}

#[repr(C)]
pub struct HistoryDrawItemC {
    period: *mut c_char,
    date: *mut c_char,
    redeemable_date: *mut c_char,
    numbers: SortedDrawNumbersC,
}

#[repr(C)]
pub struct HistoryDrawPageC {
    total_size: usize,
    item_count: usize,
    items: *mut HistoryDrawItemC,
}

#[repr(C)]
pub struct LotteryGameQueryRangeC {
    min_month: *mut c_char,
    max_month: *mut c_char,
}

pub type DrawResultC = BonusDrawNumbersC;

#[unsafe(export_name = "download_api_doc")]
pub extern "C" fn download_api_doc_ffi(output_dir: *const c_char) -> i32 {
    let out_dir = match c_str_arg_to_string(
        output_dir,
        DownloadStatus::NullPath as i32,
        DownloadStatus::InvalidPathUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    map_download_result(download_api_doc(out_dir))
}

#[unsafe(export_name = "download_dataset")]
pub extern "C" fn download_dataset_ffi(
    output_dir: *const c_char,
    dataset_code: *const c_char,
) -> i32 {
    let out_dir = match c_str_arg_to_string(
        output_dir,
        DownloadStatus::NullPath as i32,
        DownloadStatus::InvalidPathUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let dataset_code = match c_str_arg_to_string(
        dataset_code,
        DownloadStatus::NullDatasetCode as i32,
        DownloadStatus::InvalidDatasetCodeUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    map_download_result(download_dataset(out_dir, &dataset_code))
}

#[unsafe(export_name = "download_history_draw")]
pub extern "C" fn download_history_draw_ffi(output_dir: *const c_char) -> i32 {
    let out_dir = match c_str_arg_to_string(
        output_dir,
        DownloadStatus::NullPath as i32,
        DownloadStatus::InvalidPathUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    map_download_result(download_history_draw(out_dir))
}

#[unsafe(export_name = "download_history_draw_from_gov_data")]
pub extern "C" fn download_history_draw_from_gov_data_ffi(output_dir: *const c_char) -> i32 {
    let out_dir = match c_str_arg_to_string(
        output_dir,
        DownloadStatus::NullPath as i32,
        DownloadStatus::InvalidPathUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    map_download_result(download_history_draw_from_gov_data(out_dir))
}

#[unsafe(export_name = "download_history_draw_from_taiwan_lottery")]
pub extern "C" fn download_history_draw_from_taiwan_lottery_ffi(output_dir: *const c_char) -> i32 {
    let out_dir = match c_str_arg_to_string(
        output_dir,
        DownloadStatus::NullPath as i32,
        DownloadStatus::InvalidPathUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    map_download_result(download_history_draw_from_taiwan_lottery(out_dir))
}

#[unsafe(export_name = "download_all")]
pub extern "C" fn download_all_ffi(output_dir: *const c_char) -> i32 {
    let out_dir = match c_str_arg_to_string(
        output_dir,
        DownloadStatus::NullPath as i32,
        DownloadStatus::InvalidPathUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    map_download_result(download_all(out_dir))
}

#[unsafe(export_name = "query_history_draw")]
pub extern "C" fn query_history_draw_ffi(
    output_dir: *const c_char,
    game: i32,
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
    out_page: *mut *mut HistoryDrawPageC,
) -> i32 {
    let out_dir = match c_str_arg_to_string(
        output_dir,
        DownloadStatus::NullPath as i32,
        DownloadStatus::InvalidPathUtf8 as i32,
    ) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let query = match build_history_draw_query(period, month, end_month) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let result = query_history_draw(out_dir, game, query);
    map_history_result_to_struct_status(result, out_page)
}

#[unsafe(export_name = "query_history_draw_from_taiwan_lottery")]
pub extern "C" fn query_history_draw_from_taiwan_lottery_ffi(
    game: i32,
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
    out_page: *mut *mut HistoryDrawPageC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let query = match build_history_draw_query(period, month, end_month) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let result = query_history_draw_from_taiwan_lottery(game, query);
    map_history_result_to_struct_status(result, out_page)
}

#[unsafe(export_name = "lottery_game_query_month_range")]
pub extern "C" fn lottery_game_query_month_range_ffi(
    game: i32,
    out_range: *mut *mut LotteryGameQueryRangeC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_range.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let range = game.query_month_range();
    let c_range = lottery_game_query_range_to_c(range);
    unsafe {
        *out_range = Box::into_raw(c_range);
    }

    DownloadStatus::Success as i32
}

#[unsafe(export_name = "draw_by_game")]
pub extern "C" fn draw_by_game_ffi(game: i32, out_result: *mut *mut DrawResultC) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_result.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let result = draw_by_game(game);
    let c_result = draw_result_to_c(result);

    unsafe {
        *out_result = Box::into_raw(c_result);
    }

    DownloadStatus::Success as i32
}

#[unsafe(export_name = "free_draw_result")]
pub extern "C" fn free_draw_result_ffi(result: *mut DrawResultC) {
    if result.is_null() {
        return;
    }

    let result = unsafe { Box::from_raw(result) };
    free_draw_numbers(&result.base);
}

#[unsafe(export_name = "free_history_draw_page")]
pub extern "C" fn free_history_draw_page_ffi(page: *mut HistoryDrawPageC) {
    if page.is_null() {
        return;
    }

    // SAFETY: page is allocated by this crate and must be returned via this function.
    let page = unsafe { Box::from_raw(page) };

    if !page.items.is_null() {
        // SAFETY: items points to an allocation created by Box<[HistoryDrawItemC]> in this crate.
        let items_box = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                page.items,
                page.item_count,
            ))
        };

        for item in &*items_box {
            free_history_draw_item(item);
        }
    }
}

#[unsafe(export_name = "free_lottery_game_query_month_range")]
pub extern "C" fn free_lottery_game_query_month_range_ffi(range: *mut LotteryGameQueryRangeC) {
    if range.is_null() {
        return;
    }

    let range = unsafe { Box::from_raw(range) };

    if !range.min_month.is_null() {
        let _ = unsafe { CString::from_raw(range.min_month) };
    }
    if !range.max_month.is_null() {
        let _ = unsafe { CString::from_raw(range.max_month) };
    }
}

fn c_str_arg_to_string(
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

fn int_to_lottery_game(value: i32) -> Result<LotteryGame, i32> {
    LotteryGame::from_code(value).ok_or(DownloadStatus::InvalidGame as i32)
}

fn build_history_draw_query(
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
) -> Result<HistoryDrawQuery, i32> {
    let period = optional_c_str_arg_to_string(period, DownloadStatus::InvalidQueryUtf8 as i32)?;
    let month = optional_c_str_arg_to_string(month, DownloadStatus::InvalidQueryUtf8 as i32)?;
    let end_month =
        optional_c_str_arg_to_string(end_month, DownloadStatus::InvalidQueryUtf8 as i32)?;

    Ok(HistoryDrawQuery {
        period,
        month,
        end_month,
    })
}

fn map_history_result_to_struct_status(
    result: Result<crate::HistoryDrawPage, DownloadError>,
    out_page: *mut *mut HistoryDrawPageC,
) -> i32 {
    let page = match result {
        Ok(value) => value,
        Err(err) => return map_download_result::<()>(Err(err)),
    };

    if out_page.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let c_page = history_page_to_c(page);
    // SAFETY: out_page is validated non-null above and points to writable caller memory.
    unsafe {
        *out_page = Box::into_raw(c_page);
    }

    DownloadStatus::Success as i32
}

fn history_page_to_c(page: crate::HistoryDrawPage) -> Box<HistoryDrawPageC> {
    let mut items = Vec::with_capacity(page.items.len());
    for item in page.items {
        items.push(history_item_to_c(item));
    }

    let item_count = items.len();
    let boxed_items = items.into_boxed_slice();
    let items_ptr = Box::into_raw(boxed_items) as *mut HistoryDrawItemC;

    Box::new(HistoryDrawPageC {
        total_size: page.total_size,
        item_count,
        items: items_ptr,
    })
}

fn lottery_game_query_range_to_c(
    range: crate::LotteryGameQueryRange,
) -> Box<LotteryGameQueryRangeC> {
    Box::new(LotteryGameQueryRangeC {
        min_month: string_to_c_ptr(range.min_month),
        max_month: string_to_c_ptr(range.max_month),
    })
}

fn draw_result_to_c(result: crate::DrawResult) -> Box<DrawResultC> {
    Box::new(DrawResultC {
        base: draw_numbers_to_c(result.base),
        has_bonus: i32::from(result.bonus.is_some()),
        bonus: result.bonus.unwrap_or_default(),
    })
}

fn history_item_to_c(item: crate::HistoryDrawItem) -> HistoryDrawItemC {
    HistoryDrawItemC {
        period: string_to_c_ptr(item.period),
        date: optional_string_to_c_ptr(item.date),
        redeemable_date: optional_string_to_c_ptr(item.redeemable_date),
        numbers: sorted_draw_numbers_to_c(item.numbers),
    }
}

fn draw_numbers_to_c(numbers: crate::DrawNumbers) -> DrawNumbersC {
    let numbers_len = numbers.numbers.len();
    let numbers_ptr = if numbers_len == 0 {
        std::ptr::null_mut()
    } else {
        Box::into_raw(numbers.numbers.into_boxed_slice()) as *mut i32
    };

    DrawNumbersC {
        numbers: numbers_ptr,
        numbers_len,
    }
}

fn sorted_draw_numbers_to_c(numbers: crate::SortedDrawNumbers) -> SortedDrawNumbersC {
    let (sorted_numbers, sorted_numbers_len) = match numbers.sorted {
        Some(sorted_numbers) if !sorted_numbers.is_empty() => {
            let len = sorted_numbers.len();
            let ptr = Box::into_raw(sorted_numbers.into_boxed_slice()) as *mut i32;
            (ptr, len)
        }
        _ => (std::ptr::null_mut(), 0),
    };

    SortedDrawNumbersC {
        base: draw_numbers_to_c(numbers.base),
        sorted_numbers,
        sorted_numbers_len,
    }
}

fn free_history_draw_item(item: &HistoryDrawItemC) {
    if !item.period.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.period) };
    }
    if !item.date.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.date) };
    }
    if !item.redeemable_date.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.redeemable_date) };
    }

    free_sorted_draw_numbers(&item.numbers);
}

fn free_draw_numbers(numbers: &DrawNumbersC) {
    if !numbers.numbers.is_null() {
        let _ = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                numbers.numbers,
                numbers.numbers_len,
            ))
        };
    }
}

fn free_sorted_draw_numbers(numbers: &SortedDrawNumbersC) {
    free_draw_numbers(&numbers.base);

    if !numbers.sorted_numbers.is_null() {
        let _ = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                numbers.sorted_numbers,
                numbers.sorted_numbers_len,
            ))
        };
    }
}

fn string_to_c_ptr(value: String) -> *mut c_char {
    let sanitized = value.replace('\0', "");
    CString::new(sanitized)
        .expect("string sanitized to avoid interior nul")
        .into_raw()
}

fn optional_string_to_c_ptr(value: Option<String>) -> *mut c_char {
    match value {
        Some(text) => string_to_c_ptr(text),
        None => std::ptr::null_mut(),
    }
}

fn map_download_result<T>(result: Result<T, DownloadError>) -> i32 {
    match result {
        Ok(_) => DownloadStatus::Success as i32,
        Err(DownloadError::Io(_)) => DownloadStatus::Io as i32,
        Err(DownloadError::Http(_)) => DownloadStatus::Network as i32,
        Err(DownloadError::Json(_) | DownloadError::Csv(_) | DownloadError::Zip(_)) => {
            DownloadStatus::Parse as i32
        }
    }
}
