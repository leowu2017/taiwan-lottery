use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::{
    download_all, download_api_doc, download_dataset, download_history_draw,
    download_history_draw_from_gov_data, download_history_draw_from_taiwan_lottery,
    query_history_draw, query_history_draw_from_taiwan_lottory, DownloadError,
    HistoryDrawQuery, HistoryGame, HistorySession,
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
    InvalidSession = 9,
    InvalidQueryUtf8 = 10,
    NullResultPointer = 11,
}

#[repr(C)]
pub struct HistoryDrawItemC {
    period: *mut c_char,
    lottery_date: *mut c_char,
    redeemable_date: *mut c_char,
    draw_number_size: *mut i32,
    draw_number_size_len: usize,
    draw_number_appear: *mut i32,
    draw_number_appear_len: usize,
    has_draw_number_appear: u8,
}

#[repr(C)]
pub struct HistoryDrawPageC {
    total_size: usize,
    item_count: usize,
    items: *mut HistoryDrawItemC,
}

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
pub extern "C" fn download_dataset_ffi(output_dir: *const c_char, dataset_code: *const c_char) -> i32 {
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
    session: i32,
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

    let game = match int_to_history_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let session = match int_to_history_session(session) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let query = match build_history_draw_query(period, month, end_month, session) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let result = query_history_draw(out_dir, game, query);
    map_history_result_to_struct_status(result, out_page)
}

#[unsafe(export_name = "query_history_draw_from_taiwan_lottory")]
pub extern "C" fn query_history_draw_from_taiwan_lottory_ffi(
    game: i32,
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
    session: i32,
    out_page: *mut *mut HistoryDrawPageC,
) -> i32 {
    let game = match int_to_history_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let session = match int_to_history_session(session) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let query = match build_history_draw_query(period, month, end_month, session) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let result = query_history_draw_from_taiwan_lottory(game, query);
    map_history_result_to_struct_status(result, out_page)
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
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(page.items, page.item_count))
        };

        for item in &*items_box {
            free_history_draw_item(item);
        }
    }
}

fn c_str_arg_to_string(ptr: *const c_char, null_status: i32, invalid_utf8_status: i32) -> Result<String, i32> {
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

fn optional_c_str_arg_to_string(ptr: *const c_char, invalid_utf8_status: i32) -> Result<Option<String>, i32> {
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

fn int_to_history_game(value: i32) -> Result<HistoryGame, i32> {
    match value {
        0 => Ok(HistoryGame::SuperLotto638),
        1 => Ok(HistoryGame::Lotto649),
        2 => Ok(HistoryGame::Daily539),
        3 => Ok(HistoryGame::Lotto3D),
        4 => Ok(HistoryGame::Lotto4D),
        5 => Ok(HistoryGame::Lotto49M6),
        6 => Ok(HistoryGame::Lotto39M5),
        7 => Ok(HistoryGame::Lotto38M6),
        8 => Ok(HistoryGame::Lotto1224),
        9 => Ok(HistoryGame::Lotto740),
        10 => Ok(HistoryGame::TicTacToe),
        11 => Ok(HistoryGame::Lotto638),
        _ => Err(DownloadStatus::InvalidGame as i32),
    }
}

fn int_to_history_session(value: i32) -> Result<HistorySession, i32> {
    match value {
        0 => Ok(HistorySession::Third),
        1 => Ok(HistorySession::Fourth),
        2 => Ok(HistorySession::Fifth),
        _ => Err(DownloadStatus::InvalidSession as i32),
    }
}

fn build_history_draw_query(
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
    session: HistorySession,
) -> Result<HistoryDrawQuery, i32> {
    let period = optional_c_str_arg_to_string(period, DownloadStatus::InvalidQueryUtf8 as i32)?;
    let month = optional_c_str_arg_to_string(month, DownloadStatus::InvalidQueryUtf8 as i32)?;
    let end_month = optional_c_str_arg_to_string(end_month, DownloadStatus::InvalidQueryUtf8 as i32)?;

    Ok(HistoryDrawQuery {
        period,
        month,
        end_month,
        session,
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

fn history_item_to_c(item: crate::HistoryDrawItem) -> HistoryDrawItemC {
    let draw_number_size_len = item.draw_number_size.len();
    let draw_number_size_ptr = if draw_number_size_len == 0 {
        std::ptr::null_mut()
    } else {
        Box::into_raw(item.draw_number_size.into_boxed_slice()) as *mut i32
    };

    let (draw_number_appear_ptr, draw_number_appear_len, has_draw_number_appear) =
        if let Some(numbers) = item.draw_number_appear {
            if numbers.is_empty() {
                (std::ptr::null_mut(), 0, 1)
            } else {
                let len = numbers.len();
                let ptr = Box::into_raw(numbers.into_boxed_slice()) as *mut i32;
                (ptr, len, 1)
            }
        } else {
            (std::ptr::null_mut(), 0, 0)
        };

    HistoryDrawItemC {
        period: string_to_c_ptr(item.period),
        lottery_date: optional_string_to_c_ptr(item.lottery_date),
        redeemable_date: optional_string_to_c_ptr(item.redeemable_date),
        draw_number_size: draw_number_size_ptr,
        draw_number_size_len,
        draw_number_appear: draw_number_appear_ptr,
        draw_number_appear_len,
        has_draw_number_appear,
    }
}

fn free_history_draw_item(item: &HistoryDrawItemC) {
    if !item.period.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.period) };
    }
    if !item.lottery_date.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.lottery_date) };
    }
    if !item.redeemable_date.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.redeemable_date) };
    }

    if !item.draw_number_size.is_null() {
        // SAFETY: pointer/len pair was created from Box<[i32]> in this crate.
        let _ = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                item.draw_number_size,
                item.draw_number_size_len,
            ))
        };
    }

    if !item.draw_number_appear.is_null() {
        // SAFETY: pointer/len pair was created from Box<[i32]> in this crate.
        let _ = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                item.draw_number_appear,
                item.draw_number_appear_len,
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
