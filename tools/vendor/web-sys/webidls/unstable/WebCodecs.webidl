[Exposed=(Window,DedicatedWorker), SecureContext]
interface AudioDecoder {
  constructor(AudioDecoderInit init);

  readonly attribute CodecState state;
  readonly attribute unsigned long decodeQueueSize;

  undefined configure(AudioDecoderConfig config);
  undefined decode(EncodedAudioChunk chunk);
  Promise<undefined> flush();
  undefined reset();
  undefined close();

  static Promise<AudioDecoderSupport> isConfigSupported(AudioDecoderConfig config);
};

dictionary AudioDecoderInit {
  required AudioDataOutputCallback output;
  required WebCodecsErrorCallback error;
};

callback AudioDataOutputCallback = undefined(AudioData output);

[Exposed=(Window,DedicatedWorker), SecureContext]
interface VideoDecoder {
  constructor(VideoDecoderInit init);

  readonly attribute CodecState state;
  readonly attribute unsigned long decodeQueueSize;

  undefined configure(VideoDecoderConfig config);
  undefined decode(EncodedVideoChunk chunk);
  Promise<undefined> flush();
  undefined reset();
  undefined close();

  static Promise<VideoDecoderSupport> isConfigSupported(VideoDecoderConfig config);
};

dictionary VideoDecoderInit {
  required VideoFrameOutputCallback output;
  required WebCodecsErrorCallback error;
};

callback VideoFrameOutputCallback = undefined(VideoFrame output);

[Exposed=(Window,DedicatedWorker), SecureContext]
interface AudioEncoder {
  constructor(AudioEncoderInit init);

  readonly attribute CodecState state;
  readonly attribute unsigned long encodeQueueSize;

  undefined configure(AudioEncoderConfig config);
  undefined encode(AudioData data);
  Promise<undefined> flush();
  undefined reset();
  undefined close();

  static Promise<AudioEncoderSupport> isConfigSupported(AudioEncoderConfig config);
};

dictionary AudioEncoderInit {
  required EncodedAudioChunkOutputCallback output;
  required WebCodecsErrorCallback error;
};

callback EncodedAudioChunkOutputCallback =
    undefined (EncodedAudioChunk output,
               optional EncodedAudioChunkMetadata metadata = {});

dictionary EncodedAudioChunkMetadata {
  AudioDecoderConfig decoderConfig;
};

[Exposed=(Window,DedicatedWorker), SecureContext]
interface VideoEncoder {
  constructor(VideoEncoderInit init);

  readonly attribute CodecState state;
  readonly attribute unsigned long encodeQueueSize;

  undefined configure(VideoEncoderConfig config);
  undefined encode(VideoFrame frame, optional VideoEncoderEncodeOptions options = {});
  Promise<undefined> flush();
  undefined reset();
  undefined close();

  static Promise<VideoEncoderSupport> isConfigSupported(VideoEncoderConfig config);
};

dictionary VideoEncoderInit {
  required EncodedVideoChunkOutputCallback output;
  required WebCodecsErrorCallback error;
};

callback EncodedVideoChunkOutputCallback =
    undefined (EncodedVideoChunk chunk,
               optional EncodedVideoChunkMetadata metadata = {});

dictionary EncodedVideoChunkMetadata {
  VideoDecoderConfig decoderConfig;
  SvcOutputMetadata svc;
  BufferSource alphaSideData;
};

dictionary SvcOutputMetadata {
  unsigned long temporalLayerId;
};

dictionary AudioDecoderSupport {
  boolean supported;
  AudioDecoderConfig config;
};

dictionary VideoDecoderSupport {
  boolean supported;
  VideoDecoderConfig config;
};

dictionary AudioEncoderSupport {
  boolean supported;
  AudioEncoderConfig config;
};

dictionary VideoEncoderSupport {
  boolean supported;
  VideoEncoderConfig config;
};

dictionary AudioDecoderConfig {
  required DOMString codec;
  [EnforceRange] required unsigned long sampleRate;
  [EnforceRange] required unsigned long numberOfChannels;
  BufferSource description;
};

dictionary VideoDecoderConfig {
  required DOMString codec;
  BufferSource description;
  [EnforceRange] unsigned long codedWidth;
  [EnforceRange] unsigned long codedHeight;
  [EnforceRange] unsigned long displayAspectWidth;
  [EnforceRange] unsigned long displayAspectHeight;
  VideoColorSpaceInit colorSpace;
  HardwareAcceleration hardwareAcceleration = "no-preference";
  boolean optimizeForLatency;
};

dictionary AudioEncoderConfig {
  required DOMString codec;
  [EnforceRange] unsigned long sampleRate;
  [EnforceRange] unsigned long numberOfChannels;
  [EnforceRange] unsigned long long bitrate;
};

