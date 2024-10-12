use std::path::Path;

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
pub enum OptionsModel {
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

#[cfg(any(feature = "model-realesr-animevideov3", feature = "model-realesrgan-plus", feature = "model-realesrgan-plus-anime"))]
impl OptionsModel {

    pub const fn get_bytes(&self) -> (&'static [u8], &'static [u8]) {
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

    pub const fn get_scale_factor(&self) -> i32 {
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

pub enum OptionsScaleFactor {
    Double = 2,
    Triple = 3,
    Quadruple = 4,
}

pub struct Options<'a> {
    pub gpuid: i32,
    pub tta_mode: bool,
    pub tilesize: i32,
    pub scale_factor: i32,
    pub param: &'a [u8],
    pub bin: &'a [u8],
}

impl <'a>Default for Options<'a> {

    fn default() -> Self {
        Self {
            gpuid: 0,
            tta_mode: false,
            tilesize: 0,
            scale_factor: 4,
            param: Self::DEFAULT_BYTES.0,
            bin: Self::DEFAULT_BYTES.1,
        }
    }
}

impl <'a>Options<'a> {

    #[allow(unreachable_patterns)]
    const DEFAULT_BYTES: (&'static [u8], &'static [u8]) = match () {
        #[cfg(feature = "model-realesr-animevideov3")]
        () => MODEL_REALESR_ANIMEVIDEOV3_X4,
        #[cfg(feature = "model-realesrgan-plus")]
        () => MODEL_REALESRGAN_X4PLUS,
        #[cfg(feature = "model-realesrgan-plus-anime")]
        () => MODEL_REALESRGAN_X4PLUS_ANIME,
        _ => (&[], &[]),
    };

    #[cfg(any(feature = "model-realesr-animevideov3", feature = "model-realesrgan-plus", feature = "model-realesrgan-plus-anime"))]
    pub fn model(mut self, model: OptionsModel) -> Self {
        let (param, bin) = model.get_bytes();
        self.param = param;
        self.bin = bin;
        self.scale_factor = model.get_scale_factor();
        self
    }

    pub fn model_bytes(mut self, param: &'a [u8], bin: &'a [u8]) -> Self {
        self.param = param;
        self.bin = bin;
        self
    }

    pub fn model_files<P: AsRef<Path>>(mut self, param_file: P, bin_file: P) -> Result<Self, std::io::Error> {
        let param_file = std::fs::read(param_file)?;
        let bin_file = std::fs::read(bin_file)?;
        self.param = Box::leak(param_file.into_boxed_slice());
        self.bin = Box::leak(bin_file.into_boxed_slice());
        Ok(self)
    }

    pub fn gpuid(mut self, gpuid: u8) -> Self {
        self.gpuid = gpuid as i32;
        self
    }

    pub fn tta_mode(mut self, tta_mode: bool) -> Self {
        self.tta_mode = tta_mode;
        self
    }

    pub fn tilesize(mut self, tilesize: usize) -> Self {
        self.tilesize = tilesize as i32;
        self
    }

    pub fn scale_factor(mut self, scale_factor: OptionsScaleFactor) -> Self {
        self.scale_factor = scale_factor as i32;
        self
    }

}