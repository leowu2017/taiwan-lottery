use std::ffi::CStr;
use std::os::raw::c_char;

use crate::{
    download_all, download_api_doc, download_dataset, download_history_draw,
    download_history_draw_from_gov_data, download_history_draw_from_taiwan_lottery, DownloadError,
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
