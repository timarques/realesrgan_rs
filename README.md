# RealEsrgan_rs

**RealEsrgan_rs** is a Rust wrapper for the [Real-ESRGAN-ncnn-vulkan](https://github.com/xinntao/Real-ESRGAN-ncnn-vulkan). It provides a convenient interface for using real-esrgan-ncnn-vulkan features in Rust.

## Installation

Install dependencies
```sh
dnf install vulkan-headers vulkan-loader-devel
```
```sh
apt-get install libvulkan-dev
```
```sh
pacman -S vulkan-headers vulkan-icd-loader
```

Add this to your Cargo.toml:

```toml
[dependencies]
realesrgan-rs = { git = "https://github.com/timarques/realesrgan_rs.git" }
```

```rs
use realesrgan_rs::{RealEsrgan, Options, OptionsModel};
use image;

let param_path = "path/to/param/file";
let bin_path = "path/to/bin/file";

let realesrgan = RealEsrgan::new(Options::default().model(OptionsModel::RealESRAnimeVideoV3x2)).unwrap();

let input_image = image::open("input.png").unwrap();
let output_image = realesrgan.process_image(input_image)?;
output_image.save("output.png").unwrap();
```

## Advanced Configuration

The Builder pattern allows for detailed configuration:

```rs
use realesrgan_rs::{RealEsrgan, Options, OptionsScaleFactor};

let realesrgan_options = Options::default()
    .gpuid(0)
    .tta_mode(false)
    .tilesize(0)
    .scale_factor(OptionsScaleFactor::Quadruple)
    .model_files("/path/to/model.param", "/path/to/model.bin");
let realesrgan = RealEsrgan::new(options);
```

## Features

This project uses feature flags to control optional dependencies and functionalities. Below is an explanation of the available features:

- **default = ["image", "models"]**  
  The default feature set includes support for image processing through the Rust `image` library and access to various embedded AI-based upscaling models.

- **system-ncnn**  
  The `system-ncnn` feature links the project to an externally installed `ncnn` library on your system, rather than building it locally. This is useful if you have `ncnn` pre-installed and want to avoid recompiling it.

- **image**  
  The `image` feature enables the use of the Rust `image` crate for handling image processing tasks, such as decoding, encoding, and manipulating image data.

- **models = ["model-realesr-animevideov3", "model-realesrgan-plus", "model-realesrgan-plus-anime"]**  
  The `models` feature enables several embedded AI-based upscaling models for enhancing images and videos:

  - **model-realesr-animevideov3**: Adds support for the `Real-ESRGAN Anime Video v3` model, optimized for upscaling anime-style videos.
  - **model-realesrgan-plus**: Includes the `Real-ESRGAN+` model, designed for general-purpose image and video upscaling with improved detail preservation.
  - **model-realesrgan-plus-anime**: Provides upscaling for anime-style content using the `Real-ESRGAN+ Anime` model.