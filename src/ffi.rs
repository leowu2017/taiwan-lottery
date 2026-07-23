use std::ffi::CString;
use std::os::raw::c_char;

use crate::download::gaze::download_history_draw as download_history_draw_gaze;
use crate::download::tlc::download_history_draw as download_history_draw_from_taiwan_lottery_tlc;
use crate::{
    download_all, download_api_doc, download_dataset, draw_by_game, query_history_draw,
    query_history_draw_from_taiwan_lottery, DownloadError, LotteryDisplayLanguage,
};

mod ffi_args;
mod ffi_convert;
mod ffi_free;
mod ffi_status;

use ffi_args::{
    build_history_draw_query, c_str_arg_to_string, int_to_display_language, int_to_lottery_game,
};
use ffi_convert::{
    draw_result_to_c, history_page_to_c, lottery_game_date_query_range_to_c,
    lottery_game_metadata_to_c, lottery_game_query_range_to_c,
};
use ffi_free::{free_draw_numbers, free_history_draw_item};
use ffi_status::{map_download_result, DownloadStatus};

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

#[repr(C)]
pub struct LotteryGameDateQueryRangeC {
    min_date: *mut c_char,
    max_date: *mut c_char,
}

#[repr(C)]
pub struct LotteryGameNumberRuleC {
    name: *mut c_char,
    picks: usize,
    min: i32,
    max: i32,
    allow_repeat: i32,
}

#[repr(C)]
pub struct LotteryGameMetadataC {
    display_name: *mut c_char,
    display_name_english: *mut c_char,
    display_name_chinese: *mut c_char,
    number_rule: *mut c_char,
    number_range_count: usize,
    number_ranges: *mut LotteryGameNumberRuleC,
}

#[repr(C)]
pub struct RemoteQueryParamSupportC {
    month: i32,
    end_month: i32,
    open_date: i32,
    period: i32,
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

    map_download_result(download_history_draw_gaze(out_dir))
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

    map_download_result(download_history_draw_from_taiwan_lottery_tlc(out_dir))
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
    let query = match build_history_draw_query(period, month, end_month, std::ptr::null()) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let result = query_history_draw(out_dir, game, query);
    map_history_result_to_struct_status(result, out_page)
}

