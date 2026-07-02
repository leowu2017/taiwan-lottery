use std::path::{Path, PathBuf};

use crate::DownloadError;

pub fn download_api_doc(output_dir: impl AsRef<Path>) -> Result<PathBuf, DownloadError> {
    crate::download_api_doc_impl(output_dir)
}

pub fn download_dataset(
    output_dir: impl AsRef<Path>,
    dataset_code: &str,
) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_dataset_impl(output_dir, dataset_code)
}

pub fn download_history_draw_from_gov_data(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_history_draw_from_gov_data_impl(output_dir)
}

pub fn download_history_draw_from_taiwan_lottery(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_history_draw_from_taiwan_lottery_impl(output_dir)
}

pub fn download_history_draw(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_history_draw_impl(output_dir)
}

pub fn download_all(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    crate::download_all_impl(output_dir)
}
