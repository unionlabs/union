/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/webappapis.html#images
 *
 * The origin of the extended IDL file is
 * http://w3c.github.io/mediacapture-worker/#imagebitmap-extensions
 */

// Extensions
// Bug 1141979 - [FoxEye] Extend ImageBitmap with interfaces to access its
// underlying image data
//
// Note:
// Our overload resolution implementation doesn't deal with a union as the
// distinguishing argument which means we cannot overload functions via union
// types, a.k.a. we cannot overload createImageBitmap() via ImageBitmapSource
// and BufferSource. Here, we work around this issue by adding the BufferSource
// into ImageBitmapSource.

typedef (HTMLImageElement or
         HTMLVideoElement or
         HTMLCanvasElement or
         Blob or
         ImageData or
         CanvasRenderingContext2D or
         ImageBitmap or
         BufferSource) ImageBitmapSource;

[Exposed=(Window,Worker)]
interface ImageBitmap {
  [Constant]
  readonly attribute unsigned long width;
  [Constant]
  readonly attribute unsigned long height;
};

// It's crucial that there be a way to explicitly dispose of ImageBitmaps
// since they refer to potentially large graphics resources. Some uses
// of this API proposal will result in repeated allocations of ImageBitmaps,
// and garbage collection will not reliably reclaim them quickly enough.
// Here we reuse close(), which also exists on another Transferable type,
// MessagePort. Potentially, all Transferable types should inherit from a
// new interface type "Closeable".
partial interface ImageBitmap {
  // Dispose of all graphical resources associated with this ImageBitmap.
  undefined close();
};

// ImageBitmap-extensions
// Bug 1141979 - [FoxEye] Extend ImageBitmap with interfaces to access its
// underlying image data

/*
 * An image or a video frame is conceptually a two-dimensional array of data and
 * each element in the array is called a pixel. The pixels are usually stored in
 * a one-dimensional array and could be arranged in a variety of image formats.
 * Developers need to know how the pixels are formatted so that they are able to
 * process them.
 *
 * The image format describes how pixels in an image are arranged. A single
 * pixel has at least one, but usually multiple pixel values. The range of a
 * pixel value varies, which means different image formats use different data
 * types to store a single pixel value.
 *
 * The most frequently used data type is 8-bit unsigned integer whose range is
 * from 0 to 255, others could be 16-bit integer or 32-bit floating points and
 * so forth. The number of pixel values of a single pixel is called the number
 * of channels of the image format. Multiple pixel values of a pixel are used
 * together to describe the captured property which could be color or depth
 * information. For example, if the data is a color image in RGB color space,
 * then it is a three-channel image format and a pixel is described by R, G and
 * B three pixel values with range from 0 to 255. As another example, if the
 * data is a gray image, then it is a single-channel image format with 8-bit
 * unsigned integer data type and the pixel value describes the gray scale. For
 * depth data, it is a single channel image format too, but the data type is
 * 16-bit unsigned integer and the pixel value is the depth level.
 *
 * For those image formats whose pixels contain multiple pixel values, the pixel
 * values might be arranged in one of the following ways:
 * 1) Planar pixel layout:
 *    each channel has its pixel values stored consecutively in separated
 *    buffers (a.k.a. planes) and then all channel buffers are stored
 *    consecutively in memory.
 *    (Ex: RRRRRR......GGGGGG......BBBBBB......)
 * 2) Interleaving pixel layout:
 *    each pixel has its pixel values from all channels stored together and
 *    interleaves all channels.
 *    (Ex: RGBRGBRGBRGBRGB......)
 */


/*
 * The ImageBitmap extensions use this enumeration to negotiate the image format
 * while 1) accessing the underlying data of an ImageBitmap and
 *       2) creating a new ImageBitmap.
 *
 * For each format in this enumeration, we use a 2x2 small image (4 pixels) as
 * example to illustrate the pixel layout.
 *
 * 2x2 image:   +--------+--------+
 *              | pixel1 | pixel2 |
 *              +--------+--------+
 *              | pixel3 | pixel4 |
 *              +--------+--------+
 *
 */
