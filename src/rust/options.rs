use std::path::Path;

pub struct Options<'a> {
    pub gpuid: u8,
    pub tta_mode: bool,
    pub tilesize: usize,
    pub scale_factor: u8,
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
            param: &[],
            bin: &[],
        }
    }
}

impl <'a>Options<'a> {

    #[cfg(any(feature = "model-realesr-animevideov3", feature = "model-realesrgan-plus", feature = "model-realesrgan-plus-anime"))]
    pub fn model(mut self, model: crate::models::Model) -> Self {
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
        self.gpuid = gpuid;
        self
    }

    pub fn tta_mode(mut self, tta_mode: bool) -> Self {
        self.tta_mode = tta_mode;
        self
    }

    pub fn tilesize(mut self, tilesize: usize) -> Self {
        self.tilesize = tilesize;
        self
    }

    pub fn scale_factor(mut self, scale_factor: u8) -> Self {
        self.scale_factor = scale_factor;
        self
    }

}