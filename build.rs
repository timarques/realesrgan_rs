use std::path::{Path, PathBuf};
use std::fs;

fn main() {
    let output_directory = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest_directory = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_directory = manifest_directory.join("src");

    let cpp_directory = source_directory.join("cpp");
    let shaders_directory = source_directory.join("shaders");

    let spirv_header_directory = output_directory.join("spirv_headers");
    fs::create_dir_all(&spirv_header_directory).unwrap();

    println!("cargo:rerun-if-changed={}", cpp_directory.display());
    println!("cargo:rerun-if-changed={}", shaders_directory.display());

    generate_spirv_headers(&shaders_directory, &spirv_header_directory);
    build_library(&cpp_directory, &spirv_header_directory);
}

fn generate_spirv_headers(shaders_directory: &Path, output_directory: &Path) {
    let mut compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();
    
    let shaders = [
        "realesrgan_preproc.comp",
        "realesrgan_postproc.comp", 
        "realesrgan_preproc_tta.comp",
        "realesrgan_postproc_tta.comp",
    ];

    for shader_name in &shaders {
        let shader_path = shaders_directory.join(shader_name);
        let shader_source = fs::read_to_string(&shader_path).unwrap();
        let stem = Path::new(shader_name).file_stem().unwrap().to_str().unwrap();

        compile_shader_variant(&mut compiler, &options, &shader_source, stem, "", output_directory);
        compile_shader_variant(&mut compiler, &options, &shader_source, stem, "_fp16s", output_directory);
        compile_shader_variant(&mut compiler, &options, &shader_source, stem, "_int8s", output_directory);
    }
}

fn compile_shader_variant(
    compiler: &mut shaderc::Compiler,
    base_options: &shaderc::CompileOptions,
    source: &str,
    base_name: &str,
    variant_suffix: &str,
    output_directory: &Path,
) {
    let mut options = base_options.clone().unwrap();
    
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

    let result = compiler.compile_into_spirv(
        source,
        shaderc::ShaderKind::Compute,
        &format!("{base_name}.comp"),
        "main",
        Some(&options),
    ).unwrap();

    let spirv_bytes = result.as_binary_u8();
    let hex_content = generate_hex_header(spirv_bytes);
    
    let output_filename = format!("{base_name}{variant_suffix}.spv.hex.h");
    let output_path = output_directory.join(output_filename);
    fs::write(output_path, hex_content).unwrap();
}

fn generate_hex_header(spirv_bytes: &[u8]) -> String {
    let mut header = String::new();
    let spirv_u32 = spirv_bytes.chunks_exact(4)
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