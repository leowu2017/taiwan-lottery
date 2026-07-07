use std::path::PathBuf;

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} all [output_dir]");
    eprintln!("  {program} api-doc [output_dir]");
    eprintln!("  {program} history-draw [output_dir]");
    eprintln!("  {program} history-draw-gov [output_dir]");
    eprintln!("  {program} history-draw-taiwan-lottery [output_dir]");
    eprintln!("  {program} dataset <DATASET_CODE> [output_dir]");
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "download".to_string());

    let mode = args.next().unwrap_or_else(|| "all".to_string());
    let result = match mode.as_str() {
        "all" => {
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            taiwan_lottery::download_all(&output_dir).map(|files| {
                println!(
                    "Downloaded {} files into {}",
                    files.len(),
                    output_dir.display()
                );
            })
        }
        "api-doc" => {
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            taiwan_lottery::download_api_doc(&output_dir).map(|path| {
                println!("Downloaded API docs to {}", path.display());
            })
        }
        "history-draw" => {
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            taiwan_lottery::download_history_draw(&output_dir).map(|files| {
                println!(
                    "Downloaded history draw dataset with {} files into {}",
                    files.len(),
                    output_dir.display()
                );
            })
        }
        "history-draw-gov" => {
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            taiwan_lottery::download_history_draw_from_gov_data(&output_dir).map(|files| {
                println!(
                    "Downloaded history draw (gov data) with {} files into {}",
                    files.len(),
                    output_dir.display()
                );
            })
        }
        "history-draw-taiwan-lottery" => {
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            taiwan_lottery::download_history_draw_from_taiwan_lottery(&output_dir).map(|files| {
                println!(
                    "Downloaded history draw (taiwan lottery) with {} files into {}",
                    files.len(),
                    output_dir.display()
                );
            })
        }
        "dataset" => {
            let Some(dataset_code) = args.next() else {
                print_usage(&program);
                std::process::exit(2);
            };
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            taiwan_lottery::download_dataset(&output_dir, &dataset_code).map(|files| {
                println!(
                    "Downloaded dataset {} with {} files into {}",
                    dataset_code,
                    files.len(),
                    output_dir.display()
                );
            })
        }
        _ => {
            print_usage(&program);
            std::process::exit(2);
        }
    };

    if let Err(err) = result {
        eprintln!("Download failed: {err:?}");
        std::process::exit(1);
    }
}