enum ImageBitmapFormat {
  /*
   * Channel order: R, G, B, A
   * Channel size: full rgba-chennels
   * Pixel layout: interleaving rgba-channels
   * Pixel layout illustration:
   *   [Plane1]: R1 G1 B1 A1 R2 G2 B2 A2 R3 G3 B3 A3 R4 G4 B4 A4
   * Data type: 8-bit unsigned integer
   */
  "RGBA32",

  /*
   * Channel order: B, G, R, A
   * Channel size: full bgra-channels
   * Pixel layout: interleaving bgra-channels
   * Pixel layout illustration:
   *   [Plane1]: B1 G1 R1 A1 B2 G2 R2 A2 B3 G3 R3 A3 B4 G4 R4 A4
   * Data type: 8-bit unsigned integer
   */
  "BGRA32",

  /*
   * Channel order: R, G, B
   * Channel size: full rgb-channels
   * Pixel layout: interleaving rgb-channels
   * Pixel layout illustration:
   *   [Plane1]: R1 G1 B1 R2 G2 B2 R3 G3 B3 R4 G4 B4
   * Data type: 8-bit unsigned integer
   */
  "RGB24",

  /*
   * Channel order: B, G, R
   * Channel size: full bgr-channels
   * Pixel layout: interleaving bgr-channels
   * Pixel layout illustration:
   *   [Plane1]: B1 G1 R1 B2 G2 R2 B3 G3 R3 B4 G4 R4
   * Data type: 8-bit unsigned integer
   */
  "BGR24",

  /*
   * Channel order: GRAY
   * Channel size: full gray-channel
   * Pixel layout: planar gray-channel
   * Pixel layout illustration:
   *   [Plane1]: GRAY1 GRAY2 GRAY3 GRAY4
   * Data type: 8-bit unsigned integer
   */
  "GRAY8",

  /*
   * Channel order: Y, U, V
   * Channel size: full yuv-channels
   * Pixel layout: planar yuv-channels
   * Pixel layout illustration:
   *   [Plane1]: Y1 Y2 Y3 Y4
   *   [Plane2]: U1 U2 U3 U4
   *   [Plane3]: V1 V2 V3 V4
   * Data type: 8-bit unsigned integer
   */
  "YUV444P",

  /*
   * Channel order: Y, U, V
   * Channel size: full y-channel, half uv-channels
   * Pixel layout: planar yuv-channels
   * Pixel layout illustration:
   *   [Plane1]: Y1 Y2 Y3 Y4
   *   [Plane2]: U1 U3
   *   [Plane3]: V1 V3
   * Data type: 8-bit unsigned integer
   */
  "YUV422P",

  /*
   * Channel order: Y, U, V
   * Channel size: full y-channel, quarter uv-channels
   * Pixel layout: planar yuv-channels
   * Pixel layout illustration:
   *   [Plane1]: Y1 Y2 Y3 Y4
   *   [Plane2]: U1
   *   [Plane3]: V1
   * Data type: 8-bit unsigned integer
   */
  "YUV420P",

  /*
   * Channel order: Y, U, V
   * Channel size: full y-channel, quarter uv-channels
   * Pixel layout: planar y-channel, interleaving uv-channels
   * Pixel layout illustration:
   *   [Plane1]: Y1 Y2 Y3 Y4
   *   [Plane2]: U1 V1
   * Data type: 8-bit unsigned integer
   */
  "YUV420SP_NV12",

  /*
   * Channel order: Y, V, U
   * Channel size: full y-channel, quarter vu-channels
   * Pixel layout: planar y-channel, interleaving vu-channels
   * Pixel layout illustration:
   *   [Plane1]: Y1 Y2 Y3 Y4
   *   [Plane2]: V1 U1
   * Data type: 8-bit unsigned integer
   */
  "YUV420SP_NV21",

  /*
   * Channel order: H, S, V
   * Channel size: full hsv-channels
   * Pixel layout: interleaving hsv-channels
   * Pixel layout illustration:
   *   [Plane1]: H1 S1 V1 H2 S2 V2 H3 S3 V3
   * Data type: 32-bit floating point value
   */
  "HSV",

  /*
   * Channel order: l, a, b
   * Channel size: full lab-channels
   * Pixel layout: interleaving lab-channels
   * Pixel layout illustration:
   *   [Plane1]: l1 a1 b1 l2 a2 b2 l3 a3 b3
   * Data type: 32-bit floating point value
   */
  "Lab",

