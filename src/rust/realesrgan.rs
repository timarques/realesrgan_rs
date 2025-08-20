use crate::Options;
use crate::Error;

use libc::{c_int, c_uchar, c_void, FILE};

extern "C" {
    fn realesrgan_init(
        gpuid: c_int,
        tta_mode: bool,
        scale: c_int,
        tilesize: c_int,
    ) -> *mut c_void;

    fn realesrgan_get_gpu_count() -> c_int;

    fn realesrgan_destroy_gpu_instance();

    fn realesrgan_free(realesrgan: *mut c_void);

    fn realesrgan_load_files(
        realesrgan: *mut c_void, 
        param_path: *mut FILE,
        model_path: *mut FILE
    ) -> c_int;

    fn realesrgan_process(
        realesrgan: *mut c_void,
        in_image: *const c_uchar,
        out_image: *mut c_uchar,
        width: c_int,
        height: c_int,
        channels: c_int,
    ) -> c_int;
}

#[derive(Debug)]
pub struct RealEsrgan<'a> {
    pointer: *mut c_void,
    options: Options<'a>
}

impl<'a> RealEsrgan<'a> {
    fn validate_gpu(gpu: i32) -> Result<(), Error> {
        if gpu == -1 {
            return Ok(());
        }
        
        let count = unsafe { realesrgan_get_gpu_count() };
        if gpu >= count {
            unsafe { realesrgan_destroy_gpu_instance(); }
            Err(Error::GpuNotFound {
                requested: gpu,
                available: count,
            })
        } else {
            Ok(())
        }
    }

    fn create_file_pointer(contents: &[u8]) -> *mut FILE {
        unsafe { 
            libc::fmemopen(
                contents.as_ptr() as *mut c_void,
                contents.len(),
                c"rb".as_ptr()
            )
        }
    }

    fn load_model(realesrgan: *mut c_void, param: &[u8], bin: &[u8]) -> Result<(), Error> {
        if param.is_empty() || bin.is_empty() {
            return Err(Error::InvalidModel);
        }

        let file_param_pointer = Self::create_file_pointer(param);
        let file_bin_pointer = Self::create_file_pointer(bin);

        if file_bin_pointer.is_null() || file_param_pointer.is_null() {
            if !file_param_pointer.is_null() {
                unsafe { libc::fclose(file_param_pointer); }
            }

            if !file_bin_pointer.is_null() { 
                unsafe { libc::fclose(file_bin_pointer); }
            }

            return Err(Error::FilePointerCreationFailed);
        }

        let result = unsafe {
            realesrgan_load_files(
                realesrgan,
                file_param_pointer,
                file_bin_pointer
            )
        };

        unsafe {
            libc::fclose(file_param_pointer);
            libc::fclose(file_bin_pointer);
        }

        if result != 0 {
            Err(Error::ModelLoadFailed { code: result })
        } else {
            Ok(())
        }
    }

    pub fn new(options: Options<'a>) -> Result<Self, Error> {
        Self::validate_gpu(options.gpuid)?;

        let pointer = unsafe {
            realesrgan_init(
                options.gpuid,
                options.tta_mode,
                options.scale_factor,
                options.tilesize
            )
        };

        if pointer.is_null() {
            return Err(Error::InitializationFailed);
        }

        Self::load_model(pointer, options.param, options.bin)?;

        Ok(Self {
            pointer,
            options,
        })
    }

    pub fn process(&self, input: &[u8], width: usize, height: usize) -> Result<Vec<u8>, Error> {
        if self.pointer.is_null() {
            return Err(Error::InvalidPointer);
        }

        let input_length = input.len();
        let expected_length = width * height;
        
        if input_length % expected_length != 0 {
            return Err(Error::InvalidInput {
                expected_length,
                actual_length: input_length
            });
        }

        let channels = input_length / expected_length;
        let output_length = (width * self.options.scale_factor as usize) 
                          * (height * self.options.scale_factor as usize) 
                          * channels;

        let mut output = vec![0u8; output_length];

        let code = unsafe {
            realesrgan_process(
                self.pointer,
                input.as_ptr(),
                output.as_mut_ptr(),
                width as c_int,
                height as c_int,
                channels as c_int,
            )
        };

        if code == 0 {
            Ok(output)
        } else {
            Err(Error::ProcessingFailed { code })
        }
    }

    pub fn process_batch<I, B>(
        &self,
        inputs: I,
        width: usize,
        height: usize,
    ) -> Result<Vec<Vec<u8>>, Error>
    where 
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        inputs
            .into_iter()
            .map(|input_chunk| self.process(input_chunk.as_ref(), width, height))
            .collect()
    }

    #[cfg(feature = "image")]
    pub fn process_file<P>(&self, path: P) -> Result<crate::Image, Error>
    where 
        P: AsRef<std::path::Path>,
    {
        let img = image::open(path).map_err(|e| Error::ImageOpenFailed(e.to_string()))?;
        self.process_image(img)
    }

    #[cfg(feature = "image")]
    pub fn process_image(&self, image: crate::Image) -> Result<crate::Image, Error> {
        use image::{ColorType, ImageBuffer, DynamicImage};

        let color_type = image.color();
        let input = image.to_rgb8().into_raw();
        let width = image.width();
        let height = image.height();
        let output = self.process(&input, width as usize, height as usize)?;
        let new_width = width * self.options.scale_factor as u32;
        let new_height = height * self.options.scale_factor as u32;
    
        let dynamic_image = match color_type {
            ColorType::Rgb8 => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageRgb8),
            ColorType::Rgba8 => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageRgba8),
            ColorType::L8 => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageLuma8),
            ColorType::La8 => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageLumaA8),
            _ => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageRgb8),
        };
    
        dynamic_image.ok_or(Error::ColorConversionFailed)
    }
}

impl Drop for RealEsrgan<'_> {
    fn drop(&mut self) {
        if !self.pointer.is_null() {
            unsafe { realesrgan_free(self.pointer) };
        }
    }
}