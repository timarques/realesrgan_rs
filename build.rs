use std::fs;
use std::path::{Path, PathBuf};

const REALESRGAN_RELEASE: &str = "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesrgan-ncnn-vulkan-20220424-ubuntu.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let models_directory = output_directory.join("models");
    let manifest_directory = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_directory = manifest_directory.join("src");

    let cpp_directory = source_directory.join("cpp");
    let shaders_directory = source_directory.join("shaders");

    let spirv_header_directory = output_directory.join("spirv_headers");

    println!("cargo:rerun-if-changed={}", cpp_directory.display());
    println!("cargo:rerun-if-changed={}", shaders_directory.display());
    println!("cargo:rerun-if-changed={}", models_directory.display());
    println!("cargo:rustc-env=MODELS_DIRECTORY={}", models_directory.display());

    download_models(&output_directory, &models_directory)?;
    generate_spirv_headers(&shaders_directory, &spirv_header_directory)?;
    build_library(&cpp_directory, &spirv_header_directory);

    Ok(())
}

fn download_models(
    output: &Path,
    models_directory: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(models_directory)?;

    let release_dir = output.join("release");
    if release_dir.exists() {
        fs::remove_dir_all(&release_dir)?;
    }

    let zip_path = output.join("realesrgan-ncnn-vulkan-20220424-ubuntu.zip");

    let response = minreq::get(REALESRGAN_RELEASE).send()?;
    let zip_data = response.as_bytes();
    fs::write(&zip_path, zip_data)?;

    let file = fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    fs::create_dir_all(&release_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = release_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    fs::remove_file(&zip_path)?;
    copy_dir_all(&release_dir.join("models"), &models_directory.to_path_buf())?;
    fs::remove_dir_all(&release_dir)?;

    Ok(())
}

fn copy_dir_all(source: &PathBuf, destination: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn generate_spirv_headers(shaders_directory: &Path, output_directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !shaders_directory.exists() {
        return Err(format!("Shaders directory does not exist: {}", shaders_directory.display()).into());
    }

    let mut compiler = shaderc::Compiler::new()?;
    let options = shaderc::CompileOptions::new()?;

    fs::create_dir_all(output_directory)?;

    let shaders = [
        "realesrgan_preproc.comp",
        "realesrgan_postproc.comp",
        "realesrgan_preproc_tta.comp",
        "realesrgan_postproc_tta.comp",
    ];

    for shader_name in &shaders {
        let shader_path = shaders_directory.join(shader_name);
        
        if !shader_path.exists() {
            return Err(format!("Shader file does not exist: {}", shader_path.display()).into());
        }

        let shader_source = fs::read_to_string(&shader_path)?;
        let stem = Path::new(shader_name)
            .file_stem()
            .ok_or("Failed to get file stem")?
            .to_str()
            .ok_or("Failed to convert file stem to string")?;

        compile_shader_variant(
            &mut compiler,
            &options,
            &shader_source,
            stem,
            "",
            output_directory,
        )?;
        compile_shader_variant(
            &mut compiler,
            &options,
            &shader_source,
            stem,
            "_fp16s",
            output_directory,
        )?;
        compile_shader_variant(
            &mut compiler,
            &options,
            &shader_source,
            stem,
            "_int8s",
            output_directory,
        )?;
    }
    
    Ok(())
}

fn compile_shader_variant(
    compiler: &mut shaderc::Compiler,
    base_options: &shaderc::CompileOptions,
    source: &str,
    base_name: &str,
    variant_suffix: &str,
    output_directory: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut options = base_options.clone()?;

    match variant_suffix {
        "_fp16s" => {
            options.add_macro_definition("NCNN_fp16_storage", Some("1"));
        }
        "_int8s" => {
            options.add_macro_definition("NCNN_fp16_storage", Some("1"));
            options.add_macro_definition("NCNN_int8_storage", Some("1"));
        }
        _ => {}
    }

    let result = compiler
        .compile_into_spirv(
            source,
            shaderc::ShaderKind::Compute,
            &format!("{base_name}.comp"),
            "main",
            Some(&options),
        )?;

    let spirv_bytes = result.as_binary_u8();
    let hex_content = generate_hex_header(spirv_bytes);

    let output_filename = format!("{base_name}{variant_suffix}.spv.hex.h");
    let output_path = output_directory.join(output_filename);
    fs::write(output_path, hex_content)?;
    
    Ok(())
}

fn generate_hex_header(spirv_bytes: &[u8]) -> String {
    let mut header = String::new();
    let spirv_u32 = spirv_bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<u32>>();

    for (index, value) in spirv_u32.iter().enumerate() {
        if index % 8 == 0 {
            header.push_str("    ");
        }
        header.push_str(&format!("0x{value:08x}"));
        if index < spirv_u32.len() - 1 {
            header.push(',');
        }
        if index % 8 == 7 || index == spirv_u32.len() - 1 {
            header.push('\n');
        } else {
            header.push(' ');
        }
    }
    header
}

fn build_library(cpp_directory: &Path, spirv_include_directory: &Path) {
    println!("cargo:rustc-link-lib=stdc++");

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=gomp");
        println!("cargo:rustc-link-lib=pthread");
    }

    pkg_config::probe_library("ncnn").unwrap();
    pkg_config::probe_library("vulkan").unwrap();

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .std("c++17")
        .opt_level(3)
        .warnings(false)
        .flag_if_supported("-fopenmp")
        .include(cpp_directory)
        .include(spirv_include_directory)
        .file(cpp_directory.join("wrapper.cpp"))
        .file(cpp_directory.join("realesrgan.cpp"));

    build.compile("realesrgan-wrapper");
}