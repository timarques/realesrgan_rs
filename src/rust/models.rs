#[cfg(feature = "model-realesr-animevideov3")]
const MODEL_REALESR_ANIMEVIDEOV3_X2: (&[u8], &[u8]) = ( 
    include_bytes!("../../models/realesr-animevideov3-x2.param"),
    include_bytes!("../../models/realesr-animevideov3-x2.bin")
);

#[cfg(feature = "model-realesr-animevideov3")]
const MODEL_REALESR_ANIMEVIDEOV3_X3: (&[u8], &[u8]) = ( 
    include_bytes!("../../models/realesr-animevideov3-x3.param"),
    include_bytes!("../../models/realesr-animevideov3-x3.bin")
);

#[cfg(feature = "model-realesr-animevideov3")]
const MODEL_REALESR_ANIMEVIDEOV3_X4: (&[u8], &[u8]) = (
    include_bytes!("../../models/realesr-animevideov3-x4.param"),
    include_bytes!("../../models/realesr-animevideov3-x4.bin"),
);

#[cfg(feature = "model-realesrgan-plus")]
const MODEL_REALESRGAN_X4PLUS: (&[u8], &[u8]) = (
    include_bytes!("../../models/realesrgan-x4plus.param"),
    include_bytes!("../../models/realesrgan-x4plus.bin"),
);

#[cfg(feature = "model-realesrgan-plus-anime")]
const MODEL_REALESRGAN_X4PLUS_ANIME: (&[u8], &[u8]) = (
    include_bytes!("../../models/realesrgan-x4plus-anime.param"),
    include_bytes!("../../models/realesrgan-x4plus-anime.bin"),
);

#[cfg(any(feature = "model-realesr-animevideov3", feature = "model-realesrgan-plus", feature = "model-realesrgan-plus-anime"))]
pub enum Model {
    #[cfg(feature = "model-realesr-animevideov3")]
    RealESRAnimeVideoV3x2,
    #[cfg(feature = "model-realesr-animevideov3")]
    RealESRAnimeVideoV3x3,
    #[cfg(feature = "model-realesr-animevideov3")]
    RealESRAnimeVideoV3x4,
    #[cfg(feature = "model-realesrgan-plus")]
    RealESRGANPlusx4,
    #[cfg(feature = "model-realesrgan-plus-anime")]
    RealESRGANPlusx4Anime,
}

impl Model {
    pub fn get_bytes(&self) -> (&'static [u8], &'static [u8]) {
        match self {
            #[cfg(feature = "model-realesr-animevideov3")]
            Self::RealESRAnimeVideoV3x2 => MODEL_REALESR_ANIMEVIDEOV3_X2,
            #[cfg(feature = "model-realesr-animevideov3")]
            Self::RealESRAnimeVideoV3x3 => MODEL_REALESR_ANIMEVIDEOV3_X3,
            #[cfg(feature = "model-realesr-animevideov3")]
            Self::RealESRAnimeVideoV3x4 => MODEL_REALESR_ANIMEVIDEOV3_X4,
            #[cfg(feature = "model-realesrgan-plus")]
            Self::RealESRGANPlusx4 => MODEL_REALESRGAN_X4PLUS,
            #[cfg(feature = "model-realesrgan-plus-anime")]
            Self::RealESRGANPlusx4Anime => MODEL_REALESRGAN_X4PLUS_ANIME,
        }
    }

    pub fn get_scale_factor(&self) -> u8 {
        match self {
            #[cfg(feature = "model-realesr-animevideov3")]
            Self::RealESRAnimeVideoV3x2 => 2,
            #[cfg(feature = "model-realesr-animevideov3")]
            Self::RealESRAnimeVideoV3x3 => 3,
            #[cfg(feature = "model-realesr-animevideov3")]
            Self::RealESRAnimeVideoV3x4 => 4,
            #[cfg(feature = "model-realesrgan-plus")]
            Self::RealESRGANPlusx4 => 4,
            #[cfg(feature = "model-realesrgan-plus-anime")]
            Self::RealESRGANPlusx4Anime => 4,
        }
    }
}