/* -*- Mode: IDL; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://www.khronos.org/registry/webgl/specs/latest/webgl.idl
 *
 * Copyright © 2012 Khronos Group
 */

// WebGL IDL definitions scraped from the Khronos specification:
// https://www.khronos.org/registry/webgl/specs/latest/
//
// This IDL depends on the typed array specification defined at:
// https://www.khronos.org/registry/typedarray/specs/latest/typedarrays.idl

typedef unsigned long  GLenum;
typedef boolean        GLboolean;
typedef unsigned long  GLbitfield;
typedef byte           GLbyte;         /* 'byte' should be a signed 8 bit type. */
typedef short          GLshort;
typedef long           GLint;
typedef long           GLsizei;
typedef long long      GLintptr;
typedef long long      GLsizeiptr;
// Ideally the typedef below would use 'unsigned byte', but that doesn't currently exist in Web IDL.
typedef octet          GLubyte;        /* 'octet' should be an unsigned 8 bit type. */
typedef unsigned short GLushort;
typedef unsigned long  GLuint;
typedef unrestricted float GLfloat;
typedef unrestricted float GLclampf;
typedef unsigned long long GLuint64EXT;

// The power preference settings are documented in the WebGLContextAttributes
// section of the specification.
enum WebGLPowerPreference { "default", "low-power", "high-performance" };

