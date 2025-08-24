use std::path::Path;
use realesrgan_rs::{RealEsrgan, Options, OptionsModel};

const IMAGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tests/image.jpg");

#[test]
#[cfg(feature = "model-realesr-animevideov3")]
fn from_file_bytes() {
    assert!(Path::new(IMAGE).exists(), "Test image does not exist");

    let result = RealEsrgan::new(Options::default().model(OptionsModel::RealESRAnimeVideoV3x2));
    assert!(result.is_ok(), "{}", result.err().unwrap().to_string());
    let realesrgan = result.unwrap();

    let input = std::fs::read(IMAGE).unwrap();
    let result = realesrgan.process(&input, 350, 525);
    assert!(result.is_err());
}

#[test]
#[cfg(feature = "model-realesr-animevideov3")]
#[cfg(feature = "image")]
fn from_image() {
    assert!(Path::new(IMAGE).exists(), "Test image does not exist");
    let result = RealEsrgan::new(Options::default().model(OptionsModel::RealESRAnimeVideoV3x2));

    assert!(result.is_ok(), "{}", result.err().unwrap().to_string());
    let realesrgan = result.unwrap();

    let d_image = image::open(IMAGE).expect("Failed to open test image");
    let original_with = d_image.width();
    let original_height = d_image.height();

    let upscaled_image = realesrgan.process_image(&d_image).expect("Failed to upscale image");

    let upscaled_save_path = "/tmp/upscaled.png";
    upscaled_image.save_with_format(upscaled_save_path, image::ImageFormat::Png).unwrap();
    assert!(Path::new(upscaled_save_path).exists(), "Failed to save upscaled image");

    let upscaled_dimensions = image::open(upscaled_save_path).unwrap();
    assert!(
        upscaled_dimensions.width() > original_with && upscaled_dimensions.height() > original_height,
        "Upscaled image is not larger than the original"
    );

    let original_metadata = std::fs::metadata(IMAGE).unwrap();
    let upscaled_metadata = std::fs::metadata(upscaled_save_path).unwrap();
    assert!(
        upscaled_metadata.len() > original_metadata.len(),
        "Upscaled image file is not larger than the original"
    );
    let _ = std::fs::remove_file(upscaled_save_path);

}