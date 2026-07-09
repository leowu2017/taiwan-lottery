mod common;
pub mod gaze;
pub mod tlc;

pub use gaze::{
    build_csv_url, download_all, download_api_doc, download_dataset, parse_codes_from_api_docs,
};

#[deprecated(note = "use taiwan_lottery::download::gaze::download_history_draw")]
pub fn download_history_draw(
    output_dir: impl AsRef<std::path::Path>,
) -> Result<Vec<std::path::PathBuf>, crate::DownloadError> {
    gaze::download_history_draw(output_dir)
}
#[deprecated(note = "use taiwan_lottery::download::tlc::download_history_draw")]
pub fn download_history_draw_from_taiwan_lottery(
    output_dir: impl AsRef<std::path::Path>,
) -> Result<Vec<std::path::PathBuf>, crate::DownloadError> {
    tlc::download_history_draw(output_dir)
}
