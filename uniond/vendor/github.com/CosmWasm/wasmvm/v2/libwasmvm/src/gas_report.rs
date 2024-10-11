#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct GasReport {
    /// The original limit the instance was created with
    pub limit: u64,
    /// The remaining gas that can be spend
    pub remaining: u64,
    /// The amount of gas that was spend and metered externally in operations triggered by this instance
    pub used_externally: u64,
    /// The amount of gas that was spend and metered internally (i.e. by executing Wasm and calling
    /// API methods which are not metered externally)
    pub used_internally: u64,
}

impl From<cosmwasm_vm::GasReport> for GasReport {
    fn from(
        cosmwasm_vm::GasReport {
            limit,
            remaining,
            used_externally,
            used_internally,
        }: cosmwasm_vm::GasReport,
    ) -> Self {
        Self {
            limit,
            remaining,
            used_externally,
            used_internally,
        }
    }
}
