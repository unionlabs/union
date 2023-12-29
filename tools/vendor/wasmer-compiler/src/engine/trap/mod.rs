mod frame_info;
mod stack;
pub use frame_info::{
    register as register_frame_info, FunctionExtent, GlobalFrameInfoRegistration, FRAME_INFO,
};
pub use stack::get_trace_and_trapcode;
