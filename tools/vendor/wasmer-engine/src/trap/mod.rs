mod error;
mod frame_info;
pub use error::RuntimeError;
pub use frame_info::{
    register as register_frame_info, FrameInfo, FunctionExtent, GlobalFrameInfoRegistration,
    FRAME_INFO,
};