  /*
   * Channel order: DEPTH
   * Channel size: full depth-channel
   * Pixel layout: planar depth-channel
   * Pixel layout illustration:
   *   [Plane1]: DEPTH1 DEPTH2 DEPTH3 DEPTH4
   * Data type: 16-bit unsigned integer
   */
  "DEPTH",
};

enum ChannelPixelLayoutDataType {
  "uint8",
  "int8",
  "uint16",
  "int16",
  "uint32",
  "int32",
  "float32",
  "float64"
};

/*
 * Two concepts, ImagePixelLayout and ChannelPixelLayout, together generalize
 * the variety of pixel layouts among image formats.
 *
 * The ChannelPixelLayout represents the pixel layout of a single channel in a
 * certain image format and the ImagePixelLayout is just the collection of
 * ChannelPixelLayouts. So, the ChannelPixelLayout is defined as a dictionary
 * type with properties to describe the layout and the ImagePixelLayout is just
 * an alias name to a sequence of ChannelPixelLayout objects.
 *
 * Since an image format is composed of at least one channel, an
 * ImagePixelLayout object contains at least one ChannelPixelLayout object.
 *
 * Although an image or a video frame is a two-dimensional structure, its data
 * is usually stored in a one-dimensional array in the row-major way and the
 * ChannelPixelLayout objects use the following properties to describe the
 * layout of pixel values in the buffer.
 *
 * 1) offset:
 *    denotes the beginning position of the channel's data relative to the
 *    beginning position of the one-dimensional array.
 * 2) width & height:
 *    denote the width and height of the channel respectively. Each channel in
 *    an image format may have different height and width.
 * 3) data type:
 *    denotes the format used to store one single pixel value.
 * 4) stride:
 *    the number of bytes between the beginning two consecutive rows in memory.
 *    (The total bytes of each row plus the padding bytes of each raw.)
 * 5) skip value:
 *    the value is zero for the planar pixel layout, and a positive integer for
 *    the interleaving pixel layout. (Describes how many bytes there are between
 *    two adjacent pixel values in this channel.)
 */

/*
 * Example1: RGBA image, width = 620, height = 480, stride = 2560
 *
 * chanel_r: offset = 0, width = 620, height = 480, data type = uint8, stride = 2560, skip = 3
 * chanel_g: offset = 1, width = 620, height = 480, data type = uint8, stride = 2560, skip = 3
 * chanel_b: offset = 2, width = 620, height = 480, data type = uint8, stride = 2560, skip = 3
 * chanel_a: offset = 3, width = 620, height = 480, data type = uint8, stride = 2560, skip = 3
 *
 *         <---------------------------- stride ---------------------------->
 *         <---------------------- width x 4 ---------------------->
 * [index] 01234   8   12  16  20  24  28                           2479    2559
 *         |||||---|---|---|---|---|---|----------------------------|-------|
 * [data]  RGBARGBARGBARGBARGBAR___R___R...                         A%%%%%%%%
 * [data]  RGBARGBARGBARGBARGBAR___R___R...                         A%%%%%%%%
 * [data]  RGBARGBARGBARGBARGBAR___R___R...                         A%%%%%%%%
 *              ^^^
 *              r-skip
 */

/*
 * Example2: YUV420P image, width = 620, height = 480, stride = 640
 *
 * chanel_y: offset = 0, width = 620, height = 480, stride = 640, skip = 0
 * chanel_u: offset = 307200, width = 310, height = 240, data type = uint8, stride = 320, skip = 0
 * chanel_v: offset = 384000, width = 310, height = 240, data type = uint8, stride = 320, skip = 0
 *
 *         <--------------------------- y-stride --------------------------->
 *         <----------------------- y-width ----------------------->
 * [index] 012345                                                  619      639
 *         ||||||--------------------------------------------------|--------|
 * [data]  YYYYYYYYYYYYYYYYYYYYYYYYYYYYY...                        Y%%%%%%%%%
 * [data]  YYYYYYYYYYYYYYYYYYYYYYYYYYYYY...                        Y%%%%%%%%%
 * [data]  YYYYYYYYYYYYYYYYYYYYYYYYYYYYY...                        Y%%%%%%%%%
 * [data]  ......
 *         <-------- u-stride ---------->
 *         <----- u-width ----->
 * [index] 307200              307509   307519
 *         |-------------------|--------|
 * [data]  UUUUUUUUUU...       U%%%%%%%%%
 * [data]  UUUUUUUUUU...       U%%%%%%%%%
 * [data]  UUUUUUUUUU...       U%%%%%%%%%
 * [data]  ......
 *         <-------- v-stride ---------->
 *         <- --- v-width ----->
 * [index] 384000              384309   384319
 *         |-------------------|--------|
 * [data]  VVVVVVVVVV...       V%%%%%%%%%
 * [data]  VVVVVVVVVV...       V%%%%%%%%%
 * [data]  VVVVVVVVVV...       V%%%%%%%%%
 * [data]  ......
 */

