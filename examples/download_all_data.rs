use std::path::PathBuf;

fn main() {
    let output_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"));

    match taiwan_lottery::download_all_data(&output_dir) {
        Ok(files) => {
            println!("Downloaded {} files into {}", files.len(), output_dir.display());
        }
        Err(err) => {
            eprintln!("Download failed: {err:?}");
            std::process::exit(1);
        }
    }
}
