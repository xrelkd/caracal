use std::{
    path::{Path, PathBuf},
    process,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let web_frontend_index_file_source =
        [std::env::var("CARGO_MANIFEST_DIR")?.as_str(), "..", "web-frontend", "Cargo.toml"]
            .into_iter()
            .collect::<PathBuf>();

    let web_frontend_dist_dir_path =
        [std::env::var("OUT_DIR")?.as_str(), "frontend-dist"].into_iter().collect::<PathBuf>();

    let web_frontend_index_file_path =
        [web_frontend_dist_dir_path.as_path(), Path::new("index.html")]
            .into_iter()
            .collect::<PathBuf>();

    match web_frontend_index_file_path.try_exists() {
        Ok(false) | Err(_) => {
            // NOTE: flag `--release` makes the build process stuck.
            // FIXME: enable `--release`
            let _result = process::Command::new("trunk")
                .arg("build")
                .arg("--frozen")
                .arg("--locked")
                .arg("--offline")
                .arg("--public-url=/")
                .arg(&format!("--dist={}", web_frontend_dist_dir_path.display()))
                .arg(web_frontend_index_file_source)
                .stdout(process::Stdio::piped())
                .stderr(process::Stdio::piped())
                .spawn()?
                .wait_with_output()?;
            Ok(())
        }
        _ => Ok(()),
    }
}