/*
 * Example3: YUV420SP_NV12 image, width = 620, height = 480, stride = 640
 *
 * chanel_y: offset = 0, width = 620, height = 480, stride = 640, skip = 0
 * chanel_u: offset = 307200, width = 310, height = 240, data type = uint8, stride = 640, skip = 1
 * chanel_v: offset = 307201, width = 310, height = 240, data type = uint8, stride = 640, skip = 1
 *
 *         <--------------------------- y-stride -------------------------->
 *         <----------------------- y-width ---------------------->
 * [index] 012345                                                 619      639
 *         ||||||-------------------------------------------------|--------|
 * [data]  YYYYYYYYYYYYYYYYYYYYYYYYYYYYY...                       Y%%%%%%%%%
 * [data]  YYYYYYYYYYYYYYYYYYYYYYYYYYYYY...                       Y%%%%%%%%%
 * [data]  YYYYYYYYYYYYYYYYYYYYYYYYYYYYY...                       Y%%%%%%%%%
 * [data]  ......
 *         <--------------------- u-stride / v-stride -------------------->
 *         <------------------ u-width + v-width ----------------->
 * [index] 307200(u-offset)                                       307819  307839
 *         |------------------------------------------------------|-------|
 * [index] |307201(v-offset)                                      |307820 |
 *         ||-----------------------------------------------------||------|
 * [data]  UVUVUVUVUVUVUVUVUVUVUVUVUVUVUV...                      UV%%%%%%%
 * [data]  UVUVUVUVUVUVUVUVUVUVUVUVUVUVUV...                      UV%%%%%%%
 * [data]  UVUVUVUVUVUVUVUVUVUVUVUVUVUVUV...                      UV%%%%%%%
 *          ^            ^
 *         u-skip        v-skip
 */

/*
 * Example4: DEPTH image, width = 640, height = 480, stride = 1280
 *
 * chanel_d: offset = 0, width = 640, height = 480, data type = uint16, stride = 1280, skip = 0
 *
 * note: each DEPTH value uses two bytes
 *
 *         <----------------------- d-stride ---------------------->
 *         <----------------------- d-width ----------------------->
 * [index] 02468                                                   1278
 *         |||||---------------------------------------------------|
 * [data]  DDDDDDDDDDDDDDDDDDDDDDDDDDDDD...                        D
 * [data]  DDDDDDDDDDDDDDDDDDDDDDDDDDDDD...                        D
 * [data]  DDDDDDDDDDDDDDDDDDDDDDDDDDDDD...                        D
 * [data]  ......
 */

dictionary ChannelPixelLayout {
    required unsigned long              offset;
    required unsigned long              width;
    required unsigned long              height;
    required ChannelPixelLayoutDataType dataType;
    required unsigned long              stride;
    required unsigned long              skip;
};

typedef sequence<ChannelPixelLayout> ImagePixelLayout;

partial interface ImageBitmap {
    [Throws, Func="mozilla::dom::DOMPrefs::ImageBitmapExtensionsEnabled"]
    ImageBitmapFormat               findOptimalFormat (optional sequence<ImageBitmapFormat> aPossibleFormats);
    [Throws, Func="mozilla::dom::DOMPrefs::ImageBitmapExtensionsEnabled"]
    long                            mappedDataLength (ImageBitmapFormat aFormat);
    [Throws, Func="mozilla::dom::DOMPrefs::ImageBitmapExtensionsEnabled"]
    Promise<ImagePixelLayout> mapDataInto (ImageBitmapFormat aFormat, BufferSource aBuffer, long aOffset);
};
