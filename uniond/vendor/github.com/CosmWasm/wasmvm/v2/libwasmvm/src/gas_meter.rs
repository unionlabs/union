/// An opaque type. `*gas_meter_t` represents a pointer to Go memory holding the gas meter.
#[repr(C)]
pub struct gas_meter_t {
    _private: [u8; 0],
}
