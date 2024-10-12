// realesrgan implemented with ncnn library

#ifndef REALESRGAN_H
#define REALESRGAN_H

// ncnn
#include "net.h"
#include "gpu.h"
#include "layer.h"

class RealESRGAN
{
public:
    RealESRGAN(int gpuid, bool tta_mode = false);
    ~RealESRGAN();

    int load_files(FILE *param, FILE *bin);

    int process(const ncnn::Mat& inimage, ncnn::Mat& outimage) const;

public:
    // realesrgan parameters
    int scale;
    int tilesize;
    int prepadding;

private:
    ncnn::Net net;
    ncnn::Pipeline* realesrgan_preproc;
    ncnn::Pipeline* realesrgan_postproc;
    ncnn::Layer* bicubic_2x;
    ncnn::Layer* bicubic_3x;
    ncnn::Layer* bicubic_4x;
    bool tta_mode;
};

#endif // REALESRGAN_H
