#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Graphics_Direct3D12\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D12")]
    pub fn DMLCreateDevice(d3d12device: super::super::super::Graphics::Direct3D12::ID3D12Device, flags: DML_CREATE_DEVICE_FLAGS, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Graphics_Direct3D12\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D12")]
    pub fn DMLCreateDevice1(d3d12device: super::super::super::Graphics::Direct3D12::ID3D12Device, flags: DML_CREATE_DEVICE_FLAGS, minimumfeaturelevel: DML_FEATURE_LEVEL, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
}
pub type IDMLBindingTable = *mut ::core::ffi::c_void;
pub type IDMLCommandRecorder = *mut ::core::ffi::c_void;
pub type IDMLCompiledOperator = *mut ::core::ffi::c_void;
pub type IDMLDebugDevice = *mut ::core::ffi::c_void;
pub type IDMLDevice = *mut ::core::ffi::c_void;
pub type IDMLDevice1 = *mut ::core::ffi::c_void;
pub type IDMLDeviceChild = *mut ::core::ffi::c_void;
pub type IDMLDispatchable = *mut ::core::ffi::c_void;
pub type IDMLObject = *mut ::core::ffi::c_void;
pub type IDMLOperator = *mut ::core::ffi::c_void;
pub type IDMLOperatorInitializer = *mut ::core::ffi::c_void;
pub type IDMLPageable = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_MINIMUM_BUFFER_TENSOR_ALIGNMENT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_PERSISTENT_BUFFER_ALIGNMENT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TARGET_VERSION: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TEMPORARY_BUFFER_ALIGNMENT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DIMENSION_COUNT_MAX: u32 = 5u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DIMENSION_COUNT_MAX1: u32 = 8u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_AXIS_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_AXIS_DIRECTION_INCREASING: DML_AXIS_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_AXIS_DIRECTION_DECREASING: DML_AXIS_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_BINDING_TYPE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_BINDING_TYPE_NONE: DML_BINDING_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_BINDING_TYPE_BUFFER: DML_BINDING_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_BINDING_TYPE_BUFFER_ARRAY: DML_BINDING_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_CONVOLUTION_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_CONVOLUTION_DIRECTION_FORWARD: DML_CONVOLUTION_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_CONVOLUTION_DIRECTION_BACKWARD: DML_CONVOLUTION_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_CONVOLUTION_MODE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_CONVOLUTION_MODE_CONVOLUTION: DML_CONVOLUTION_MODE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_CONVOLUTION_MODE_CROSS_CORRELATION: DML_CONVOLUTION_MODE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_CREATE_DEVICE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_CREATE_DEVICE_FLAG_NONE: DML_CREATE_DEVICE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_CREATE_DEVICE_FLAG_DEBUG: DML_CREATE_DEVICE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_DEPTH_SPACE_ORDER = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_DEPTH_SPACE_ORDER_DEPTH_COLUMN_ROW: DML_DEPTH_SPACE_ORDER = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_DEPTH_SPACE_ORDER_COLUMN_ROW_DEPTH: DML_DEPTH_SPACE_ORDER = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_EXECUTION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_EXECUTION_FLAG_NONE: DML_EXECUTION_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_EXECUTION_FLAG_ALLOW_HALF_PRECISION_COMPUTATION: DML_EXECUTION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_EXECUTION_FLAG_DISABLE_META_COMMANDS: DML_EXECUTION_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_EXECUTION_FLAG_DESCRIPTORS_VOLATILE: DML_EXECUTION_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_FEATURE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_TENSOR_DATA_TYPE_SUPPORT: DML_FEATURE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_FEATURE_LEVELS: DML_FEATURE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_FEATURE_LEVEL = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_LEVEL_1_0: DML_FEATURE_LEVEL = 4096i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_LEVEL_2_0: DML_FEATURE_LEVEL = 8192i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_LEVEL_2_1: DML_FEATURE_LEVEL = 8448i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_LEVEL_3_0: DML_FEATURE_LEVEL = 12288i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_LEVEL_3_1: DML_FEATURE_LEVEL = 12544i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_FEATURE_LEVEL_4_0: DML_FEATURE_LEVEL = 16384i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_GRAPH_EDGE_TYPE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_GRAPH_EDGE_TYPE_INVALID: DML_GRAPH_EDGE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_GRAPH_EDGE_TYPE_INPUT: DML_GRAPH_EDGE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_GRAPH_EDGE_TYPE_OUTPUT: DML_GRAPH_EDGE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_GRAPH_EDGE_TYPE_INTERMEDIATE: DML_GRAPH_EDGE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_GRAPH_NODE_TYPE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_GRAPH_NODE_TYPE_INVALID: DML_GRAPH_NODE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_GRAPH_NODE_TYPE_OPERATOR: DML_GRAPH_NODE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_INTERPOLATION_MODE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_INTERPOLATION_MODE_NEAREST_NEIGHBOR: DML_INTERPOLATION_MODE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_INTERPOLATION_MODE_LINEAR: DML_INTERPOLATION_MODE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_IS_INFINITY_MODE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_IS_INFINITY_MODE_EITHER: DML_IS_INFINITY_MODE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_IS_INFINITY_MODE_POSITIVE: DML_IS_INFINITY_MODE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_IS_INFINITY_MODE_NEGATIVE: DML_IS_INFINITY_MODE = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_MATRIX_TRANSFORM = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_MATRIX_TRANSFORM_NONE: DML_MATRIX_TRANSFORM = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_MATRIX_TRANSFORM_TRANSPOSE: DML_MATRIX_TRANSFORM = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_OPERATOR_TYPE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_INVALID: DML_OPERATOR_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_IDENTITY: DML_OPERATOR_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ABS: DML_OPERATOR_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ACOS: DML_OPERATOR_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ADD: DML_OPERATOR_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ASIN: DML_OPERATOR_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ATAN: DML_OPERATOR_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_CEIL: DML_OPERATOR_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_CLIP: DML_OPERATOR_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_COS: DML_OPERATOR_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_DIVIDE: DML_OPERATOR_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_EXP: DML_OPERATOR_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_FLOOR: DML_OPERATOR_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOG: DML_OPERATOR_TYPE = 13i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_AND: DML_OPERATOR_TYPE = 14i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_EQUALS: DML_OPERATOR_TYPE = 15i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_GREATER_THAN: DML_OPERATOR_TYPE = 16i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_LESS_THAN: DML_OPERATOR_TYPE = 17i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_NOT: DML_OPERATOR_TYPE = 18i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_OR: DML_OPERATOR_TYPE = 19i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_XOR: DML_OPERATOR_TYPE = 20i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_MAX: DML_OPERATOR_TYPE = 21i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_MEAN: DML_OPERATOR_TYPE = 22i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_MIN: DML_OPERATOR_TYPE = 23i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_MULTIPLY: DML_OPERATOR_TYPE = 24i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_POW: DML_OPERATOR_TYPE = 25i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_CONSTANT_POW: DML_OPERATOR_TYPE = 26i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_RECIP: DML_OPERATOR_TYPE = 27i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_SIN: DML_OPERATOR_TYPE = 28i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_SQRT: DML_OPERATOR_TYPE = 29i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_SUBTRACT: DML_OPERATOR_TYPE = 30i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_TAN: DML_OPERATOR_TYPE = 31i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_THRESHOLD: DML_OPERATOR_TYPE = 32i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_QUANTIZE_LINEAR: DML_OPERATOR_TYPE = 33i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_DEQUANTIZE_LINEAR: DML_OPERATOR_TYPE = 34i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_ELU: DML_OPERATOR_TYPE = 35i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_HARDMAX: DML_OPERATOR_TYPE = 36i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_HARD_SIGMOID: DML_OPERATOR_TYPE = 37i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_IDENTITY: DML_OPERATOR_TYPE = 38i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_LEAKY_RELU: DML_OPERATOR_TYPE = 39i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_LINEAR: DML_OPERATOR_TYPE = 40i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_LOG_SOFTMAX: DML_OPERATOR_TYPE = 41i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_PARAMETERIZED_RELU: DML_OPERATOR_TYPE = 42i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_PARAMETRIC_SOFTPLUS: DML_OPERATOR_TYPE = 43i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_RELU: DML_OPERATOR_TYPE = 44i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_SCALED_ELU: DML_OPERATOR_TYPE = 45i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_SCALED_TANH: DML_OPERATOR_TYPE = 46i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_SIGMOID: DML_OPERATOR_TYPE = 47i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_SOFTMAX: DML_OPERATOR_TYPE = 48i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_SOFTPLUS: DML_OPERATOR_TYPE = 49i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_SOFTSIGN: DML_OPERATOR_TYPE = 50i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_TANH: DML_OPERATOR_TYPE = 51i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_THRESHOLDED_RELU: DML_OPERATOR_TYPE = 52i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_CONVOLUTION: DML_OPERATOR_TYPE = 53i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_GEMM: DML_OPERATOR_TYPE = 54i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_REDUCE: DML_OPERATOR_TYPE = 55i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_AVERAGE_POOLING: DML_OPERATOR_TYPE = 56i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_LP_POOLING: DML_OPERATOR_TYPE = 57i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MAX_POOLING: DML_OPERATOR_TYPE = 58i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ROI_POOLING: DML_OPERATOR_TYPE = 59i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SLICE: DML_OPERATOR_TYPE = 60i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_CAST: DML_OPERATOR_TYPE = 61i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SPLIT: DML_OPERATOR_TYPE = 62i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_JOIN: DML_OPERATOR_TYPE = 63i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_PADDING: DML_OPERATOR_TYPE = 64i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_VALUE_SCALE_2D: DML_OPERATOR_TYPE = 65i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_UPSAMPLE_2D: DML_OPERATOR_TYPE = 66i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_GATHER: DML_OPERATOR_TYPE = 67i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SPACE_TO_DEPTH: DML_OPERATOR_TYPE = 68i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_DEPTH_TO_SPACE: DML_OPERATOR_TYPE = 69i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_TILE: DML_OPERATOR_TYPE = 70i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_TOP_K: DML_OPERATOR_TYPE = 71i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_BATCH_NORMALIZATION: DML_OPERATOR_TYPE = 72i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MEAN_VARIANCE_NORMALIZATION: DML_OPERATOR_TYPE = 73i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_LOCAL_RESPONSE_NORMALIZATION: DML_OPERATOR_TYPE = 74i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_LP_NORMALIZATION: DML_OPERATOR_TYPE = 75i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_RNN: DML_OPERATOR_TYPE = 76i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_LSTM: DML_OPERATOR_TYPE = 77i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_GRU: DML_OPERATOR_TYPE = 78i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_SIGN: DML_OPERATOR_TYPE = 79i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_IS_NAN: DML_OPERATOR_TYPE = 80i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ERF: DML_OPERATOR_TYPE = 81i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_SINH: DML_OPERATOR_TYPE = 82i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_COSH: DML_OPERATOR_TYPE = 83i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_TANH: DML_OPERATOR_TYPE = 84i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ASINH: DML_OPERATOR_TYPE = 85i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ACOSH: DML_OPERATOR_TYPE = 86i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ATANH: DML_OPERATOR_TYPE = 87i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_IF: DML_OPERATOR_TYPE = 88i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ADD1: DML_OPERATOR_TYPE = 89i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_SHRINK: DML_OPERATOR_TYPE = 90i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MAX_POOLING1: DML_OPERATOR_TYPE = 91i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MAX_UNPOOLING: DML_OPERATOR_TYPE = 92i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_DIAGONAL_MATRIX: DML_OPERATOR_TYPE = 93i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SCATTER_ELEMENTS: DML_OPERATOR_TYPE = 94i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SCATTER: DML_OPERATOR_TYPE = 94i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ONE_HOT: DML_OPERATOR_TYPE = 95i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_RESAMPLE: DML_OPERATOR_TYPE = 96i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_BIT_SHIFT_LEFT: DML_OPERATOR_TYPE = 97i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_BIT_SHIFT_RIGHT: DML_OPERATOR_TYPE = 98i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ROUND: DML_OPERATOR_TYPE = 99i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_IS_INFINITY: DML_OPERATOR_TYPE = 100i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_MODULUS_TRUNCATE: DML_OPERATOR_TYPE = 101i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_MODULUS_FLOOR: DML_OPERATOR_TYPE = 102i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_FILL_VALUE_CONSTANT: DML_OPERATOR_TYPE = 103i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_FILL_VALUE_SEQUENCE: DML_OPERATOR_TYPE = 104i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_CUMULATIVE_SUMMATION: DML_OPERATOR_TYPE = 105i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_REVERSE_SUBSEQUENCES: DML_OPERATOR_TYPE = 106i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_GATHER_ELEMENTS: DML_OPERATOR_TYPE = 107i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_GATHER_ND: DML_OPERATOR_TYPE = 108i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SCATTER_ND: DML_OPERATOR_TYPE = 109i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MAX_POOLING2: DML_OPERATOR_TYPE = 110i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SLICE1: DML_OPERATOR_TYPE = 111i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_TOP_K1: DML_OPERATOR_TYPE = 112i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_DEPTH_TO_SPACE1: DML_OPERATOR_TYPE = 113i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SPACE_TO_DEPTH1: DML_OPERATOR_TYPE = 114i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MEAN_VARIANCE_NORMALIZATION1: DML_OPERATOR_TYPE = 115i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_RESAMPLE1: DML_OPERATOR_TYPE = 116i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MATRIX_MULTIPLY_INTEGER: DML_OPERATOR_TYPE = 117i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_QUANTIZED_LINEAR_MATRIX_MULTIPLY: DML_OPERATOR_TYPE = 118i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_CONVOLUTION_INTEGER: DML_OPERATOR_TYPE = 119i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_QUANTIZED_LINEAR_CONVOLUTION: DML_OPERATOR_TYPE = 120i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_BIT_AND: DML_OPERATOR_TYPE = 121i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_BIT_OR: DML_OPERATOR_TYPE = 122i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_BIT_XOR: DML_OPERATOR_TYPE = 123i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_BIT_NOT: DML_OPERATOR_TYPE = 124i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_BIT_COUNT: DML_OPERATOR_TYPE = 125i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_GREATER_THAN_OR_EQUAL: DML_OPERATOR_TYPE = 126i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_LOGICAL_LESS_THAN_OR_EQUAL: DML_OPERATOR_TYPE = 127i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_CELU: DML_OPERATOR_TYPE = 128i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ACTIVATION_RELU_GRAD: DML_OPERATOR_TYPE = 129i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_AVERAGE_POOLING_GRAD: DML_OPERATOR_TYPE = 130i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_MAX_POOLING_GRAD: DML_OPERATOR_TYPE = 131i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_RANDOM_GENERATOR: DML_OPERATOR_TYPE = 132i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_NONZERO_COORDINATES: DML_OPERATOR_TYPE = 133i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_RESAMPLE_GRAD: DML_OPERATOR_TYPE = 134i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_SLICE_GRAD: DML_OPERATOR_TYPE = 135i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ADAM_OPTIMIZER: DML_OPERATOR_TYPE = 136i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ARGMIN: DML_OPERATOR_TYPE = 137i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ARGMAX: DML_OPERATOR_TYPE = 138i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ROI_ALIGN: DML_OPERATOR_TYPE = 139i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_GATHER_ND1: DML_OPERATOR_TYPE = 140i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_ATAN_YX: DML_OPERATOR_TYPE = 141i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_CLIP_GRAD: DML_OPERATOR_TYPE = 142i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_DIFFERENCE_SQUARE: DML_OPERATOR_TYPE = 143i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_LOCAL_RESPONSE_NORMALIZATION_GRAD: DML_OPERATOR_TYPE = 144i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_CUMULATIVE_PRODUCT: DML_OPERATOR_TYPE = 145i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_BATCH_NORMALIZATION_GRAD: DML_OPERATOR_TYPE = 146i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ELEMENT_WISE_QUANTIZED_LINEAR_ADD: DML_OPERATOR_TYPE = 147i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_DYNAMIC_QUANTIZE_LINEAR: DML_OPERATOR_TYPE = 148i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_OPERATOR_ROI_ALIGN1: DML_OPERATOR_TYPE = 149i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_PADDING_MODE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_PADDING_MODE_CONSTANT: DML_PADDING_MODE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_PADDING_MODE_EDGE: DML_PADDING_MODE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_PADDING_MODE_REFLECTION: DML_PADDING_MODE = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_PADDING_MODE_SYMMETRIC: DML_PADDING_MODE = 3i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_RANDOM_GENERATOR_TYPE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_RANDOM_GENERATOR_TYPE_PHILOX_4X32_10: DML_RANDOM_GENERATOR_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_RECURRENT_NETWORK_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_RECURRENT_NETWORK_DIRECTION_FORWARD: DML_RECURRENT_NETWORK_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_RECURRENT_NETWORK_DIRECTION_BACKWARD: DML_RECURRENT_NETWORK_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_RECURRENT_NETWORK_DIRECTION_BIDIRECTIONAL: DML_RECURRENT_NETWORK_DIRECTION = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_REDUCE_FUNCTION = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_ARGMAX: DML_REDUCE_FUNCTION = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_ARGMIN: DML_REDUCE_FUNCTION = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_AVERAGE: DML_REDUCE_FUNCTION = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_L1: DML_REDUCE_FUNCTION = 3i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_L2: DML_REDUCE_FUNCTION = 4i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_LOG_SUM: DML_REDUCE_FUNCTION = 5i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_LOG_SUM_EXP: DML_REDUCE_FUNCTION = 6i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_MAX: DML_REDUCE_FUNCTION = 7i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_MIN: DML_REDUCE_FUNCTION = 8i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_MULTIPLY: DML_REDUCE_FUNCTION = 9i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_SUM: DML_REDUCE_FUNCTION = 10i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_REDUCE_FUNCTION_SUM_SQUARE: DML_REDUCE_FUNCTION = 11i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_ROUNDING_MODE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_ROUNDING_MODE_HALVES_TO_NEAREST_EVEN: DML_ROUNDING_MODE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_ROUNDING_MODE_TOWARD_ZERO: DML_ROUNDING_MODE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_ROUNDING_MODE_TOWARD_INFINITY: DML_ROUNDING_MODE = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_TENSOR_DATA_TYPE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_UNKNOWN: DML_TENSOR_DATA_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_FLOAT32: DML_TENSOR_DATA_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_FLOAT16: DML_TENSOR_DATA_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_UINT32: DML_TENSOR_DATA_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_UINT16: DML_TENSOR_DATA_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_UINT8: DML_TENSOR_DATA_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_INT32: DML_TENSOR_DATA_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_INT16: DML_TENSOR_DATA_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_INT8: DML_TENSOR_DATA_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_FLOAT64: DML_TENSOR_DATA_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_UINT64: DML_TENSOR_DATA_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_DATA_TYPE_INT64: DML_TENSOR_DATA_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_TENSOR_FLAGS = u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_FLAG_NONE: DML_TENSOR_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_FLAG_OWNED_BY_DML: DML_TENSOR_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub type DML_TENSOR_TYPE = i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_TYPE_INVALID: DML_TENSOR_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub const DML_TENSOR_TYPE_BUFFER: DML_TENSOR_TYPE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_CELU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_CELU_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_CELU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_ELU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_ELU_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_ELU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_HARDMAX_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_HARDMAX_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_HARDMAX_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_HARD_SIGMOID_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
    pub Beta: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_HARD_SIGMOID_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_HARD_SIGMOID_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_IDENTITY_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_IDENTITY_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_IDENTITY_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_LEAKY_RELU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_LEAKY_RELU_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_LEAKY_RELU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_LINEAR_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
    pub Beta: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_LINEAR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_LINEAR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_LOG_SOFTMAX_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_LOG_SOFTMAX_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_LOG_SOFTMAX_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_PARAMETERIZED_RELU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub SlopeTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_PARAMETERIZED_RELU_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_PARAMETERIZED_RELU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_PARAMETRIC_SOFTPLUS_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
    pub Beta: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_PARAMETRIC_SOFTPLUS_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_PARAMETRIC_SOFTPLUS_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_RELU_GRAD_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_RELU_GRAD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_RELU_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_RELU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_RELU_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_RELU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_SCALED_ELU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
    pub Gamma: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_SCALED_ELU_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_SCALED_ELU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_SCALED_TANH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
    pub Beta: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_SCALED_TANH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_SCALED_TANH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_SHRINK_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Bias: f32,
    pub Threshold: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_SHRINK_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_SHRINK_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_SIGMOID_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_SIGMOID_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_SIGMOID_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_SOFTMAX_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_SOFTMAX_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_SOFTMAX_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_SOFTPLUS_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Steepness: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_SOFTPLUS_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_SOFTPLUS_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_SOFTSIGN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_SOFTSIGN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_SOFTSIGN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_TANH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ACTIVATION_TANH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_TANH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ACTIVATION_THRESHOLDED_RELU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Alpha: f32,
}
impl ::core::marker::Copy for DML_ACTIVATION_THRESHOLDED_RELU_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ACTIVATION_THRESHOLDED_RELU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ADAM_OPTIMIZER_OPERATOR_DESC {
    pub InputParametersTensor: *const DML_TENSOR_DESC,
    pub InputFirstMomentTensor: *const DML_TENSOR_DESC,
    pub InputSecondMomentTensor: *const DML_TENSOR_DESC,
    pub GradientTensor: *const DML_TENSOR_DESC,
    pub TrainingStepTensor: *const DML_TENSOR_DESC,
    pub OutputParametersTensor: *const DML_TENSOR_DESC,
    pub OutputFirstMomentTensor: *const DML_TENSOR_DESC,
    pub OutputSecondMomentTensor: *const DML_TENSOR_DESC,
    pub LearningRate: f32,
    pub Beta1: f32,
    pub Beta2: f32,
    pub Epsilon: f32,
}
impl ::core::marker::Copy for DML_ADAM_OPTIMIZER_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ADAM_OPTIMIZER_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ARGMAX_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub AxisCount: u32,
    pub Axes: *const u32,
    pub AxisDirection: DML_AXIS_DIRECTION,
}
impl ::core::marker::Copy for DML_ARGMAX_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ARGMAX_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ARGMIN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub AxisCount: u32,
    pub Axes: *const u32,
    pub AxisDirection: DML_AXIS_DIRECTION,
}
impl ::core::marker::Copy for DML_ARGMIN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ARGMIN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_AVERAGE_POOLING_GRAD_OPERATOR_DESC {
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub WindowSize: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub IncludePadding: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_AVERAGE_POOLING_GRAD_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_AVERAGE_POOLING_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_AVERAGE_POOLING_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub WindowSize: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub IncludePadding: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_AVERAGE_POOLING_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_AVERAGE_POOLING_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_BATCH_NORMALIZATION_GRAD_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub MeanTensor: *const DML_TENSOR_DESC,
    pub VarianceTensor: *const DML_TENSOR_DESC,
    pub ScaleTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputScaleGradientTensor: *const DML_TENSOR_DESC,
    pub OutputBiasGradientTensor: *const DML_TENSOR_DESC,
    pub Epsilon: f32,
}
impl ::core::marker::Copy for DML_BATCH_NORMALIZATION_GRAD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_BATCH_NORMALIZATION_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_BATCH_NORMALIZATION_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub MeanTensor: *const DML_TENSOR_DESC,
    pub VarianceTensor: *const DML_TENSOR_DESC,
    pub ScaleTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Spatial: super::super::super::Foundation::BOOL,
    pub Epsilon: f32,
    pub FusedActivation: *const DML_OPERATOR_DESC,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_BATCH_NORMALIZATION_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_BATCH_NORMALIZATION_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_BINDING_DESC {
    pub Type: DML_BINDING_TYPE,
    pub Desc: *const ::core::ffi::c_void,
}
impl ::core::marker::Copy for DML_BINDING_DESC {}
impl ::core::clone::Clone for DML_BINDING_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_BINDING_PROPERTIES {
    pub RequiredDescriptorCount: u32,
    pub TemporaryResourceSize: u64,
    pub PersistentResourceSize: u64,
}
impl ::core::marker::Copy for DML_BINDING_PROPERTIES {}
impl ::core::clone::Clone for DML_BINDING_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Graphics_Direct3D12\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D12")]
pub struct DML_BINDING_TABLE_DESC {
    pub Dispatchable: IDMLDispatchable,
    pub CPUDescriptorHandle: super::super::super::Graphics::Direct3D12::D3D12_CPU_DESCRIPTOR_HANDLE,
    pub GPUDescriptorHandle: super::super::super::Graphics::Direct3D12::D3D12_GPU_DESCRIPTOR_HANDLE,
    pub SizeInDescriptors: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D12")]
impl ::core::marker::Copy for DML_BINDING_TABLE_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D12")]
impl ::core::clone::Clone for DML_BINDING_TABLE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Graphics_Direct3D12\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D12")]
pub struct DML_BUFFER_ARRAY_BINDING {
    pub BindingCount: u32,
    pub Bindings: *const DML_BUFFER_BINDING,
}
#[cfg(feature = "Win32_Graphics_Direct3D12")]
impl ::core::marker::Copy for DML_BUFFER_ARRAY_BINDING {}
#[cfg(feature = "Win32_Graphics_Direct3D12")]
impl ::core::clone::Clone for DML_BUFFER_ARRAY_BINDING {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Graphics_Direct3D12\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D12")]
pub struct DML_BUFFER_BINDING {
    pub Buffer: super::super::super::Graphics::Direct3D12::ID3D12Resource,
    pub Offset: u64,
    pub SizeInBytes: u64,
}
#[cfg(feature = "Win32_Graphics_Direct3D12")]
impl ::core::marker::Copy for DML_BUFFER_BINDING {}
#[cfg(feature = "Win32_Graphics_Direct3D12")]
impl ::core::clone::Clone for DML_BUFFER_BINDING {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_BUFFER_TENSOR_DESC {
    pub DataType: DML_TENSOR_DATA_TYPE,
    pub Flags: DML_TENSOR_FLAGS,
    pub DimensionCount: u32,
    pub Sizes: *const u32,
    pub Strides: *const u32,
    pub TotalTensorSizeInBytes: u64,
    pub GuaranteedBaseOffsetAlignment: u32,
}
impl ::core::marker::Copy for DML_BUFFER_TENSOR_DESC {}
impl ::core::clone::Clone for DML_BUFFER_TENSOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_CAST_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_CAST_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_CAST_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_CONVOLUTION_INTEGER_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub InputZeroPointTensor: *const DML_TENSOR_DESC,
    pub FilterTensor: *const DML_TENSOR_DESC,
    pub FilterZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub Dilations: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub GroupCount: u32,
}
impl ::core::marker::Copy for DML_CONVOLUTION_INTEGER_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_CONVOLUTION_INTEGER_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_CONVOLUTION_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub FilterTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Mode: DML_CONVOLUTION_MODE,
    pub Direction: DML_CONVOLUTION_DIRECTION,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub Dilations: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub OutputPadding: *const u32,
    pub GroupCount: u32,
    pub FusedActivation: *const DML_OPERATOR_DESC,
}
impl ::core::marker::Copy for DML_CONVOLUTION_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_CONVOLUTION_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_CUMULATIVE_PRODUCT_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
    pub AxisDirection: DML_AXIS_DIRECTION,
    pub HasExclusiveProduct: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_CUMULATIVE_PRODUCT_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_CUMULATIVE_PRODUCT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_CUMULATIVE_SUMMATION_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
    pub AxisDirection: DML_AXIS_DIRECTION,
    pub HasExclusiveSum: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_CUMULATIVE_SUMMATION_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_CUMULATIVE_SUMMATION_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_DEPTH_TO_SPACE1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub BlockSize: u32,
    pub Order: DML_DEPTH_SPACE_ORDER,
}
impl ::core::marker::Copy for DML_DEPTH_TO_SPACE1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_DEPTH_TO_SPACE1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_DEPTH_TO_SPACE_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub BlockSize: u32,
}
impl ::core::marker::Copy for DML_DEPTH_TO_SPACE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_DEPTH_TO_SPACE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_DIAGONAL_MATRIX_OPERATOR_DESC {
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Offset: i32,
    pub Value: f32,
}
impl ::core::marker::Copy for DML_DIAGONAL_MATRIX_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_DIAGONAL_MATRIX_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_DYNAMIC_QUANTIZE_LINEAR_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub OutputScaleTensor: *const DML_TENSOR_DESC,
    pub OutputZeroPointTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_DYNAMIC_QUANTIZE_LINEAR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_DYNAMIC_QUANTIZE_LINEAR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ABS_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ABS_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ABS_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ACOSH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ACOSH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ACOSH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ACOS_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ACOS_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ACOS_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ADD1_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub FusedActivation: *const DML_OPERATOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ADD1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ADD1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ADD_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ADD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ADD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ASINH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ASINH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ASINH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ASIN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ASIN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ASIN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ATANH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ATANH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ATANH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ATAN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ATAN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ATAN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ATAN_YX_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ATAN_YX_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ATAN_YX_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_BIT_AND_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_BIT_AND_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_BIT_AND_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_BIT_COUNT_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_BIT_COUNT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_BIT_COUNT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_BIT_NOT_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_BIT_NOT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_BIT_NOT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_BIT_OR_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_BIT_OR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_BIT_OR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_BIT_SHIFT_LEFT_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_BIT_SHIFT_LEFT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_BIT_SHIFT_LEFT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_BIT_SHIFT_RIGHT_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_BIT_SHIFT_RIGHT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_BIT_SHIFT_RIGHT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_BIT_XOR_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_BIT_XOR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_BIT_XOR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_CEIL_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_CEIL_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_CEIL_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_CLIP_GRAD_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
    pub Min: f32,
    pub Max: f32,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_CLIP_GRAD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_CLIP_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_CLIP_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
    pub Min: f32,
    pub Max: f32,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_CLIP_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_CLIP_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_CONSTANT_POW_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
    pub Exponent: f32,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_CONSTANT_POW_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_CONSTANT_POW_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_COSH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_COSH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_COSH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_COS_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_COS_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_COS_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_DEQUANTIZE_LINEAR_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ScaleTensor: *const DML_TENSOR_DESC,
    pub ZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_DEQUANTIZE_LINEAR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_DEQUANTIZE_LINEAR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_DIFFERENCE_SQUARE_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_DIFFERENCE_SQUARE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_DIFFERENCE_SQUARE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_DIVIDE_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_DIVIDE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_DIVIDE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ERF_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ERF_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ERF_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_EXP_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_EXP_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_EXP_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_FLOOR_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_FLOOR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_FLOOR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_IDENTITY_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_IDENTITY_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_IDENTITY_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_IF_OPERATOR_DESC {
    pub ConditionTensor: *const DML_TENSOR_DESC,
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_IF_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_IF_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_IS_INFINITY_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub InfinityMode: DML_IS_INFINITY_MODE,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_IS_INFINITY_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_IS_INFINITY_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_IS_NAN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_IS_NAN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_IS_NAN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_AND_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_AND_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_AND_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_EQUALS_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_EQUALS_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_EQUALS_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_GREATER_THAN_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_GREATER_THAN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_GREATER_THAN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_GREATER_THAN_OR_EQUAL_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_GREATER_THAN_OR_EQUAL_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_GREATER_THAN_OR_EQUAL_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_LESS_THAN_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_LESS_THAN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_LESS_THAN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_LESS_THAN_OR_EQUAL_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_LESS_THAN_OR_EQUAL_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_LESS_THAN_OR_EQUAL_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_NOT_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_NOT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_NOT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_OR_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_OR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_OR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOGICAL_XOR_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOGICAL_XOR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOGICAL_XOR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_LOG_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_LOG_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_LOG_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_MAX_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_MAX_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_MAX_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_MEAN_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_MEAN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_MEAN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_MIN_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_MIN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_MIN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_MODULUS_FLOOR_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_MODULUS_FLOOR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_MODULUS_FLOOR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_MODULUS_TRUNCATE_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_MODULUS_TRUNCATE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_MODULUS_TRUNCATE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_MULTIPLY_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_MULTIPLY_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_MULTIPLY_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_POW_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ExponentTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_POW_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_POW_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_QUANTIZED_LINEAR_ADD_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub AScaleTensor: *const DML_TENSOR_DESC,
    pub AZeroPointTensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub BScaleTensor: *const DML_TENSOR_DESC,
    pub BZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputScaleTensor: *const DML_TENSOR_DESC,
    pub OutputZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_QUANTIZED_LINEAR_ADD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_QUANTIZED_LINEAR_ADD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_QUANTIZE_LINEAR_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ScaleTensor: *const DML_TENSOR_DESC,
    pub ZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_QUANTIZE_LINEAR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_QUANTIZE_LINEAR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_RECIP_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_RECIP_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_RECIP_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_ROUND_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub RoundingMode: DML_ROUNDING_MODE,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_ROUND_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_ROUND_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_SIGN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_SIGN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_SIGN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_SINH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_SINH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_SINH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_SIN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_SIN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_SIN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_SQRT_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_SQRT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_SQRT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_SUBTRACT_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_SUBTRACT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_SUBTRACT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_TANH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_TANH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_TANH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_TAN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_TAN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_TAN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ELEMENT_WISE_THRESHOLD_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleBias: *const DML_SCALE_BIAS,
    pub Min: f32,
}
impl ::core::marker::Copy for DML_ELEMENT_WISE_THRESHOLD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ELEMENT_WISE_THRESHOLD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_FEATURE_DATA_FEATURE_LEVELS {
    pub MaxSupportedFeatureLevel: DML_FEATURE_LEVEL,
}
impl ::core::marker::Copy for DML_FEATURE_DATA_FEATURE_LEVELS {}
impl ::core::clone::Clone for DML_FEATURE_DATA_FEATURE_LEVELS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_FEATURE_DATA_TENSOR_DATA_TYPE_SUPPORT {
    pub IsSupported: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_FEATURE_DATA_TENSOR_DATA_TYPE_SUPPORT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_FEATURE_DATA_TENSOR_DATA_TYPE_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_FEATURE_QUERY_FEATURE_LEVELS {
    pub RequestedFeatureLevelCount: u32,
    pub RequestedFeatureLevels: *const DML_FEATURE_LEVEL,
}
impl ::core::marker::Copy for DML_FEATURE_QUERY_FEATURE_LEVELS {}
impl ::core::clone::Clone for DML_FEATURE_QUERY_FEATURE_LEVELS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_FEATURE_QUERY_TENSOR_DATA_TYPE_SUPPORT {
    pub DataType: DML_TENSOR_DATA_TYPE,
}
impl ::core::marker::Copy for DML_FEATURE_QUERY_TENSOR_DATA_TYPE_SUPPORT {}
impl ::core::clone::Clone for DML_FEATURE_QUERY_TENSOR_DATA_TYPE_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_FILL_VALUE_CONSTANT_OPERATOR_DESC {
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ValueDataType: DML_TENSOR_DATA_TYPE,
    pub Value: DML_SCALAR_UNION,
}
impl ::core::marker::Copy for DML_FILL_VALUE_CONSTANT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_FILL_VALUE_CONSTANT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_FILL_VALUE_SEQUENCE_OPERATOR_DESC {
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ValueDataType: DML_TENSOR_DATA_TYPE,
    pub ValueStart: DML_SCALAR_UNION,
    pub ValueDelta: DML_SCALAR_UNION,
}
impl ::core::marker::Copy for DML_FILL_VALUE_SEQUENCE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_FILL_VALUE_SEQUENCE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GATHER_ELEMENTS_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
}
impl ::core::marker::Copy for DML_GATHER_ELEMENTS_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_GATHER_ELEMENTS_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GATHER_ND1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub InputDimensionCount: u32,
    pub IndicesDimensionCount: u32,
    pub BatchDimensionCount: u32,
}
impl ::core::marker::Copy for DML_GATHER_ND1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_GATHER_ND1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GATHER_ND_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub InputDimensionCount: u32,
    pub IndicesDimensionCount: u32,
}
impl ::core::marker::Copy for DML_GATHER_ND_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_GATHER_ND_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GATHER_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
    pub IndexDimensions: u32,
}
impl ::core::marker::Copy for DML_GATHER_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_GATHER_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GEMM_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub CTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub TransA: DML_MATRIX_TRANSFORM,
    pub TransB: DML_MATRIX_TRANSFORM,
    pub Alpha: f32,
    pub Beta: f32,
    pub FusedActivation: *const DML_OPERATOR_DESC,
}
impl ::core::marker::Copy for DML_GEMM_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_GEMM_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GRAPH_DESC {
    pub InputCount: u32,
    pub OutputCount: u32,
    pub NodeCount: u32,
    pub Nodes: *const DML_GRAPH_NODE_DESC,
    pub InputEdgeCount: u32,
    pub InputEdges: *const DML_GRAPH_EDGE_DESC,
    pub OutputEdgeCount: u32,
    pub OutputEdges: *const DML_GRAPH_EDGE_DESC,
    pub IntermediateEdgeCount: u32,
    pub IntermediateEdges: *const DML_GRAPH_EDGE_DESC,
}
impl ::core::marker::Copy for DML_GRAPH_DESC {}
impl ::core::clone::Clone for DML_GRAPH_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GRAPH_EDGE_DESC {
    pub Type: DML_GRAPH_EDGE_TYPE,
    pub Desc: *const ::core::ffi::c_void,
}
impl ::core::marker::Copy for DML_GRAPH_EDGE_DESC {}
impl ::core::clone::Clone for DML_GRAPH_EDGE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_GRAPH_NODE_DESC {
    pub Type: DML_GRAPH_NODE_TYPE,
    pub Desc: *const ::core::ffi::c_void,
}
impl ::core::marker::Copy for DML_GRAPH_NODE_DESC {}
impl ::core::clone::Clone for DML_GRAPH_NODE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_GRU_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub WeightTensor: *const DML_TENSOR_DESC,
    pub RecurrenceTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub HiddenInitTensor: *const DML_TENSOR_DESC,
    pub SequenceLengthsTensor: *const DML_TENSOR_DESC,
    pub OutputSequenceTensor: *const DML_TENSOR_DESC,
    pub OutputSingleTensor: *const DML_TENSOR_DESC,
    pub ActivationDescCount: u32,
    pub ActivationDescs: *const DML_OPERATOR_DESC,
    pub Direction: DML_RECURRENT_NETWORK_DIRECTION,
    pub LinearBeforeReset: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_GRU_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_GRU_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_INPUT_GRAPH_EDGE_DESC {
    pub GraphInputIndex: u32,
    pub ToNodeIndex: u32,
    pub ToNodeInputIndex: u32,
    pub Name: ::windows_sys::core::PCSTR,
}
impl ::core::marker::Copy for DML_INPUT_GRAPH_EDGE_DESC {}
impl ::core::clone::Clone for DML_INPUT_GRAPH_EDGE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_INTERMEDIATE_GRAPH_EDGE_DESC {
    pub FromNodeIndex: u32,
    pub FromNodeOutputIndex: u32,
    pub ToNodeIndex: u32,
    pub ToNodeInputIndex: u32,
    pub Name: ::windows_sys::core::PCSTR,
}
impl ::core::marker::Copy for DML_INTERMEDIATE_GRAPH_EDGE_DESC {}
impl ::core::clone::Clone for DML_INTERMEDIATE_GRAPH_EDGE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_JOIN_OPERATOR_DESC {
    pub InputCount: u32,
    pub InputTensors: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
}
impl ::core::marker::Copy for DML_JOIN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_JOIN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_LOCAL_RESPONSE_NORMALIZATION_GRAD_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
    pub CrossChannel: super::super::super::Foundation::BOOL,
    pub LocalSize: u32,
    pub Alpha: f32,
    pub Beta: f32,
    pub Bias: f32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_LOCAL_RESPONSE_NORMALIZATION_GRAD_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_LOCAL_RESPONSE_NORMALIZATION_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_LOCAL_RESPONSE_NORMALIZATION_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub CrossChannel: super::super::super::Foundation::BOOL,
    pub LocalSize: u32,
    pub Alpha: f32,
    pub Beta: f32,
    pub Bias: f32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_LOCAL_RESPONSE_NORMALIZATION_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_LOCAL_RESPONSE_NORMALIZATION_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_LP_NORMALIZATION_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
    pub Epsilon: f32,
    pub P: u32,
}
impl ::core::marker::Copy for DML_LP_NORMALIZATION_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_LP_NORMALIZATION_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_LP_POOLING_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub WindowSize: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub P: u32,
}
impl ::core::marker::Copy for DML_LP_POOLING_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_LP_POOLING_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_LSTM_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub WeightTensor: *const DML_TENSOR_DESC,
    pub RecurrenceTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub HiddenInitTensor: *const DML_TENSOR_DESC,
    pub CellMemInitTensor: *const DML_TENSOR_DESC,
    pub SequenceLengthsTensor: *const DML_TENSOR_DESC,
    pub PeepholeTensor: *const DML_TENSOR_DESC,
    pub OutputSequenceTensor: *const DML_TENSOR_DESC,
    pub OutputSingleTensor: *const DML_TENSOR_DESC,
    pub OutputCellSingleTensor: *const DML_TENSOR_DESC,
    pub ActivationDescCount: u32,
    pub ActivationDescs: *const DML_OPERATOR_DESC,
    pub Direction: DML_RECURRENT_NETWORK_DIRECTION,
    pub ClipThreshold: f32,
    pub UseClipThreshold: super::super::super::Foundation::BOOL,
    pub CoupleInputForget: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_LSTM_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_LSTM_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_MATRIX_MULTIPLY_INTEGER_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub AZeroPointTensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub BZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_MATRIX_MULTIPLY_INTEGER_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_MATRIX_MULTIPLY_INTEGER_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_MAX_POOLING1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub OutputIndicesTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub WindowSize: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
}
impl ::core::marker::Copy for DML_MAX_POOLING1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_MAX_POOLING1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_MAX_POOLING2_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub OutputIndicesTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub WindowSize: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub Dilations: *const u32,
}
impl ::core::marker::Copy for DML_MAX_POOLING2_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_MAX_POOLING2_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_MAX_POOLING_GRAD_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub WindowSize: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub Dilations: *const u32,
}
impl ::core::marker::Copy for DML_MAX_POOLING_GRAD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_MAX_POOLING_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_MAX_POOLING_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub WindowSize: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
}
impl ::core::marker::Copy for DML_MAX_POOLING_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_MAX_POOLING_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_MAX_UNPOOLING_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_MAX_UNPOOLING_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_MAX_UNPOOLING_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_MEAN_VARIANCE_NORMALIZATION1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ScaleTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub AxisCount: u32,
    pub Axes: *const u32,
    pub NormalizeVariance: super::super::super::Foundation::BOOL,
    pub Epsilon: f32,
    pub FusedActivation: *const DML_OPERATOR_DESC,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_MEAN_VARIANCE_NORMALIZATION1_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_MEAN_VARIANCE_NORMALIZATION1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_MEAN_VARIANCE_NORMALIZATION_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ScaleTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub CrossChannel: super::super::super::Foundation::BOOL,
    pub NormalizeVariance: super::super::super::Foundation::BOOL,
    pub Epsilon: f32,
    pub FusedActivation: *const DML_OPERATOR_DESC,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_MEAN_VARIANCE_NORMALIZATION_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_MEAN_VARIANCE_NORMALIZATION_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_NONZERO_COORDINATES_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputCountTensor: *const DML_TENSOR_DESC,
    pub OutputCoordinatesTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_NONZERO_COORDINATES_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_NONZERO_COORDINATES_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ONE_HOT_OPERATOR_DESC {
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub ValuesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
}
impl ::core::marker::Copy for DML_ONE_HOT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ONE_HOT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_OPERATOR_DESC {
    pub Type: DML_OPERATOR_TYPE,
    pub Desc: *const ::core::ffi::c_void,
}
impl ::core::marker::Copy for DML_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_OPERATOR_GRAPH_NODE_DESC {
    pub Operator: IDMLOperator,
    pub Name: ::windows_sys::core::PCSTR,
}
impl ::core::marker::Copy for DML_OPERATOR_GRAPH_NODE_DESC {}
impl ::core::clone::Clone for DML_OPERATOR_GRAPH_NODE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_OUTPUT_GRAPH_EDGE_DESC {
    pub FromNodeIndex: u32,
    pub FromNodeOutputIndex: u32,
    pub GraphOutputIndex: u32,
    pub Name: ::windows_sys::core::PCSTR,
}
impl ::core::marker::Copy for DML_OUTPUT_GRAPH_EDGE_DESC {}
impl ::core::clone::Clone for DML_OUTPUT_GRAPH_EDGE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_PADDING_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub PaddingMode: DML_PADDING_MODE,
    pub PaddingValue: f32,
    pub DimensionCount: u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
}
impl ::core::marker::Copy for DML_PADDING_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_PADDING_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_QUANTIZED_LINEAR_CONVOLUTION_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub InputScaleTensor: *const DML_TENSOR_DESC,
    pub InputZeroPointTensor: *const DML_TENSOR_DESC,
    pub FilterTensor: *const DML_TENSOR_DESC,
    pub FilterScaleTensor: *const DML_TENSOR_DESC,
    pub FilterZeroPointTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub OutputScaleTensor: *const DML_TENSOR_DESC,
    pub OutputZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Strides: *const u32,
    pub Dilations: *const u32,
    pub StartPadding: *const u32,
    pub EndPadding: *const u32,
    pub GroupCount: u32,
}
impl ::core::marker::Copy for DML_QUANTIZED_LINEAR_CONVOLUTION_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_QUANTIZED_LINEAR_CONVOLUTION_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_QUANTIZED_LINEAR_MATRIX_MULTIPLY_OPERATOR_DESC {
    pub ATensor: *const DML_TENSOR_DESC,
    pub AScaleTensor: *const DML_TENSOR_DESC,
    pub AZeroPointTensor: *const DML_TENSOR_DESC,
    pub BTensor: *const DML_TENSOR_DESC,
    pub BScaleTensor: *const DML_TENSOR_DESC,
    pub BZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputScaleTensor: *const DML_TENSOR_DESC,
    pub OutputZeroPointTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
}
impl ::core::marker::Copy for DML_QUANTIZED_LINEAR_MATRIX_MULTIPLY_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_QUANTIZED_LINEAR_MATRIX_MULTIPLY_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_RANDOM_GENERATOR_OPERATOR_DESC {
    pub InputStateTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub OutputStateTensor: *const DML_TENSOR_DESC,
    pub Type: DML_RANDOM_GENERATOR_TYPE,
}
impl ::core::marker::Copy for DML_RANDOM_GENERATOR_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_RANDOM_GENERATOR_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_REDUCE_OPERATOR_DESC {
    pub Function: DML_REDUCE_FUNCTION,
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub AxisCount: u32,
    pub Axes: *const u32,
}
impl ::core::marker::Copy for DML_REDUCE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_REDUCE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_RESAMPLE1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub InterpolationMode: DML_INTERPOLATION_MODE,
    pub DimensionCount: u32,
    pub Scales: *const f32,
    pub InputPixelOffsets: *const f32,
    pub OutputPixelOffsets: *const f32,
}
impl ::core::marker::Copy for DML_RESAMPLE1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_RESAMPLE1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_RESAMPLE_GRAD_OPERATOR_DESC {
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
    pub InterpolationMode: DML_INTERPOLATION_MODE,
    pub DimensionCount: u32,
    pub Scales: *const f32,
    pub InputPixelOffsets: *const f32,
    pub OutputPixelOffsets: *const f32,
}
impl ::core::marker::Copy for DML_RESAMPLE_GRAD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_RESAMPLE_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_RESAMPLE_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub InterpolationMode: DML_INTERPOLATION_MODE,
    pub ScaleCount: u32,
    pub Scales: *const f32,
}
impl ::core::marker::Copy for DML_RESAMPLE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_RESAMPLE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_REVERSE_SUBSEQUENCES_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub SequenceLengthsTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
}
impl ::core::marker::Copy for DML_REVERSE_SUBSEQUENCES_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_REVERSE_SUBSEQUENCES_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_RNN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub WeightTensor: *const DML_TENSOR_DESC,
    pub RecurrenceTensor: *const DML_TENSOR_DESC,
    pub BiasTensor: *const DML_TENSOR_DESC,
    pub HiddenInitTensor: *const DML_TENSOR_DESC,
    pub SequenceLengthsTensor: *const DML_TENSOR_DESC,
    pub OutputSequenceTensor: *const DML_TENSOR_DESC,
    pub OutputSingleTensor: *const DML_TENSOR_DESC,
    pub ActivationDescCount: u32,
    pub ActivationDescs: *const DML_OPERATOR_DESC,
    pub Direction: DML_RECURRENT_NETWORK_DIRECTION,
}
impl ::core::marker::Copy for DML_RNN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_RNN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DML_ROI_ALIGN1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ROITensor: *const DML_TENSOR_DESC,
    pub BatchIndicesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ReductionFunction: DML_REDUCE_FUNCTION,
    pub InterpolationMode: DML_INTERPOLATION_MODE,
    pub SpatialScaleX: f32,
    pub SpatialScaleY: f32,
    pub InputPixelOffset: f32,
    pub OutputPixelOffset: f32,
    pub OutOfBoundsInputValue: f32,
    pub MinimumSamplesPerOutput: u32,
    pub MaximumSamplesPerOutput: u32,
    pub AlignRegionsToCorners: super::super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DML_ROI_ALIGN1_OPERATOR_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DML_ROI_ALIGN1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ROI_ALIGN_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ROITensor: *const DML_TENSOR_DESC,
    pub BatchIndicesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ReductionFunction: DML_REDUCE_FUNCTION,
    pub InterpolationMode: DML_INTERPOLATION_MODE,
    pub SpatialScaleX: f32,
    pub SpatialScaleY: f32,
    pub OutOfBoundsInputValue: f32,
    pub MinimumSamplesPerOutput: u32,
    pub MaximumSamplesPerOutput: u32,
}
impl ::core::marker::Copy for DML_ROI_ALIGN_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ROI_ALIGN_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_ROI_POOLING_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub ROITensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub SpatialScale: f32,
    pub PooledSize: DML_SIZE_2D,
}
impl ::core::marker::Copy for DML_ROI_POOLING_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_ROI_POOLING_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub union DML_SCALAR_UNION {
    pub Bytes: [u8; 8],
    pub Int8: i8,
    pub UInt8: u8,
    pub Int16: i16,
    pub UInt16: u16,
    pub Int32: i32,
    pub UInt32: u32,
    pub Int64: i64,
    pub UInt64: u64,
    pub Float32: f32,
    pub Float64: f64,
}
impl ::core::marker::Copy for DML_SCALAR_UNION {}
impl ::core::clone::Clone for DML_SCALAR_UNION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SCALE_BIAS {
    pub Scale: f32,
    pub Bias: f32,
}
impl ::core::marker::Copy for DML_SCALE_BIAS {}
impl ::core::clone::Clone for DML_SCALE_BIAS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SCATTER_ND_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub UpdatesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub InputDimensionCount: u32,
    pub IndicesDimensionCount: u32,
}
impl ::core::marker::Copy for DML_SCATTER_ND_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SCATTER_ND_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SCATTER_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub IndicesTensor: *const DML_TENSOR_DESC,
    pub UpdatesTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
}
impl ::core::marker::Copy for DML_SCATTER_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SCATTER_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SIZE_2D {
    pub Width: u32,
    pub Height: u32,
}
impl ::core::marker::Copy for DML_SIZE_2D {}
impl ::core::clone::Clone for DML_SIZE_2D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SLICE1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub InputWindowOffsets: *const u32,
    pub InputWindowSizes: *const u32,
    pub InputWindowStrides: *const i32,
}
impl ::core::marker::Copy for DML_SLICE1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SLICE1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SLICE_GRAD_OPERATOR_DESC {
    pub InputGradientTensor: *const DML_TENSOR_DESC,
    pub OutputGradientTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub InputWindowOffsets: *const u32,
    pub InputWindowSizes: *const u32,
    pub InputWindowStrides: *const i32,
}
impl ::core::marker::Copy for DML_SLICE_GRAD_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SLICE_GRAD_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SLICE_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub DimensionCount: u32,
    pub Offsets: *const u32,
    pub Sizes: *const u32,
    pub Strides: *const u32,
}
impl ::core::marker::Copy for DML_SLICE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SLICE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SPACE_TO_DEPTH1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub BlockSize: u32,
    pub Order: DML_DEPTH_SPACE_ORDER,
}
impl ::core::marker::Copy for DML_SPACE_TO_DEPTH1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SPACE_TO_DEPTH1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SPACE_TO_DEPTH_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub BlockSize: u32,
}
impl ::core::marker::Copy for DML_SPACE_TO_DEPTH_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SPACE_TO_DEPTH_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_SPLIT_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputCount: u32,
    pub OutputTensors: *const DML_TENSOR_DESC,
    pub Axis: u32,
}
impl ::core::marker::Copy for DML_SPLIT_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_SPLIT_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_TENSOR_DESC {
    pub Type: DML_TENSOR_TYPE,
    pub Desc: *const ::core::ffi::c_void,
}
impl ::core::marker::Copy for DML_TENSOR_DESC {}
impl ::core::clone::Clone for DML_TENSOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_TILE_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub RepeatsCount: u32,
    pub Repeats: *const u32,
}
impl ::core::marker::Copy for DML_TILE_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_TILE_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_TOP_K1_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputValueTensor: *const DML_TENSOR_DESC,
    pub OutputIndexTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
    pub K: u32,
    pub AxisDirection: DML_AXIS_DIRECTION,
}
impl ::core::marker::Copy for DML_TOP_K1_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_TOP_K1_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_TOP_K_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputValueTensor: *const DML_TENSOR_DESC,
    pub OutputIndexTensor: *const DML_TENSOR_DESC,
    pub Axis: u32,
    pub K: u32,
}
impl ::core::marker::Copy for DML_TOP_K_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_TOP_K_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_UPSAMPLE_2D_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub ScaleSize: DML_SIZE_2D,
    pub InterpolationMode: DML_INTERPOLATION_MODE,
}
impl ::core::marker::Copy for DML_UPSAMPLE_2D_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_UPSAMPLE_2D_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_AI_MachineLearning_DirectML\"`*"]
pub struct DML_VALUE_SCALE_2D_OPERATOR_DESC {
    pub InputTensor: *const DML_TENSOR_DESC,
    pub OutputTensor: *const DML_TENSOR_DESC,
    pub Scale: f32,
    pub ChannelCount: u32,
    pub Bias: *const f32,
}
impl ::core::marker::Copy for DML_VALUE_SCALE_2D_OPERATOR_DESC {}
impl ::core::clone::Clone for DML_VALUE_SCALE_2D_OPERATOR_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