#[unsafe(export_name = "query_history_draw_with_open_date")]
pub extern "C" fn query_history_draw_with_open_date_ffi(
    output_dir: *const c_char,
    game: i32,
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
    open_date: *const c_char,
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
    let query = match build_history_draw_query(period, month, end_month, open_date) {
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
    let query = match build_history_draw_query(period, month, end_month, std::ptr::null()) {
        Ok(value) => value,
        Err(status) => return status,
    };

    let result = query_history_draw_from_taiwan_lottery(game, query);
    map_history_result_to_struct_status(result, out_page)
}

#[unsafe(export_name = "query_history_draw_from_taiwan_lottery_with_open_date")]
pub extern "C" fn query_history_draw_from_taiwan_lottery_with_open_date_ffi(
    game: i32,
    period: *const c_char,
    month: *const c_char,
    end_month: *const c_char,
    open_date: *const c_char,
    out_page: *mut *mut HistoryDrawPageC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let query = match build_history_draw_query(period, month, end_month, open_date) {
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

#[unsafe(export_name = "lottery_game_query_date_range")]
pub extern "C" fn lottery_game_query_date_range_ffi(
    game: i32,
    out_range: *mut *mut LotteryGameDateQueryRangeC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_range.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let range = game.query_date_range();
    let c_range = lottery_game_date_query_range_to_c(range);
    unsafe {
        *out_range = Box::into_raw(c_range);
    }

    DownloadStatus::Success as i32
}

#[unsafe(export_name = "lottery_game_query_date_range_for_local")]
pub extern "C" fn lottery_game_query_date_range_for_local_ffi(
    game: i32,
    out_range: *mut *mut LotteryGameDateQueryRangeC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_range.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let range = game.query_date_range_for_local();
    let c_range = lottery_game_date_query_range_to_c(range);
    unsafe {
        *out_range = Box::into_raw(c_range);
    }

    DownloadStatus::Success as i32
}

#[unsafe(export_name = "lottery_game_query_date_range_for_remote")]
pub extern "C" fn lottery_game_query_date_range_for_remote_ffi(
    game: i32,
    out_range: *mut *mut LotteryGameDateQueryRangeC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_range.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let range = game.query_date_range_for_remote();
    let c_range = lottery_game_date_query_range_to_c(range);
    unsafe {
        *out_range = Box::into_raw(c_range);
    }

    DownloadStatus::Success as i32
}

#[unsafe(export_name = "lottery_game_metadata")]
pub extern "C" fn lottery_game_metadata_ffi(
    game: i32,
    out_metadata: *mut *mut LotteryGameMetadataC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_metadata.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let metadata = game.metadata_with_language(LotteryDisplayLanguage::English);
    let c_metadata = lottery_game_metadata_to_c(metadata);
    unsafe {
        *out_metadata = Box::into_raw(c_metadata);
    }

    DownloadStatus::Success as i32
}

#[unsafe(export_name = "lottery_game_metadata_with_language")]
pub extern "C" fn lottery_game_metadata_with_language_ffi(
    game: i32,
    language: i32,
    out_metadata: *mut *mut LotteryGameMetadataC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let language = match int_to_display_language(language) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_metadata.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let metadata = game.metadata_with_language(language);
    let c_metadata = lottery_game_metadata_to_c(metadata);
    unsafe {
        *out_metadata = Box::into_raw(c_metadata);
    }

    DownloadStatus::Success as i32
}

#[unsafe(export_name = "lottery_game_remote_query_param_support")]
pub extern "C" fn lottery_game_remote_query_param_support_ffi(
    game: i32,
    out_support: *mut RemoteQueryParamSupportC,
) -> i32 {
    let game = match int_to_lottery_game(game) {
        Ok(value) => value,
        Err(status) => return status,
    };

    if out_support.is_null() {
        return DownloadStatus::NullResultPointer as i32;
    }

    let support = game.remote_query_param_support();
    unsafe {
        *out_support = RemoteQueryParamSupportC {
            month: i32::from(support.month),
            end_month: i32::from(support.end_month),
            open_date: i32::from(support.open_date),
            period: i32::from(support.period),
        };
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

#[unsafe(export_name = "free_lottery_game_query_date_range")]
pub extern "C" fn free_lottery_game_query_date_range_ffi(range: *mut LotteryGameDateQueryRangeC) {
    if range.is_null() {
        return;
    }

    let range = unsafe { Box::from_raw(range) };

    if !range.min_date.is_null() {
        let _ = unsafe { CString::from_raw(range.min_date) };
    }
    if !range.max_date.is_null() {
        let _ = unsafe { CString::from_raw(range.max_date) };
    }
}

#[unsafe(export_name = "free_lottery_game_metadata")]
pub extern "C" fn free_lottery_game_metadata_ffi(metadata: *mut LotteryGameMetadataC) {
    if metadata.is_null() {
        return;
    }

    let metadata = unsafe { Box::from_raw(metadata) };

    if !metadata.display_name.is_null() {
        let _ = unsafe { CString::from_raw(metadata.display_name) };
    }
    if !metadata.display_name_english.is_null() {
        let _ = unsafe { CString::from_raw(metadata.display_name_english) };
    }
    if !metadata.display_name_chinese.is_null() {
        let _ = unsafe { CString::from_raw(metadata.display_name_chinese) };
    }
    if !metadata.number_rule.is_null() {
        let _ = unsafe { CString::from_raw(metadata.number_rule) };
    }
    if !metadata.number_ranges.is_null() {
        let rules = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                metadata.number_ranges,
                metadata.number_range_count,
            ))
        };

        for rule in &*rules {
            if !rule.name.is_null() {
                let _ = unsafe { CString::from_raw(rule.name) };
            }
        }
    }
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
