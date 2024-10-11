#include "realesrgan.h"

#include <algorithm>
#include <vector>
#include <map>

// ncnn
#include "gpu.h"

extern "C" RealESRGAN *realesrgan_init(int gpuid, bool tta_mode) {
    return new RealESRGAN(gpuid, tta_mode);
}

extern "C" int realesrgan_get_gpu_count() {
    return ncnn::get_gpu_count();
}

extern "C" int realesrgan_load_files(
    RealESRGAN *realesrgan,
    FILE* param,
    FILE* bin
) {
    return realesrgan->load_files(param, bin);
}

extern "C" void realesrgan_set_parameters(
    RealESRGAN *realesrgan,
    int gpuid,
    int scale,
    int tilesize
) {
    if (tilesize == 0) {
        uint32_t heap_budget = ncnn::get_gpu_device(gpuid)->get_heap_budget();
        if (heap_budget > 1900) {
            tilesize = 200;
        } else if (heap_budget > 550) {
            tilesize = 100;
        } else if (heap_budget > 190) {
            tilesize = 64;
        } else {
            tilesize = 32;
        }
    }
    realesrgan->tilesize = tilesize;
    realesrgan->scale = scale;
    realesrgan->prepadding = 10;
}

extern "C" int realesrgan_process(
    RealESRGAN *realesrgan,
    unsigned char *input_data,
    unsigned char *output_data,
    int width,
    int height,
    int channels
) {
    ncnn::Mat in_image_mat = ncnn::Mat(width, height, (void *)input_data, (size_t)channels, channels);
    ncnn::Mat out_image_mat = ncnn::Mat(width * realesrgan->scale, height * realesrgan->scale, (void *)output_data, (size_t)channels, channels);
    return realesrgan->process(in_image_mat, out_image_mat);
}

extern "C" void realesrgan_free(RealESRGAN *realesrgan) {
    delete realesrgan;
}

extern "C" void realesrgan_destroy_gpu_instance() {
    ncnn::destroy_gpu_instance();
}