dictionary VideoEncoderConfig {
  required DOMString codec;
  [EnforceRange] required unsigned long width;
  [EnforceRange] required unsigned long height;
  [EnforceRange] unsigned long displayWidth;
  [EnforceRange] unsigned long displayHeight;
  [EnforceRange] unsigned long long bitrate;
  [EnforceRange] double framerate;
  HardwareAcceleration hardwareAcceleration = "no-preference";
  AlphaOption alpha = "discard";
  DOMString scalabilityMode;
  BitrateMode bitrateMode = "variable";
  LatencyMode latencyMode = "quality";
};

enum HardwareAcceleration {
  "no-preference",
  "prefer-hardware",
  "prefer-software",
};

enum AlphaOption {
  "keep",
  "discard",
};

enum LatencyMode {
  "quality",
  "realtime"
};

dictionary VideoEncoderEncodeOptions {
  boolean keyFrame = false;
};

enum CodecState {
  "unconfigured",
  "configured",
  "closed"
};

callback WebCodecsErrorCallback = undefined(DOMException error);

[Exposed=(Window,DedicatedWorker)]
interface EncodedAudioChunk {
  constructor(EncodedAudioChunkInit init);
  readonly attribute EncodedAudioChunkType type;
  readonly attribute long long timestamp;          // microseconds
  readonly attribute unsigned long long? duration; // microseconds
  readonly attribute unsigned long byteLength;

  undefined copyTo([AllowShared] BufferSource destination);
};

dictionary EncodedAudioChunkInit {
  required EncodedAudioChunkType type;
  [EnforceRange] required long long timestamp;    // microseconds
  [EnforceRange] unsigned long long duration;     // microseconds
  required BufferSource data;
};

enum EncodedAudioChunkType {
    "key",
    "delta",
};

[Exposed=(Window,DedicatedWorker)]
interface EncodedVideoChunk {
  constructor(EncodedVideoChunkInit init);
  readonly attribute EncodedVideoChunkType type;
  readonly attribute long long timestamp;             // microseconds
  readonly attribute unsigned long long? duration;    // microseconds
  readonly attribute unsigned long byteLength;

  undefined copyTo([AllowShared] BufferSource destination);
};

dictionary EncodedVideoChunkInit {
  required EncodedVideoChunkType type;
  [EnforceRange] required long long timestamp;        // microseconds
  [EnforceRange] unsigned long long duration;         // microseconds
  required BufferSource data;
};

enum EncodedVideoChunkType {
    "key",
    "delta",
};

[Exposed=(Window,DedicatedWorker), Serializable, Transferable]
interface AudioData {
  constructor(AudioDataInit init);

  readonly attribute AudioSampleFormat? format;
  readonly attribute float sampleRate;
  readonly attribute unsigned long numberOfFrames;
  readonly attribute unsigned long numberOfChannels;
  readonly attribute unsigned long long duration;  // microseconds
  readonly attribute long long timestamp;          // microseconds

  unsigned long allocationSize(AudioDataCopyToOptions options);
  undefined copyTo([AllowShared] BufferSource destination, AudioDataCopyToOptions options);
  AudioData clone();
  undefined close();
};

dictionary AudioDataInit {
  required AudioSampleFormat format;
  required float sampleRate;
  [EnforceRange] required unsigned long numberOfFrames;
  [EnforceRange] required unsigned long numberOfChannels;
  [EnforceRange] required long long timestamp;  // microseconds
  required BufferSource data;
};

dictionary AudioDataCopyToOptions {
  [EnforceRange] required unsigned long planeIndex;
  [EnforceRange] unsigned long frameOffset = 0;
  [EnforceRange] unsigned long frameCount;
  AudioSampleFormat format;
};

enum AudioSampleFormat {
  "u8",
  "s16",
  "s32",
  "f32",
  "u8-planar",
  "s16-planar",
  "s32-planar",
  "f32-planar",
};

[Exposed=(Window,DedicatedWorker), Serializable, Transferable]
interface VideoFrame {
  constructor(CanvasImageSource image, optional VideoFrameInit init = {});
  constructor([AllowShared] BufferSource data, VideoFrameBufferInit init);

  readonly attribute VideoPixelFormat? format;
  readonly attribute unsigned long codedWidth;
  readonly attribute unsigned long codedHeight;
  readonly attribute DOMRectReadOnly? codedRect;
  readonly attribute DOMRectReadOnly? visibleRect;
  readonly attribute unsigned long displayWidth;
  readonly attribute unsigned long displayHeight;
  readonly attribute unsigned long long? duration;  // microseconds
  readonly attribute long long? timestamp;          // microseconds
  readonly attribute VideoColorSpace colorSpace;

  unsigned long allocationSize(
      optional VideoFrameCopyToOptions options = {});
  Promise<sequence<PlaneLayout>> copyTo(
      [AllowShared] BufferSource destination,
      optional VideoFrameCopyToOptions options = {});
  VideoFrame clone();
  undefined close();
};

