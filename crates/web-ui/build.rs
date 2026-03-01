use std::{
    path::{Path, PathBuf},
    process::Command,
};

const UI_DIR: &str = "ui";

mod env {
    pub const PREBUILT_WEBUI_DIST: &str = "PREBUILT_WEBUI_DIST";
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_target_dir = {
        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is available");
        [out_dir.as_str(), UI_DIR].iter().collect::<PathBuf>()
    };

    if build_target_dir.exists() {
        std::fs::remove_dir_all(&build_target_dir)?;
    }

    std::fs::create_dir_all(&build_target_dir)?;

    if let Ok(prebuilt_dist) = std::env::var(env::PREBUILT_WEBUI_DIST) {
        copy_prebuilt_webui(&prebuilt_dist, &build_target_dir)?;
    } else {
        build_webui(UI_DIR, &build_target_dir)?;
    }

    static_files::resource_dir(&build_target_dir).build()?;
    Ok(())
}

fn copy_prebuilt_webui(
    prebuilt_dist: impl AsRef<Path>,
    build_target_dir: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:warning=Using prebuilt frontend: {}", prebuilt_dist.as_ref().display());
    copy_dir_all(prebuilt_dist, build_target_dir)
}

fn build_webui(
    source_dir: impl AsRef<Path>,
    build_target_dir: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_dir = source_dir.as_ref().to_path_buf();
    println!("cargo:rerun-if-changed={}/app", source_dir.display());
    println!("cargo:rerun-if-changed={}/public", source_dir.display());
    println!("cargo:rerun-if-changed={}/package.json", source_dir.display());
    println!("cargo:rerun-if-changed={}/package-lock.json", source_dir.display());

    let node_modules_path =
        [source_dir.clone(), PathBuf::from("node_modules")].iter().collect::<PathBuf>();

    if !node_modules_path.exists() {
        // Install frontend dependencies
        let install_output = Command::new("yarn").current_dir(UI_DIR).output()?;
        if !install_output.status.success() {
            eprintln!("npm install failed: {install_output:?}");
            panic!("Failed to install frontend dependencies");
        }
    }

    // Build the frontend
    let build_output = Command::new("yarn").arg("build").current_dir(&source_dir).output()?;
    if !build_output.status.success() {
        eprintln!("yarn build failed: {build_output:?}");
        panic!("Failed to build frontend assets");
    }

    let output_dir = [source_dir, PathBuf::from("out")].iter().collect::<PathBuf>();

    copy_dir_all(output_dir, &build_target_dir)?;

    println!("Frontend built successfully.");

    Ok(())
}

fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = src.as_ref().canonicalize()?;
    std::fs::create_dir_all(&dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        let dst_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            let contents = std::fs::read(entry.path())?;
            std::fs::write(&dst_path, contents)?;
        }
    }

    Ok(())
}