dictionary WebGLContextAttributes {
    // boolean alpha = true;
    // We deviate from the spec here.
    // If alpha isn't specified, we rely on a pref ("webgl.default-no-alpha")
    GLboolean alpha;
    GLboolean depth = true;
    GLboolean stencil = false;
    GLboolean antialias = true;
    GLboolean premultipliedAlpha = true;
    GLboolean preserveDrawingBuffer = false;
    GLboolean failIfMajorPerformanceCaveat = false;
    WebGLPowerPreference powerPreference = "default";
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLBuffer {
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLFramebuffer {
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLProgram {
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLRenderbuffer {
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLShader {
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLTexture {
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLUniformLocation {
};

interface WebGLVertexArrayObject {
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLActiveInfo {
    readonly attribute GLint size;
    readonly attribute GLenum type;
    readonly attribute DOMString name;
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLShaderPrecisionFormat {
    readonly attribute GLint rangeMin;
    readonly attribute GLint rangeMax;
    readonly attribute GLint precision;
};

typedef (Float32Array or sequence<GLfloat>) Float32List;
typedef (Int32Array or sequence<GLint>) Int32List;

// Shared interface for the things that WebGLRenderingContext and
// WebGL2RenderingContext have in common.  This doesn't have all the things they
// have in common, because we don't support splitting multiple overloads of the
// same method across separate interfaces and pulling them in with "implements".
interface mixin WebGLRenderingContextBase {
    /* ClearBufferMask */
    const GLenum DEPTH_BUFFER_BIT               = 0x00000100;
    const GLenum STENCIL_BUFFER_BIT             = 0x00000400;
    const GLenum COLOR_BUFFER_BIT               = 0x00004000;

    /* BeginMode */
    const GLenum POINTS                         = 0x0000;
    const GLenum LINES                          = 0x0001;
    const GLenum LINE_LOOP                      = 0x0002;
    const GLenum LINE_STRIP                     = 0x0003;
    const GLenum TRIANGLES                      = 0x0004;
    const GLenum TRIANGLE_STRIP                 = 0x0005;
    const GLenum TRIANGLE_FAN                   = 0x0006;

    /* AlphaFunction (not supported in ES20) */
    /*      NEVER */
    /*      LESS */
    /*      EQUAL */
    /*      LEQUAL */
    /*      GREATER */
    /*      NOTEQUAL */
    /*      GEQUAL */
    /*      ALWAYS */

    /* BlendingFactorDest */
    const GLenum ZERO                           = 0;
    const GLenum ONE                            = 1;
    const GLenum SRC_COLOR                      = 0x0300;
    const GLenum ONE_MINUS_SRC_COLOR            = 0x0301;
    const GLenum SRC_ALPHA                      = 0x0302;
    const GLenum ONE_MINUS_SRC_ALPHA            = 0x0303;
    const GLenum DST_ALPHA                      = 0x0304;
    const GLenum ONE_MINUS_DST_ALPHA            = 0x0305;

    /* BlendingFactorSrc */
    /*      ZERO */
    /*      ONE */
    const GLenum DST_COLOR                      = 0x0306;
    const GLenum ONE_MINUS_DST_COLOR            = 0x0307;
    const GLenum SRC_ALPHA_SATURATE             = 0x0308;
    /*      SRC_ALPHA */
    /*      ONE_MINUS_SRC_ALPHA */
    /*      DST_ALPHA */
    /*      ONE_MINUS_DST_ALPHA */

    /* BlendEquationSeparate */
    const GLenum FUNC_ADD                       = 0x8006;
    const GLenum BLEND_EQUATION                 = 0x8009;
    const GLenum BLEND_EQUATION_RGB             = 0x8009;   /* same as BLEND_EQUATION */
    const GLenum BLEND_EQUATION_ALPHA           = 0x883D;

    /* BlendSubtract */
    const GLenum FUNC_SUBTRACT                  = 0x800A;
    const GLenum FUNC_REVERSE_SUBTRACT          = 0x800B;

    /* Separate Blend Functions */
    const GLenum BLEND_DST_RGB                  = 0x80C8;
    const GLenum BLEND_SRC_RGB                  = 0x80C9;
    const GLenum BLEND_DST_ALPHA                = 0x80CA;
    const GLenum BLEND_SRC_ALPHA                = 0x80CB;
    const GLenum CONSTANT_COLOR                 = 0x8001;
    const GLenum ONE_MINUS_CONSTANT_COLOR       = 0x8002;
    const GLenum CONSTANT_ALPHA                 = 0x8003;
    const GLenum ONE_MINUS_CONSTANT_ALPHA       = 0x8004;
    const GLenum BLEND_COLOR                    = 0x8005;

    /* Buffer Objects */
    const GLenum ARRAY_BUFFER                   = 0x8892;
    const GLenum ELEMENT_ARRAY_BUFFER           = 0x8893;
    const GLenum ARRAY_BUFFER_BINDING           = 0x8894;
    const GLenum ELEMENT_ARRAY_BUFFER_BINDING   = 0x8895;

    const GLenum STREAM_DRAW                    = 0x88E0;
    const GLenum STATIC_DRAW                    = 0x88E4;
    const GLenum DYNAMIC_DRAW                   = 0x88E8;

    const GLenum BUFFER_SIZE                    = 0x8764;
    const GLenum BUFFER_USAGE                   = 0x8765;

    const GLenum CURRENT_VERTEX_ATTRIB          = 0x8626;

    /* CullFaceMode */
    const GLenum FRONT                          = 0x0404;
    const GLenum BACK                           = 0x0405;
    const GLenum FRONT_AND_BACK                 = 0x0408;

    /* DepthFunction */
    /*      NEVER */
    /*      LESS */
    /*      EQUAL */
    /*      LEQUAL */
    /*      GREATER */
    /*      NOTEQUAL */
    /*      GEQUAL */
    /*      ALWAYS */

    /* EnableCap */
    /* TEXTURE_2D */
    const GLenum CULL_FACE                      = 0x0B44;
    const GLenum BLEND                          = 0x0BE2;
    const GLenum DITHER                         = 0x0BD0;
    const GLenum STENCIL_TEST                   = 0x0B90;
    const GLenum DEPTH_TEST                     = 0x0B71;
    const GLenum SCISSOR_TEST                   = 0x0C11;
    const GLenum POLYGON_OFFSET_FILL            = 0x8037;
    const GLenum SAMPLE_ALPHA_TO_COVERAGE       = 0x809E;
    const GLenum SAMPLE_COVERAGE                = 0x80A0;

    /* ErrorCode */
    [NeedsWindowsUndef]
    const GLenum NO_ERROR                       = 0;
    const GLenum INVALID_ENUM                   = 0x0500;
    const GLenum INVALID_VALUE                  = 0x0501;
    const GLenum INVALID_OPERATION              = 0x0502;
    const GLenum OUT_OF_MEMORY                  = 0x0505;

    /* FrontFaceDirection */
    const GLenum CW                             = 0x0900;
    const GLenum CCW                            = 0x0901;

    /* GetPName */
    const GLenum LINE_WIDTH                     = 0x0B21;
    const GLenum ALIASED_POINT_SIZE_RANGE       = 0x846D;
    const GLenum ALIASED_LINE_WIDTH_RANGE       = 0x846E;
    const GLenum CULL_FACE_MODE                 = 0x0B45;
    const GLenum FRONT_FACE                     = 0x0B46;
    const GLenum DEPTH_RANGE                    = 0x0B70;
    const GLenum DEPTH_WRITEMASK                = 0x0B72;
    const GLenum DEPTH_CLEAR_VALUE              = 0x0B73;
    const GLenum DEPTH_FUNC                     = 0x0B74;
    const GLenum STENCIL_CLEAR_VALUE            = 0x0B91;
    const GLenum STENCIL_FUNC                   = 0x0B92;
    const GLenum STENCIL_FAIL                   = 0x0B94;
    const GLenum STENCIL_PASS_DEPTH_FAIL        = 0x0B95;
    const GLenum STENCIL_PASS_DEPTH_PASS        = 0x0B96;
    const GLenum STENCIL_REF                    = 0x0B97;
    const GLenum STENCIL_VALUE_MASK             = 0x0B93;
    const GLenum STENCIL_WRITEMASK              = 0x0B98;
    const GLenum STENCIL_BACK_FUNC              = 0x8800;
    const GLenum STENCIL_BACK_FAIL              = 0x8801;
    const GLenum STENCIL_BACK_PASS_DEPTH_FAIL   = 0x8802;
    const GLenum STENCIL_BACK_PASS_DEPTH_PASS   = 0x8803;
    const GLenum STENCIL_BACK_REF               = 0x8CA3;
    const GLenum STENCIL_BACK_VALUE_MASK        = 0x8CA4;
    const GLenum STENCIL_BACK_WRITEMASK         = 0x8CA5;
    const GLenum VIEWPORT                       = 0x0BA2;
    const GLenum SCISSOR_BOX                    = 0x0C10;
    /*      SCISSOR_TEST */
    const GLenum COLOR_CLEAR_VALUE              = 0x0C22;
    const GLenum COLOR_WRITEMASK                = 0x0C23;
    const GLenum UNPACK_ALIGNMENT               = 0x0CF5;
    const GLenum PACK_ALIGNMENT                 = 0x0D05;
    const GLenum MAX_TEXTURE_SIZE               = 0x0D33;
    const GLenum MAX_VIEWPORT_DIMS              = 0x0D3A;
    const GLenum SUBPIXEL_BITS                  = 0x0D50;
    const GLenum RED_BITS                       = 0x0D52;
    const GLenum GREEN_BITS                     = 0x0D53;
    const GLenum BLUE_BITS                      = 0x0D54;
    const GLenum ALPHA_BITS                     = 0x0D55;
    const GLenum DEPTH_BITS                     = 0x0D56;
    const GLenum STENCIL_BITS                   = 0x0D57;
    const GLenum POLYGON_OFFSET_UNITS           = 0x2A00;
    /*      POLYGON_OFFSET_FILL */
    const GLenum POLYGON_OFFSET_FACTOR          = 0x8038;
    const GLenum TEXTURE_BINDING_2D             = 0x8069;
    const GLenum SAMPLE_BUFFERS                 = 0x80A8;
    const GLenum SAMPLES                        = 0x80A9;
    const GLenum SAMPLE_COVERAGE_VALUE          = 0x80AA;
    const GLenum SAMPLE_COVERAGE_INVERT         = 0x80AB;

    /* GetTextureParameter */
    /*      TEXTURE_MAG_FILTER */
    /*      TEXTURE_MIN_FILTER */
    /*      TEXTURE_WRAP_S */
    /*      TEXTURE_WRAP_T */

    const GLenum COMPRESSED_TEXTURE_FORMATS     = 0x86A3;

    /* HintMode */
    const GLenum DONT_CARE                      = 0x1100;
    const GLenum FASTEST                        = 0x1101;
    const GLenum NICEST                         = 0x1102;

    /* HintTarget */
    const GLenum GENERATE_MIPMAP_HINT            = 0x8192;

    /* DataType */
    const GLenum BYTE                           = 0x1400;
    const GLenum UNSIGNED_BYTE                  = 0x1401;
    const GLenum SHORT                          = 0x1402;
    const GLenum UNSIGNED_SHORT                 = 0x1403;
    const GLenum INT                            = 0x1404;
    const GLenum UNSIGNED_INT                   = 0x1405;
    const GLenum FLOAT                          = 0x1406;

    /* PixelFormat */
    const GLenum DEPTH_COMPONENT                = 0x1902;
    const GLenum ALPHA                          = 0x1906;
    const GLenum RGB                            = 0x1907;
    const GLenum RGBA                           = 0x1908;
    const GLenum LUMINANCE                      = 0x1909;
    const GLenum LUMINANCE_ALPHA                = 0x190A;

    /* PixelType */
    /*      UNSIGNED_BYTE */
    const GLenum UNSIGNED_SHORT_4_4_4_4         = 0x8033;
    const GLenum UNSIGNED_SHORT_5_5_5_1         = 0x8034;
    const GLenum UNSIGNED_SHORT_5_6_5           = 0x8363;

    /* Shaders */
    const GLenum FRAGMENT_SHADER                  = 0x8B30;
    const GLenum VERTEX_SHADER                    = 0x8B31;
    const GLenum MAX_VERTEX_ATTRIBS               = 0x8869;
    const GLenum MAX_VERTEX_UNIFORM_VECTORS       = 0x8DFB;
    const GLenum MAX_VARYING_VECTORS              = 0x8DFC;
    const GLenum MAX_COMBINED_TEXTURE_IMAGE_UNITS = 0x8B4D;
    const GLenum MAX_VERTEX_TEXTURE_IMAGE_UNITS   = 0x8B4C;
    const GLenum MAX_TEXTURE_IMAGE_UNITS          = 0x8872;
    const GLenum MAX_FRAGMENT_UNIFORM_VECTORS     = 0x8DFD;
    const GLenum SHADER_TYPE                      = 0x8B4F;
    const GLenum DELETE_STATUS                    = 0x8B80;
    const GLenum LINK_STATUS                      = 0x8B82;
    const GLenum VALIDATE_STATUS                  = 0x8B83;
    const GLenum ATTACHED_SHADERS                 = 0x8B85;
    const GLenum ACTIVE_UNIFORMS                  = 0x8B86;
    const GLenum ACTIVE_ATTRIBUTES                = 0x8B89;
    const GLenum SHADING_LANGUAGE_VERSION         = 0x8B8C;
    const GLenum CURRENT_PROGRAM                  = 0x8B8D;

    /* StencilFunction */
    const GLenum NEVER                          = 0x0200;
    const GLenum LESS                           = 0x0201;
    const GLenum EQUAL                          = 0x0202;
    const GLenum LEQUAL                         = 0x0203;
    const GLenum GREATER                        = 0x0204;
    const GLenum NOTEQUAL                       = 0x0205;
    const GLenum GEQUAL                         = 0x0206;
    const GLenum ALWAYS                         = 0x0207;

    /* StencilOp */
    /*      ZERO */
    const GLenum KEEP                           = 0x1E00;
    const GLenum REPLACE                        = 0x1E01;
    const GLenum INCR                           = 0x1E02;
    const GLenum DECR                           = 0x1E03;
    const GLenum INVERT                         = 0x150A;
    const GLenum INCR_WRAP                      = 0x8507;
    const GLenum DECR_WRAP                      = 0x8508;

    /* StringName */
    const GLenum VENDOR                         = 0x1F00;
    const GLenum RENDERER                       = 0x1F01;
    const GLenum VERSION                        = 0x1F02;

    /* TextureMagFilter */
    const GLenum NEAREST                        = 0x2600;
    const GLenum LINEAR                         = 0x2601;

    /* TextureMinFilter */
    /*      NEAREST */
    /*      LINEAR */
    const GLenum NEAREST_MIPMAP_NEAREST         = 0x2700;
    const GLenum LINEAR_MIPMAP_NEAREST          = 0x2701;
    const GLenum NEAREST_MIPMAP_LINEAR          = 0x2702;
    const GLenum LINEAR_MIPMAP_LINEAR           = 0x2703;

    /* TextureParameterName */
    const GLenum TEXTURE_MAG_FILTER             = 0x2800;
    const GLenum TEXTURE_MIN_FILTER             = 0x2801;
    const GLenum TEXTURE_WRAP_S                 = 0x2802;
    const GLenum TEXTURE_WRAP_T                 = 0x2803;

    /* TextureTarget */
    const GLenum TEXTURE_2D                     = 0x0DE1;
    const GLenum TEXTURE                        = 0x1702;

    const GLenum TEXTURE_CUBE_MAP               = 0x8513;
    const GLenum TEXTURE_BINDING_CUBE_MAP       = 0x8514;
    const GLenum TEXTURE_CUBE_MAP_POSITIVE_X    = 0x8515;
    const GLenum TEXTURE_CUBE_MAP_NEGATIVE_X    = 0x8516;
    const GLenum TEXTURE_CUBE_MAP_POSITIVE_Y    = 0x8517;
    const GLenum TEXTURE_CUBE_MAP_NEGATIVE_Y    = 0x8518;
    const GLenum TEXTURE_CUBE_MAP_POSITIVE_Z    = 0x8519;
    const GLenum TEXTURE_CUBE_MAP_NEGATIVE_Z    = 0x851A;
    const GLenum MAX_CUBE_MAP_TEXTURE_SIZE      = 0x851C;

    /* TextureUnit */
    const GLenum TEXTURE0                       = 0x84C0;
    const GLenum TEXTURE1                       = 0x84C1;
    const GLenum TEXTURE2                       = 0x84C2;
    const GLenum TEXTURE3                       = 0x84C3;
    const GLenum TEXTURE4                       = 0x84C4;
    const GLenum TEXTURE5                       = 0x84C5;
    const GLenum TEXTURE6                       = 0x84C6;
    const GLenum TEXTURE7                       = 0x84C7;
    const GLenum TEXTURE8                       = 0x84C8;
    const GLenum TEXTURE9                       = 0x84C9;
    const GLenum TEXTURE10                      = 0x84CA;
    const GLenum TEXTURE11                      = 0x84CB;
    const GLenum TEXTURE12                      = 0x84CC;
    const GLenum TEXTURE13                      = 0x84CD;
    const GLenum TEXTURE14                      = 0x84CE;
    const GLenum TEXTURE15                      = 0x84CF;
    const GLenum TEXTURE16                      = 0x84D0;
    const GLenum TEXTURE17                      = 0x84D1;
    const GLenum TEXTURE18                      = 0x84D2;
    const GLenum TEXTURE19                      = 0x84D3;
    const GLenum TEXTURE20                      = 0x84D4;
    const GLenum TEXTURE21                      = 0x84D5;
    const GLenum TEXTURE22                      = 0x84D6;
    const GLenum TEXTURE23                      = 0x84D7;
    const GLenum TEXTURE24                      = 0x84D8;
    const GLenum TEXTURE25                      = 0x84D9;
    const GLenum TEXTURE26                      = 0x84DA;
    const GLenum TEXTURE27                      = 0x84DB;
    const GLenum TEXTURE28                      = 0x84DC;
    const GLenum TEXTURE29                      = 0x84DD;
    const GLenum TEXTURE30                      = 0x84DE;
    const GLenum TEXTURE31                      = 0x84DF;
    const GLenum ACTIVE_TEXTURE                 = 0x84E0;

    /* TextureWrapMode */
    const GLenum REPEAT                         = 0x2901;
    const GLenum CLAMP_TO_EDGE                  = 0x812F;
    const GLenum MIRRORED_REPEAT                = 0x8370;

    /* Uniform Types */
    const GLenum FLOAT_VEC2                     = 0x8B50;
    const GLenum FLOAT_VEC3                     = 0x8B51;
    const GLenum FLOAT_VEC4                     = 0x8B52;
    const GLenum INT_VEC2                       = 0x8B53;
    const GLenum INT_VEC3                       = 0x8B54;
    const GLenum INT_VEC4                       = 0x8B55;
    const GLenum BOOL                           = 0x8B56;
    const GLenum BOOL_VEC2                      = 0x8B57;
    const GLenum BOOL_VEC3                      = 0x8B58;
    const GLenum BOOL_VEC4                      = 0x8B59;
    const GLenum FLOAT_MAT2                     = 0x8B5A;
    const GLenum FLOAT_MAT3                     = 0x8B5B;
    const GLenum FLOAT_MAT4                     = 0x8B5C;
    const GLenum SAMPLER_2D                     = 0x8B5E;
    const GLenum SAMPLER_CUBE                   = 0x8B60;

    /* Vertex Arrays */
    const GLenum VERTEX_ATTRIB_ARRAY_ENABLED        = 0x8622;
    const GLenum VERTEX_ATTRIB_ARRAY_SIZE           = 0x8623;
    const GLenum VERTEX_ATTRIB_ARRAY_STRIDE         = 0x8624;
    const GLenum VERTEX_ATTRIB_ARRAY_TYPE           = 0x8625;
    const GLenum VERTEX_ATTRIB_ARRAY_NORMALIZED     = 0x886A;
    const GLenum VERTEX_ATTRIB_ARRAY_POINTER        = 0x8645;
    const GLenum VERTEX_ATTRIB_ARRAY_BUFFER_BINDING = 0x889F;

    /* Read Format */
    const GLenum IMPLEMENTATION_COLOR_READ_TYPE   = 0x8B9A;
    const GLenum IMPLEMENTATION_COLOR_READ_FORMAT = 0x8B9B;

    /* Shader Source */
    const GLenum COMPILE_STATUS                 = 0x8B81;

    /* Shader Precision-Specified Types */
    const GLenum LOW_FLOAT                      = 0x8DF0;
    const GLenum MEDIUM_FLOAT                   = 0x8DF1;
    const GLenum HIGH_FLOAT                     = 0x8DF2;
    const GLenum LOW_INT                        = 0x8DF3;
    const GLenum MEDIUM_INT                     = 0x8DF4;
    const GLenum HIGH_INT                       = 0x8DF5;

    /* Framebuffer Object. */
    const GLenum FRAMEBUFFER                    = 0x8D40;
    const GLenum RENDERBUFFER                   = 0x8D41;

    const GLenum RGBA4                          = 0x8056;
    const GLenum RGB5_A1                        = 0x8057;
    const GLenum RGB565                         = 0x8D62;
    const GLenum DEPTH_COMPONENT16              = 0x81A5;
    const GLenum STENCIL_INDEX8                 = 0x8D48;
    const GLenum DEPTH_STENCIL                  = 0x84F9;

    const GLenum RENDERBUFFER_WIDTH             = 0x8D42;
    const GLenum RENDERBUFFER_HEIGHT            = 0x8D43;
    const GLenum RENDERBUFFER_INTERNAL_FORMAT   = 0x8D44;
    const GLenum RENDERBUFFER_RED_SIZE          = 0x8D50;
    const GLenum RENDERBUFFER_GREEN_SIZE        = 0x8D51;
    const GLenum RENDERBUFFER_BLUE_SIZE         = 0x8D52;
    const GLenum RENDERBUFFER_ALPHA_SIZE        = 0x8D53;
    const GLenum RENDERBUFFER_DEPTH_SIZE        = 0x8D54;
    const GLenum RENDERBUFFER_STENCIL_SIZE      = 0x8D55;

    const GLenum FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE           = 0x8CD0;
    const GLenum FRAMEBUFFER_ATTACHMENT_OBJECT_NAME           = 0x8CD1;
    const GLenum FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL         = 0x8CD2;
    const GLenum FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE = 0x8CD3;

    const GLenum COLOR_ATTACHMENT0              = 0x8CE0;
    const GLenum DEPTH_ATTACHMENT               = 0x8D00;
    const GLenum STENCIL_ATTACHMENT             = 0x8D20;
    const GLenum DEPTH_STENCIL_ATTACHMENT       = 0x821A;

    const GLenum NONE                           = 0;

    const GLenum FRAMEBUFFER_COMPLETE                      = 0x8CD5;
    const GLenum FRAMEBUFFER_INCOMPLETE_ATTACHMENT         = 0x8CD6;
    const GLenum FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT = 0x8CD7;
    const GLenum FRAMEBUFFER_INCOMPLETE_DIMENSIONS         = 0x8CD9;
    const GLenum FRAMEBUFFER_UNSUPPORTED                   = 0x8CDD;

    const GLenum FRAMEBUFFER_BINDING            = 0x8CA6;
    const GLenum RENDERBUFFER_BINDING           = 0x8CA7;
    const GLenum MAX_RENDERBUFFER_SIZE          = 0x84E8;

    const GLenum INVALID_FRAMEBUFFER_OPERATION  = 0x0506;

    /* WebGL-specific enums */
    const GLenum UNPACK_FLIP_Y_WEBGL            = 0x9240;
    const GLenum UNPACK_PREMULTIPLY_ALPHA_WEBGL = 0x9241;
    const GLenum CONTEXT_LOST_WEBGL             = 0x9242;
    const GLenum UNPACK_COLORSPACE_CONVERSION_WEBGL = 0x9243;
    const GLenum BROWSER_DEFAULT_WEBGL          = 0x9244;

    // The canvas might actually be null in some cases, apparently.
    readonly attribute (HTMLCanvasElement or OffscreenCanvas)? canvas;
    readonly attribute GLsizei drawingBufferWidth;
    readonly attribute GLsizei drawingBufferHeight;

    [WebGLHandlesContextLoss] WebGLContextAttributes? getContextAttributes();
    [WebGLHandlesContextLoss] boolean isContextLost();

    [NeedsCallerType]
    sequence<DOMString>? getSupportedExtensions();

    [Throws, NeedsCallerType]
    object? getExtension(DOMString name);

    undefined activeTexture(GLenum texture);
    undefined attachShader(WebGLProgram program, WebGLShader shader);
    undefined bindAttribLocation(WebGLProgram program, GLuint index, DOMString name);
    undefined bindBuffer(GLenum target, WebGLBuffer? buffer);
    undefined bindFramebuffer(GLenum target, WebGLFramebuffer? framebuffer);
    undefined bindRenderbuffer(GLenum target, WebGLRenderbuffer? renderbuffer);
    undefined bindTexture(GLenum target, WebGLTexture? texture);
    undefined blendColor(GLfloat red, GLfloat green, GLfloat blue, GLfloat alpha);
    undefined blendEquation(GLenum mode);
    undefined blendEquationSeparate(GLenum modeRGB, GLenum modeAlpha);
    undefined blendFunc(GLenum sfactor, GLenum dfactor);
    undefined blendFuncSeparate(GLenum srcRGB, GLenum dstRGB,
                           GLenum srcAlpha, GLenum dstAlpha);

    [WebGLHandlesContextLoss] GLenum checkFramebufferStatus(GLenum target);
    undefined clear(GLbitfield mask);
    undefined clearColor(GLfloat red, GLfloat green, GLfloat blue, GLfloat alpha);
    undefined clearDepth(GLclampf depth);
    undefined clearStencil(GLint s);
    undefined colorMask(GLboolean red, GLboolean green, GLboolean blue, GLboolean alpha);
    undefined compileShader(WebGLShader shader);

    undefined copyTexImage2D(GLenum target, GLint level, GLenum internalformat,
                        GLint x, GLint y, GLsizei width, GLsizei height,
                        GLint border);
    undefined copyTexSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                           GLint x, GLint y, GLsizei width, GLsizei height);

    WebGLBuffer? createBuffer();
    WebGLFramebuffer? createFramebuffer();
    WebGLProgram? createProgram();
    WebGLRenderbuffer? createRenderbuffer();
    WebGLShader? createShader(GLenum type);
    WebGLTexture? createTexture();

    undefined cullFace(GLenum mode);

    undefined deleteBuffer(WebGLBuffer? buffer);
    undefined deleteFramebuffer(WebGLFramebuffer? framebuffer);
    undefined deleteProgram(WebGLProgram? program);
    undefined deleteRenderbuffer(WebGLRenderbuffer? renderbuffer);
    undefined deleteShader(WebGLShader? shader);
    undefined deleteTexture(WebGLTexture? texture);

    undefined depthFunc(GLenum func);
    undefined depthMask(GLboolean flag);
    undefined depthRange(GLclampf zNear, GLclampf zFar);
    undefined detachShader(WebGLProgram program, WebGLShader shader);
    undefined disable(GLenum cap);
    undefined disableVertexAttribArray(GLuint index);
    undefined drawArrays(GLenum mode, GLint first, GLsizei count);
    undefined drawElements(GLenum mode, GLsizei count, GLenum type, GLintptr offset);

    undefined enable(GLenum cap);
    undefined enableVertexAttribArray(GLuint index);
    undefined finish();
    undefined flush();
    undefined framebufferRenderbuffer(GLenum target, GLenum attachment,
                                 GLenum renderbuffertarget,
                                 WebGLRenderbuffer? renderbuffer);
    undefined framebufferTexture2D(GLenum target, GLenum attachment, GLenum textarget,
                              WebGLTexture? texture, GLint level);
    undefined frontFace(GLenum mode);

    undefined generateMipmap(GLenum target);

    [NewObject]
    WebGLActiveInfo? getActiveAttrib(WebGLProgram program, GLuint index);
    [NewObject]
    WebGLActiveInfo? getActiveUniform(WebGLProgram program, GLuint index);

    sequence<WebGLShader>? getAttachedShaders(WebGLProgram program);

    [WebGLHandlesContextLoss] GLint getAttribLocation(WebGLProgram program, DOMString name);

    any getBufferParameter(GLenum target, GLenum pname);
    [Throws]
    any getParameter(GLenum pname);

    [WebGLHandlesContextLoss] GLenum getError();

    [Throws]
    any getFramebufferAttachmentParameter(GLenum target, GLenum attachment,
                                          GLenum pname);
    any getProgramParameter(WebGLProgram program, GLenum pname);
    DOMString? getProgramInfoLog(WebGLProgram program);
    any getRenderbufferParameter(GLenum target, GLenum pname);
    any getShaderParameter(WebGLShader shader, GLenum pname);

    [NewObject]
    WebGLShaderPrecisionFormat? getShaderPrecisionFormat(GLenum shadertype, GLenum precisiontype);

    DOMString? getShaderInfoLog(WebGLShader shader);

    DOMString? getShaderSource(WebGLShader shader);

    any getTexParameter(GLenum target, GLenum pname);

    any getUniform(WebGLProgram program, WebGLUniformLocation location);

    [NewObject]
    WebGLUniformLocation? getUniformLocation(WebGLProgram program, DOMString name);

    [Throws]
    any getVertexAttrib(GLuint index, GLenum pname);

    [WebGLHandlesContextLoss] GLintptr getVertexAttribOffset(GLuint index, GLenum pname);

    undefined hint(GLenum target, GLenum mode);
    [WebGLHandlesContextLoss] GLboolean isBuffer(WebGLBuffer? buffer);
    [WebGLHandlesContextLoss] GLboolean isEnabled(GLenum cap);
    [WebGLHandlesContextLoss] GLboolean isFramebuffer(WebGLFramebuffer? framebuffer);
    [WebGLHandlesContextLoss] GLboolean isProgram(WebGLProgram? program);
    [WebGLHandlesContextLoss] GLboolean isRenderbuffer(WebGLRenderbuffer? renderbuffer);
    [WebGLHandlesContextLoss] GLboolean isShader(WebGLShader? shader);
    [WebGLHandlesContextLoss] GLboolean isTexture(WebGLTexture? texture);
    undefined lineWidth(GLfloat width);
    undefined linkProgram(WebGLProgram program);
    undefined pixelStorei(GLenum pname, GLint param);
    undefined polygonOffset(GLfloat factor, GLfloat units);

    undefined renderbufferStorage(GLenum target, GLenum internalformat,
                             GLsizei width, GLsizei height);
    undefined sampleCoverage(GLclampf value, GLboolean invert);
    undefined scissor(GLint x, GLint y, GLsizei width, GLsizei height);

    undefined shaderSource(WebGLShader shader, DOMString source);

    undefined stencilFunc(GLenum func, GLint ref, GLuint mask);
    undefined stencilFuncSeparate(GLenum face, GLenum func, GLint ref, GLuint mask);
    undefined stencilMask(GLuint mask);
    undefined stencilMaskSeparate(GLenum face, GLuint mask);
    undefined stencilOp(GLenum fail, GLenum zfail, GLenum zpass);
    undefined stencilOpSeparate(GLenum face, GLenum fail, GLenum zfail, GLenum zpass);

    undefined texParameterf(GLenum target, GLenum pname, GLfloat param);
    undefined texParameteri(GLenum target, GLenum pname, GLint param);

    undefined uniform1f(WebGLUniformLocation? location, GLfloat x);
    undefined uniform2f(WebGLUniformLocation? location, GLfloat x, GLfloat y);
    undefined uniform3f(WebGLUniformLocation? location, GLfloat x, GLfloat y, GLfloat z);
    undefined uniform4f(WebGLUniformLocation? location, GLfloat x, GLfloat y, GLfloat z, GLfloat w);

    undefined uniform1i(WebGLUniformLocation? location, GLint x);
    undefined uniform2i(WebGLUniformLocation? location, GLint x, GLint y);
    undefined uniform3i(WebGLUniformLocation? location, GLint x, GLint y, GLint z);
    undefined uniform4i(WebGLUniformLocation? location, GLint x, GLint y, GLint z, GLint w);

    undefined useProgram(WebGLProgram? program);
    undefined validateProgram(WebGLProgram program);

    undefined vertexAttrib1f(GLuint indx, GLfloat x);
    undefined vertexAttrib1fv(GLuint indx, Float32List values);
    undefined vertexAttrib2f(GLuint indx, GLfloat x, GLfloat y);
    undefined vertexAttrib2fv(GLuint indx, Float32List values);
    undefined vertexAttrib3f(GLuint indx, GLfloat x, GLfloat y, GLfloat z);
    undefined vertexAttrib3fv(GLuint indx, Float32List values);
    undefined vertexAttrib4f(GLuint indx, GLfloat x, GLfloat y, GLfloat z, GLfloat w);
    undefined vertexAttrib4fv(GLuint indx, Float32List values);
    undefined vertexAttribPointer(GLuint indx, GLint size, GLenum type,
                             GLboolean normalized, GLsizei stride, GLintptr offset);

    undefined viewport(GLint x, GLint y, GLsizei width, GLsizei height);
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLRenderingContext {
    // bufferData has WebGL2 overloads.
    undefined bufferData(GLenum target, GLsizeiptr size, GLenum usage);
    undefined bufferData(GLenum target, ArrayBuffer? data, GLenum usage);
    undefined bufferData(GLenum target, ArrayBufferView data, GLenum usage);
    // bufferSubData has WebGL2 overloads.
    undefined bufferSubData(GLenum target, GLintptr offset, ArrayBuffer data);
    undefined bufferSubData(GLenum target, GLintptr offset, ArrayBufferView data);

    // compressedTexImage2D has WebGL2 overloads.
    undefined compressedTexImage2D(GLenum target, GLint level, GLenum internalformat,
                              GLsizei width, GLsizei height, GLint border,
                              ArrayBufferView data);
    // compressedTexSubImage2D has WebGL2 overloads.
    undefined compressedTexSubImage2D(GLenum target, GLint level,
                                 GLint xoffset, GLint yoffset,
                                 GLsizei width, GLsizei height, GLenum format,
                                 ArrayBufferView data);

    // readPixels has WebGL2 overloads.
    [Throws, NeedsCallerType]
    undefined readPixels(GLint x, GLint y, GLsizei width, GLsizei height,
                    GLenum format, GLenum type, ArrayBufferView? pixels);

    // texImage2D has WebGL2 overloads.
    // Overloads must share [Throws].
    [Throws] // Can't actually throw.
    undefined texImage2D(GLenum target, GLint level, GLint internalformat,
                    GLsizei width, GLsizei height, GLint border, GLenum format,
                    GLenum type, ArrayBufferView? pixels);
    [Throws] // Can't actually throw.
    undefined texImage2D(GLenum target, GLint level, GLint internalformat,
                    GLenum format, GLenum type, ImageBitmap pixels);
    [Throws] // Can't actually throw.
    undefined texImage2D(GLenum target, GLint level, GLint internalformat,
                    GLenum format, GLenum type, ImageData pixels);
    [Throws]
    undefined texImage2D(GLenum target, GLint level, GLint internalformat,
                    GLenum format, GLenum type, HTMLImageElement image); // May throw DOMException
    [Throws]
    undefined texImage2D(GLenum target, GLint level, GLint internalformat,
                    GLenum format, GLenum type, HTMLCanvasElement canvas); // May throw DOMException
    [Throws]
    undefined texImage2D(GLenum target, GLint level, GLint internalformat,
                    GLenum format, GLenum type, HTMLVideoElement video); // May throw DOMException

    // texSubImage2D has WebGL2 overloads.
    [Throws] // Can't actually throw.
    undefined texSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                       GLsizei width, GLsizei height,
                       GLenum format, GLenum type, ArrayBufferView? pixels);
    [Throws] // Can't actually throw.
    undefined texSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                       GLenum format, GLenum type, ImageBitmap pixels);
    [Throws] // Can't actually throw.
    undefined texSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                       GLenum format, GLenum type, ImageData pixels);
    [Throws]
    undefined texSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                       GLenum format, GLenum type, HTMLImageElement image); // May throw DOMException
    [Throws]
    undefined texSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                       GLenum format, GLenum type, HTMLCanvasElement canvas); // May throw DOMException
    [Throws]
    undefined texSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                       GLenum format, GLenum type, HTMLVideoElement video); // May throw DOMException

    // uniform*fv have WebGL2 overloads, or rather extensions, that are not
    // distinguishable from the WebGL1 versions when called with two arguments.
    undefined uniform1fv(WebGLUniformLocation? location, Float32List data);
    undefined uniform2fv(WebGLUniformLocation? location, Float32List data);
    undefined uniform3fv(WebGLUniformLocation? location, Float32List data);
    undefined uniform4fv(WebGLUniformLocation? location, Float32List data);

    // uniform*iv have WebGL2 overloads, or rather extensions, that are not
    // distinguishable from the WebGL1 versions when called with two arguments.
    undefined uniform1iv(WebGLUniformLocation? location, Int32List data);
    undefined uniform2iv(WebGLUniformLocation? location, Int32List data);
    undefined uniform3iv(WebGLUniformLocation? location, Int32List data);
    undefined uniform4iv(WebGLUniformLocation? location, Int32List data);

    // uniformMatrix*fv have WebGL2 overloads, or rather extensions, that are
    // not distinguishable from the WebGL1 versions when called with two
    // arguments.
    undefined uniformMatrix2fv(WebGLUniformLocation? location, GLboolean transpose, Float32List data);
    undefined uniformMatrix3fv(WebGLUniformLocation? location, GLboolean transpose, Float32List data);
    undefined uniformMatrix4fv(WebGLUniformLocation? location, GLboolean transpose, Float32List data);
};

WebGLRenderingContext includes WebGLRenderingContextBase;

// For OffscreenCanvas
// Reference: https://wiki.whatwg.org/wiki/OffscreenCanvas
[Exposed=(Window,Worker)]
partial interface WebGLRenderingContext {
    [Func="mozilla::dom::DOMPrefs::OffscreenCanvasEnabled"]
    undefined commit();
};

////////////////////////////////////////
// specific extension interfaces

[NoInterfaceObject]
interface WEBGL_compressed_texture_s3tc
{
    const GLenum COMPRESSED_RGB_S3TC_DXT1_EXT  = 0x83F0;
    const GLenum COMPRESSED_RGBA_S3TC_DXT1_EXT = 0x83F1;
    const GLenum COMPRESSED_RGBA_S3TC_DXT3_EXT = 0x83F2;
    const GLenum COMPRESSED_RGBA_S3TC_DXT5_EXT = 0x83F3;
};

[NoInterfaceObject]
interface WEBGL_compressed_texture_s3tc_srgb {
    /* Compressed Texture Formats */
    const GLenum COMPRESSED_SRGB_S3TC_DXT1_EXT        = 0x8C4C;
    const GLenum COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT  = 0x8C4D;
    const GLenum COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT  = 0x8C4E;
    const GLenum COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT  = 0x8C4F;
};

[NoInterfaceObject]
interface WEBGL_compressed_texture_astc {
    /* Compressed Texture Format */
    const GLenum COMPRESSED_RGBA_ASTC_4x4_KHR = 0x93B0;
    const GLenum COMPRESSED_RGBA_ASTC_5x4_KHR = 0x93B1;
    const GLenum COMPRESSED_RGBA_ASTC_5x5_KHR = 0x93B2;
    const GLenum COMPRESSED_RGBA_ASTC_6x5_KHR = 0x93B3;
    const GLenum COMPRESSED_RGBA_ASTC_6x6_KHR = 0x93B4;
    const GLenum COMPRESSED_RGBA_ASTC_8x5_KHR = 0x93B5;
    const GLenum COMPRESSED_RGBA_ASTC_8x6_KHR = 0x93B6;
    const GLenum COMPRESSED_RGBA_ASTC_8x8_KHR = 0x93B7;
    const GLenum COMPRESSED_RGBA_ASTC_10x5_KHR = 0x93B8;
    const GLenum COMPRESSED_RGBA_ASTC_10x6_KHR = 0x93B9;
    const GLenum COMPRESSED_RGBA_ASTC_10x8_KHR = 0x93BA;
    const GLenum COMPRESSED_RGBA_ASTC_10x10_KHR = 0x93BB;
    const GLenum COMPRESSED_RGBA_ASTC_12x10_KHR = 0x93BC;
    const GLenum COMPRESSED_RGBA_ASTC_12x12_KHR = 0x93BD;

    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR = 0x93D0;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR = 0x93D1;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR = 0x93D2;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR = 0x93D3;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR = 0x93D4;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR = 0x93D5;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR = 0x93D6;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR = 0x93D7;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR = 0x93D8;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR = 0x93D9;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR = 0x93DA;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR = 0x93DB;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR = 0x93DC;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR = 0x93DD;

    // Profile query support.
    sequence<DOMString>? getSupportedProfiles();
};

[NoInterfaceObject]
interface WEBGL_compressed_texture_atc
{
    const GLenum COMPRESSED_RGB_ATC_WEBGL                     = 0x8C92;
    const GLenum COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL     = 0x8C93;
    const GLenum COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL = 0x87EE;
};

[NoInterfaceObject]
interface WEBGL_compressed_texture_etc
{
    const GLenum COMPRESSED_R11_EAC                                 = 0x9270;
    const GLenum COMPRESSED_SIGNED_R11_EAC                          = 0x9271;
    const GLenum COMPRESSED_RG11_EAC                                = 0x9272;
    const GLenum COMPRESSED_SIGNED_RG11_EAC                         = 0x9273;
    const GLenum COMPRESSED_RGB8_ETC2                               = 0x9274;
    const GLenum COMPRESSED_SRGB8_ETC2                              = 0x9275;
    const GLenum COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2           = 0x9276;
    const GLenum COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2          = 0x9277;
    const GLenum COMPRESSED_RGBA8_ETC2_EAC                          = 0x9278;
    const GLenum COMPRESSED_SRGB8_ALPHA8_ETC2_EAC                   = 0x9279;
};

[NoInterfaceObject]
interface WEBGL_compressed_texture_etc1
{
    const GLenum COMPRESSED_RGB_ETC1_WEBGL = 0x8D64;
};

[NoInterfaceObject]
interface WEBGL_compressed_texture_pvrtc
{
    const GLenum COMPRESSED_RGB_PVRTC_4BPPV1_IMG  = 0x8C00;
    const GLenum COMPRESSED_RGB_PVRTC_2BPPV1_IMG  = 0x8C01;
    const GLenum COMPRESSED_RGBA_PVRTC_4BPPV1_IMG = 0x8C02;
    const GLenum COMPRESSED_RGBA_PVRTC_2BPPV1_IMG = 0x8C03;
};

[NoInterfaceObject]
interface WEBGL_debug_renderer_info
{
    const GLenum UNMASKED_VENDOR_WEBGL        = 0x9245;
    const GLenum UNMASKED_RENDERER_WEBGL      = 0x9246;
};

[NoInterfaceObject]
interface WEBGL_debug_shaders
{
    DOMString getTranslatedShaderSource(WebGLShader shader);
};

[NoInterfaceObject]
interface WEBGL_depth_texture
{
    const GLenum UNSIGNED_INT_24_8_WEBGL = 0x84FA;
};

[NoInterfaceObject]
interface OES_element_index_uint
{
};

[NoInterfaceObject]
interface EXT_frag_depth
{
};

[NoInterfaceObject]
interface WEBGL_lose_context {
    undefined loseContext();
    undefined restoreContext();
};

[NoInterfaceObject]
interface EXT_texture_filter_anisotropic
{
    const GLenum TEXTURE_MAX_ANISOTROPY_EXT     = 0x84FE;
    const GLenum MAX_TEXTURE_MAX_ANISOTROPY_EXT = 0x84FF;
};

[NoInterfaceObject]
interface EXT_sRGB
{
    const GLenum SRGB_EXT                                  = 0x8C40;
    const GLenum SRGB_ALPHA_EXT                            = 0x8C42;
    const GLenum SRGB8_ALPHA8_EXT                          = 0x8C43;
    const GLenum FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT = 0x8210;
};

[NoInterfaceObject]
interface OES_standard_derivatives {
    const GLenum FRAGMENT_SHADER_DERIVATIVE_HINT_OES = 0x8B8B;
};

[NoInterfaceObject]
interface OES_texture_float
{
};

[NoInterfaceObject]
interface WEBGL_draw_buffers {
    const GLenum COLOR_ATTACHMENT0_WEBGL     = 0x8CE0;
    const GLenum COLOR_ATTACHMENT1_WEBGL     = 0x8CE1;
    const GLenum COLOR_ATTACHMENT2_WEBGL     = 0x8CE2;
    const GLenum COLOR_ATTACHMENT3_WEBGL     = 0x8CE3;
    const GLenum COLOR_ATTACHMENT4_WEBGL     = 0x8CE4;
    const GLenum COLOR_ATTACHMENT5_WEBGL     = 0x8CE5;
    const GLenum COLOR_ATTACHMENT6_WEBGL     = 0x8CE6;
    const GLenum COLOR_ATTACHMENT7_WEBGL     = 0x8CE7;
    const GLenum COLOR_ATTACHMENT8_WEBGL     = 0x8CE8;
    const GLenum COLOR_ATTACHMENT9_WEBGL     = 0x8CE9;
    const GLenum COLOR_ATTACHMENT10_WEBGL    = 0x8CEA;
    const GLenum COLOR_ATTACHMENT11_WEBGL    = 0x8CEB;
    const GLenum COLOR_ATTACHMENT12_WEBGL    = 0x8CEC;
    const GLenum COLOR_ATTACHMENT13_WEBGL    = 0x8CED;
    const GLenum COLOR_ATTACHMENT14_WEBGL    = 0x8CEE;
    const GLenum COLOR_ATTACHMENT15_WEBGL    = 0x8CEF;

    const GLenum DRAW_BUFFER0_WEBGL          = 0x8825;
    const GLenum DRAW_BUFFER1_WEBGL          = 0x8826;
    const GLenum DRAW_BUFFER2_WEBGL          = 0x8827;
    const GLenum DRAW_BUFFER3_WEBGL          = 0x8828;
    const GLenum DRAW_BUFFER4_WEBGL          = 0x8829;
    const GLenum DRAW_BUFFER5_WEBGL          = 0x882A;
    const GLenum DRAW_BUFFER6_WEBGL          = 0x882B;
    const GLenum DRAW_BUFFER7_WEBGL          = 0x882C;
    const GLenum DRAW_BUFFER8_WEBGL          = 0x882D;
    const GLenum DRAW_BUFFER9_WEBGL          = 0x882E;
    const GLenum DRAW_BUFFER10_WEBGL         = 0x882F;
    const GLenum DRAW_BUFFER11_WEBGL         = 0x8830;
    const GLenum DRAW_BUFFER12_WEBGL         = 0x8831;
    const GLenum DRAW_BUFFER13_WEBGL         = 0x8832;
    const GLenum DRAW_BUFFER14_WEBGL         = 0x8833;
    const GLenum DRAW_BUFFER15_WEBGL         = 0x8834;

    const GLenum MAX_COLOR_ATTACHMENTS_WEBGL = 0x8CDF;
    const GLenum MAX_DRAW_BUFFERS_WEBGL      = 0x8824;

    undefined drawBuffersWEBGL(sequence<GLenum> buffers);
};

[NoInterfaceObject]
interface OES_texture_float_linear
{
};

[NoInterfaceObject]
interface EXT_shader_texture_lod
{
};

[NoInterfaceObject]
interface OES_texture_half_float
{
    const GLenum HALF_FLOAT_OES = 0x8D61;
};

[NoInterfaceObject]
interface OES_texture_half_float_linear
{
};

[NoInterfaceObject]
interface WEBGL_color_buffer_float
{
    const GLenum RGBA32F_EXT = 0x8814;
    const GLenum RGB32F_EXT = 0x8815;
    const GLenum FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT = 0x8211;
    const GLenum UNSIGNED_NORMALIZED_EXT = 0x8C17;
};

[NoInterfaceObject]
interface EXT_color_buffer_half_float
{
    const GLenum RGBA16F_EXT = 0x881A;
    const GLenum RGB16F_EXT = 0x881B;
    const GLenum FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT = 0x8211;
    const GLenum UNSIGNED_NORMALIZED_EXT = 0x8C17;
};

[NoInterfaceObject]
interface OES_vertex_array_object {
    const GLenum VERTEX_ARRAY_BINDING_OES = 0x85B5;

    WebGLVertexArrayObject? createVertexArrayOES();
    undefined deleteVertexArrayOES(WebGLVertexArrayObject? arrayObject);
    [WebGLHandlesContextLoss] GLboolean isVertexArrayOES(WebGLVertexArrayObject? arrayObject);
    undefined bindVertexArrayOES(WebGLVertexArrayObject? arrayObject);
};

[NoInterfaceObject]
interface ANGLE_instanced_arrays {
    const GLenum VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE = 0x88FE;

    undefined drawArraysInstancedANGLE(GLenum mode, GLint first, GLsizei count, GLsizei primcount);
    undefined drawElementsInstancedANGLE(GLenum mode, GLsizei count, GLenum type, GLintptr offset, GLsizei primcount);
    undefined vertexAttribDivisorANGLE(GLuint index, GLuint divisor);
};

[NoInterfaceObject]
interface EXT_blend_minmax {
    const GLenum MIN_EXT = 0x8007;
    const GLenum MAX_EXT = 0x8008;
};

interface WebGLQuery {
};

[NoInterfaceObject]
interface EXT_disjoint_timer_query {
    const GLenum QUERY_COUNTER_BITS_EXT = 0x8864;
    const GLenum CURRENT_QUERY_EXT = 0x8865;
    const GLenum QUERY_RESULT_EXT = 0x8866;
    const GLenum QUERY_RESULT_AVAILABLE_EXT = 0x8867;
    const GLenum TIME_ELAPSED_EXT = 0x88BF;
    const GLenum TIMESTAMP_EXT = 0x8E28;
    const GLenum GPU_DISJOINT_EXT = 0x8FBB;

    WebGLQuery? createQueryEXT();
    undefined deleteQueryEXT(WebGLQuery? query);
    [WebGLHandlesContextLoss] boolean isQueryEXT(WebGLQuery? query);
    undefined beginQueryEXT(GLenum target, WebGLQuery query);
    undefined endQueryEXT(GLenum target);
    undefined queryCounterEXT(WebGLQuery query, GLenum target);
    any getQueryEXT(GLenum target, GLenum pname);
    any getQueryObjectEXT(WebGLQuery query, GLenum pname);
};

[NoInterfaceObject]
interface MOZ_debug {
    const GLenum EXTENSIONS = 0x1F03;
    const GLenum WSI_INFO   = 0x10000;
    const GLenum UNPACK_REQUIRE_FASTPATH = 0x10001;

    [Throws]
    any getParameter(GLenum pname);
};
