mod options;
mod realesrgan;
mod error;

pub use options::Options;
pub use options::OptionsScaleFactor;
pub use error::Error;
pub use realesrgan::RealEsrgan;

#[cfg(any(feature = "model-realesr-animevideov3", feature = "model-realesrgan-plus", feature = "model-realesrgan-plus-anime"))]
pub use options::OptionsModel;

#[cfg(feature = "image")]
pub use image::DynamicImage as Image;