dictionary VideoFrameInit {
  unsigned long long duration;  // microseconds
  long long timestamp;          // microseconds
  AlphaOption alpha = "keep";

  // Default matches image. May be used to efficiently crop. Will trigger
  // new computation of displayWidth and displayHeight using image’s pixel
  // aspect ratio unless an explicit displayWidth and displayHeight are given.
  DOMRectInit visibleRect;

  // Default matches image unless visibleRect is provided.
  [EnforceRange] unsigned long displayWidth;
  [EnforceRange] unsigned long displayHeight;
};

dictionary VideoFrameBufferInit {
  required VideoPixelFormat format;
  [EnforceRange] required unsigned long codedWidth;
  [EnforceRange] required unsigned long codedHeight;
  [EnforceRange] required long long timestamp;  // microseconds
  [EnforceRange] unsigned long long duration;  // microseconds

  // Default layout is tightly-packed.
  sequence<PlaneLayout> layout;

  // Default visible rect is coded size positioned at (0,0)
  DOMRectInit visibleRect;

  // Default display dimensions match visibleRect.
  [EnforceRange] unsigned long displayWidth;
  [EnforceRange] unsigned long displayHeight;

  VideoColorSpaceInit colorSpace;
};

dictionary VideoFrameCopyToOptions {
  DOMRectInit rect;
  sequence<PlaneLayout> layout;
};

dictionary PlaneLayout {
  [EnforceRange] required unsigned long offset;
  [EnforceRange] required unsigned long stride;
};

enum VideoPixelFormat {
  // 4:2:0 Y, U, V
  "I420",
  // 4:2:0 Y, U, V, A
  "I420A",
  // 4:2:2 Y, U, V
  "I422",
  // 4:4:4 Y, U, V
  "I444",
  // 4:2:0 Y, UV
  "NV12",
  // 32bpp RGBA
  "RGBA",
  // 32bpp RGBX (opaque)
  "RGBX",
  // 32bpp BGRA
  "BGRA",
  // 32bpp BGRX (opaque)
  "BGRX",
};

[Exposed=(Window,DedicatedWorker)]
interface VideoColorSpace {
  constructor(optional VideoColorSpaceInit init = {});

  readonly attribute VideoColorPrimaries? primaries;
  readonly attribute VideoTransferCharacteristics? transfer;
  readonly attribute VideoMatrixCoefficients? matrix;
  readonly attribute boolean? fullRange;

  [Default] VideoColorSpaceInit toJSON();
};

dictionary VideoColorSpaceInit {
  VideoColorPrimaries primaries;
  VideoTransferCharacteristics transfer;
  VideoMatrixCoefficients matrix;
  boolean fullRange;
};

enum VideoColorPrimaries {
  "bt709",      // BT.709, sRGB
  "bt470bg",    // BT.601 PAL
  "smpte170m",  // BT.601 NTSC
};

enum VideoTransferCharacteristics {
  "bt709",         // BT.709
  "smpte170m",     // BT.601 (functionally the same as bt709)
  "iec61966-2-1",  // sRGB
};

enum VideoMatrixCoefficients {
  "rgb",        // sRGB
  "bt709",      // BT.709
  "bt470bg",    // BT.601 PAL
  "smpte170m",  // BT.601 NTSC (functionally the same as bt470bg)
};

[Exposed=(Window,DedicatedWorker), SecureContext]
interface ImageDecoder {
  constructor(ImageDecoderInit init);

  readonly attribute DOMString type;
  readonly attribute boolean complete;
  readonly attribute Promise<undefined> completed;
  readonly attribute ImageTrackList tracks;

  Promise<ImageDecodeResult> decode(optional ImageDecodeOptions options = {});
  undefined reset();
  undefined close();

  static Promise<boolean> isTypeSupported(DOMString type);
};


typedef (BufferSource or ReadableStream) ImageBufferSource;
dictionary ImageDecoderInit {
  required DOMString type;
  required ImageBufferSource data;
  PremultiplyAlpha premultiplyAlpha = "default";
  ColorSpaceConversion colorSpaceConversion = "default";
  [EnforceRange] unsigned long desiredWidth;
  [EnforceRange] unsigned long desiredHeight;
  boolean preferAnimation;
};


dictionary ImageDecodeOptions {
  [EnforceRange] unsigned long frameIndex = 0;
  boolean completeFramesOnly = true;
};


dictionary ImageDecodeResult {
  required VideoFrame image;
  required boolean complete;
};


[Exposed=(Window,DedicatedWorker)]
interface ImageTrackList {
  getter ImageTrack (unsigned long index);

  readonly attribute Promise<undefined> ready;
  readonly attribute unsigned long length;
  readonly attribute long selectedIndex;
  readonly attribute ImageTrack? selectedTrack;
};


[Exposed=(Window,DedicatedWorker)]
interface ImageTrack : EventTarget {
  readonly attribute boolean animated;
  readonly attribute unsigned long frameCount;
  readonly attribute unrestricted float repetitionCount;
  attribute EventHandler onchange;
  attribute boolean selected;
};
