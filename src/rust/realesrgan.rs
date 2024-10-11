use crate::Options;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Once;

use image::{ColorType, DynamicImage, ImageBuffer};
use libc::{c_char, c_int, c_uchar, c_void, FILE};

extern "C" {
    fn realesrgan_init(
        gpuid: c_int,
        tta_mode: bool,
    ) -> *mut c_void;

    fn realesrgan_set_parameters(
        realesrgan: *mut c_void,
        gpuid: c_int,
        scale: c_int,
        tilesize: c_int,
    );

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
        out_image: *const c_uchar,
        width: c_int,
        height: c_int,
        channels: c_int,
    ) -> c_int;
}


#[derive(Clone, Debug)]
pub struct RealEsrgan {
    pointer: Arc<AtomicPtr<c_void>>,
    scale_factor: u8,
}

impl RealEsrgan {

    fn validate_gpu(gpu: i32) -> Result<(), String> {
        if gpu == -1 {
            return Ok(())
        }
        let count = unsafe { realesrgan_get_gpu_count() };
        if gpu >= count {
            unsafe { realesrgan_destroy_gpu_instance() }
            return Err(format!("gpu {} not found. available gpus: {}", gpu, count))
        }
        Ok(())
    }

    fn create_file_pointer(contents: &[u8]) -> *mut FILE {
        let buffer = contents.as_ptr() as *mut c_void;
        let size = contents.len();
        
        unsafe { libc::fmemopen(buffer, size, "rb\0".as_ptr() as *const c_char) }
    }

    fn load_model(realcugan: *mut c_void, param: &[u8], bin: &[u8]) -> Result<(), String> {
        if param.len() == 0 || bin.len() == 0 {
            return Err(format!("invalid model"))
        }

        let file_bin_pointer = Self::create_file_pointer(bin);
        let file_param_pointer = Self::create_file_pointer(param);
        if file_bin_pointer.is_null() || file_param_pointer.is_null() {
            return Err(format!("failed to create file pointers"));
        }

        let result = unsafe { realesrgan_load_files(realcugan, file_param_pointer, file_bin_pointer) };
        if result != 0 {
            Err(format!("failed to load model files. error code: {}", result))
        } else {
            Ok(())
        }
    }

    fn clean_up() {
        static CLEANUP: Once = Once::new();
        CLEANUP.call_once(|| {
            extern "C" fn cleanup() {
                unsafe { realesrgan_destroy_gpu_instance() }
            }
            unsafe { libc::atexit(cleanup) };
        });
    }

    pub fn new(options: Options) -> Result<Self, String> {
        let gpuid = options.gpuid as i32;
        Self::validate_gpu(gpuid)?;
        let pointer = unsafe { realesrgan_init(gpuid, options.tta_mode) };
        Self::load_model(pointer, options.param, options.bin)?;
        unsafe { realesrgan_set_parameters(pointer, gpuid, options.scale_factor as i32, options.tilesize as i32) };
        Self::clean_up();

        Ok(Self {
            pointer: Arc::new(AtomicPtr::new(pointer)),
            scale_factor: options.scale_factor,
        })
    }

    pub fn process(&self, input: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
        let ptr = self.pointer.load(Ordering::Acquire);
        if ptr.is_null() {
            return Err(format!("invalid pointer"))
        }

        let input_length = input.len();
        let channels = input_length / (width * height);

        if input_length % (width * height) != 0 {
            return Err(format!("invalid input"))
        }

        let output_length = (width * self.scale_factor as usize) * (height * self.scale_factor as usize)  * channels;
        let mut output = vec![0u8; output_length];

        let result = unsafe {
            realesrgan_process(
                ptr,
                input.as_ptr(),
                output.as_mut_ptr(),
                width as c_int,
                height as c_int,
                channels as c_int,
            )
        };

        if result != 0 {
            return Err(format!("failed to process image"))
        }

        Ok(output)
    }

    #[cfg(feature = "image")]
    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<DynamicImage, String> {
        let img = image::open(path).map_err(|e| format!("failed to open image: {}", e))?;
        self.process_image(img)
    }

    #[cfg(feature = "image")]
    pub fn process_image(&self, image: DynamicImage) -> Result<DynamicImage, String> {
        let color_type = image.color();
        let input = image.to_rgb8().into_raw();
        let width = image.width();
        let height = image.height();
        let output = self.process(&input, width as usize, height as usize)?;
        let new_width = width * self.scale_factor as u32;
        let new_height = height * self.scale_factor as u32;
    
        let dynamic_image = match color_type {
            ColorType::Rgb8 => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageRgb8),
            ColorType::Rgba8 => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageRgba8),
            ColorType::L8 => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageLuma8),
            ColorType::La8 => image::ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageLumaA8),
            _ => ImageBuffer::from_raw(new_width, new_height, output).map(DynamicImage::ImageRgb8),
        };
    
        Ok(dynamic_image.ok_or(format!("failed to convert color type"))?)
    }

}

impl Drop for RealEsrgan {
    fn drop(&mut self) {
        if Arc::strong_count(&self.pointer) == 1 {
            let ptr = self.pointer.load(Ordering::Acquire);
            if !ptr.is_null() {
                unsafe { realesrgan_free(ptr) }
            }
        }
    }
}

unsafe impl Send for RealEsrgan {}
unsafe impl Sync for RealEsrgan {}