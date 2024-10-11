use cmake::Config;
use core::panic;
use std::process::Command;
use std::io::BufRead;
use std::path::PathBuf;

const NCNN_REPO_URL: &str = "https://github.com/Tencent/ncnn";
const NCNN_COMMIT_HASH: &str = "6125c9f47cd14b589de0521350668cf9d3d37e3c";
const REALESRGAN_RELEASE: &str = "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesrgan-ncnn-vulkan-20220424-ubuntu.zip";

fn execute_command(command: &mut Command) -> Result<(), String> {
    let status = command.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("Command failed with exit code: {}", status));
    }
    Ok(())
}

fn download_models(output: &PathBuf) -> Result<(), String> {
    let models_dir = "./models";
    if std::fs::exists(&models_dir).unwrap() {
        return Ok(())
    }

    let release_dir = output.join("release");
    execute_command(
        Command::new("wget")
            .args(&["-P", &output.to_str().unwrap()])
            .arg(REALESRGAN_RELEASE)
    )?;

    execute_command(
        Command::new("unzip")
            .arg(output.join("realesrgan-ncnn-vulkan-20220424-ubuntu.zip"))
            .args(&["-d", &release_dir.to_str().unwrap()])
    )?;

    std::fs::rename(release_dir.join("models"), &models_dir)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn clone_ncnn(target_dir: &PathBuf) -> Result<(), String> {
    if std::fs::exists(target_dir).unwrap() {
        return Ok(())
    }
    execute_command(
        Command::new("git")
            .args(&["clone", "--recursive", NCNN_REPO_URL])
            .arg(target_dir)
    )?;

    execute_command(
        Command::new("git")
            .current_dir(target_dir)
            .args(&["checkout", NCNN_COMMIT_HASH])
    )?;
    Ok(())
}

fn configure_ncnn_build(target_dir: &PathBuf) -> Config {
    let mut config = Config::new(target_dir);
    config.define("NCNN_BUILD_TOOLS", "OFF")
          .define("NCNN_BUILD_EXAMPLES", "OFF")
          .define("NCNN_BUILD_BENCHMARK", "OFF")
          .define("NCNN_ENABLE_LTO", "ON")
          .define("NCNN_SHARED_LIB", "OFF")
          .define("NCNN_VULKAN", "ON")
          .define("NCNN_SYSTEM_GLSLANG", "OFF")
          .define("CMAKE_BUILD_TYPE", "Release");
    config
}

fn disable_logs(target_dir: &PathBuf) -> Result<(), std::io::Error> {
    let platform_file = target_dir.join("src").join("platform.h.in");
    let file = std::fs::File::open(&platform_file)?;
    let reader = std::io::BufReader::new(file);

    let mut text = String::new();
    let mut skip_lines = false;

    for line_result in reader.lines() {
        let line = line_result?;

        if line.contains("#define NCNN_LOGE(...) do {") {
            skip_lines = true;
            text += &format!("#define NCNN_LOGE(...)\n");
            continue
        } else if (line.contains("#endif") || line.contains("#else")) && skip_lines {
            skip_lines = false;
        } else if skip_lines {
            continue
        }
        text += &format!("{}\n", line);
    }

    std::fs::write(platform_file, text)?;

    Ok(())
}

fn build_ncnn(output: &PathBuf) -> Result<(), String> {
    let target_dir = output.join("ncnn");

    println!("cargo:rustc-link-lib={}", "vulkan");
    println!("cargo:rustc-link-lib={}", "omp");

    clone_ncnn(&target_dir)?;
    disable_logs(&target_dir).map_err(|r| r.to_string())?;
    configure_ncnn_build(&target_dir)
        .cflag("-O3")
        .cxxflag("-O3")
        .build();

    println!("cargo:rustc-link-lib=static={}", "MachineIndependent");
    println!("cargo:rustc-link-lib=static={}", "SPIRV");
    println!("cargo:rustc-link-lib=static={}", "GenericCodeGen");
    println!("cargo:rustc-link-lib=static={}", "OSDependent");
    println!("cargo:rustc-link-lib=static={}", "OGLCompiler");
    println!("cargo:rustc-link-lib=static={}", "glslang");
    println!("cargo:rustc-link-lib=static={}", "ncnn");
    Ok(())
}

fn main() {
    let output = std::env::var("OUT_DIR").unwrap();
    let output_path = PathBuf::from(&output);

    if cfg!(feature = "model-realesr-animevideov3") || cfg!(feature = "model-realesrgan-plus") || cfg!(feature = "model-realesrgan-plus-anime") {
        if let Err(e) = download_models(&output_path) {
            panic!("Failed to download models: {}", e);
        }
    }

    println!("cargo:rustc-link-search=native={}", output_path.join("lib").display());
    println!("cargo:rustc-link-search=native={}", output_path.join("lib64").display());

    println!("cargo:rustc-link-lib={}", "stdc++");
    if cfg!(feature = "system-ncnn") {
        println!("cargo:rustc-link-lib={}", "ncnn");
    } else {
        if let Err(e) = build_ncnn(&output_path) {
            panic!("Failed to build ncnn: {}", e);
        }
    }
    Config::new("src").build();
    println!("cargo:rustc-link-lib=static={}", "realesrgan-wrapper");
}