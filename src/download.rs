use std::path::{Path, PathBuf};

use crate::DownloadError;

/// Downloads the FinancialPlanning API documentation.
///
/// # Arguments
/// * `output_dir` - Directory where the API docs JSON file will be saved
///
/// # Returns
/// The path to the downloaded `financialplanning_api_docs.json` file.
///
/// # Errors
/// Returns [`DownloadError`] if the download or file write fails.
pub fn download_api_doc(output_dir: impl AsRef<Path>) -> Result<PathBuf, DownloadError> {
    crate::download_api_doc_impl(output_dir)
}

/// Downloads a specific lottery dataset by code.
///
/// Downloads the CSV file and all linked resources, automatically extracting ZIP archives.
///
/// # Arguments
/// * `output_dir` - Directory where files will be saved (creates `output_dir/<dataset_code>/`)
/// * `dataset_code` - Dataset code, e.g., "D423F", "D416F"
///
/// # Returns
/// A vector of paths to all downloaded and extracted files.
///
/// # Errors
/// Returns [`DownloadError`] if the download or extraction fails.
pub fn download_dataset(
    output_dir: impl AsRef<Path>,
    dataset_code: &str,
) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_dataset_impl(output_dir, dataset_code)
}

/// Downloads history draw data from FinancialPlanning OpenData.
///
/// Uses the `D423F` dataset as the primary source.
///
/// # Arguments
/// * `output_dir` - Directory where files will be saved (creates `output_dir/D423F/`)
///
/// # Returns
/// A vector of paths to all downloaded files.
///
/// # Errors
/// Returns [`DownloadError`] if the download fails.
pub fn download_history_draw_from_gov_data(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_history_draw_from_gov_data_impl(output_dir)
}

/// Downloads history draw data from Taiwan Lottery API.
///
/// Downloads yearly ZIP files starting from 2007.
///
/// # Arguments
/// * `output_dir` - Directory where files will be saved (creates `output_dir/D423F/`)
///
/// # Returns
/// A vector of paths to all downloaded files.
///
/// # Errors
/// Returns [`DownloadError`] if the download fails.
pub fn download_history_draw_from_taiwan_lottery(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_history_draw_from_taiwan_lottery_impl(output_dir)
}

/// Downloads history draw data with automatic fallback strategy.
///
/// Attempts to download from FinancialPlanning OpenData first, and falls back to
/// Taiwan Lottery API only if the primary source fails with an HTTP/network error.
///
/// # Arguments
/// * `output_dir` - Directory where files will be saved (creates `output_dir/D423F/`)
///
/// # Returns
/// A vector of paths to all downloaded files.
///
/// # Errors
/// Returns [`DownloadError`] if both primary and fallback sources fail.
pub fn download_history_draw(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_history_draw_impl(output_dir)
}

/// Downloads all available lottery datasets and API documentation.
///
/// This is a convenience function that:
/// 1. Downloads the API documentation
/// 2. Parses all available dataset codes
/// 3. Downloads each dataset with all linked resources
///
/// # Arguments
/// * `output_dir` - Root directory where all files will be organized by dataset code
///
/// # Returns
/// A vector of paths to all downloaded files.
///
/// # Errors
/// Returns [`DownloadError`] if any download or parsing step fails.
pub fn download_all(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_all_impl(output_dir)
}
