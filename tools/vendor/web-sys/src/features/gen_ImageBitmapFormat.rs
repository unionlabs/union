#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `ImageBitmapFormat` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `ImageBitmapFormat`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageBitmapFormat {
    Rgba32 = "RGBA32",
    Bgra32 = "BGRA32",
    Rgb24 = "RGB24",
    Bgr24 = "BGR24",
    Gray8 = "GRAY8",
    Yuv444p = "YUV444P",
    Yuv422p = "YUV422P",
    Yuv420p = "YUV420P",
    Yuv420spNv12 = "YUV420SP_NV12",
    Yuv420spNv21 = "YUV420SP_NV21",
    Hsv = "HSV",
    Lab = "Lab",
    Depth = "DEPTH",
}
