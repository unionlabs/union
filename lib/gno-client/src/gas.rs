use gno_rpc::rpc_types::TxFee;
use num_rational::BigRational;

pub mod any;
pub mod dynamic;
pub mod fixed;

pub trait GasFillerT {
    async fn max_gas(&self) -> u64;

    async fn mk_fee(&self, gas: u64) -> Result<TxFee, crate::BroadcastTxCommitError>;
}

impl<T: GasFillerT> GasFillerT for &T {
    async fn max_gas(&self) -> u64 {
        (*self).max_gas().await
    }

    async fn mk_fee(&self, gas: u64) -> Result<TxFee, crate::BroadcastTxCommitError> {
        (*self).mk_fee(gas).await
    }
}

pub(crate) fn u128_saturating_mul_f64(u: u128, f: f64) -> u128 {
    (BigRational::from_integer(u.into()) * BigRational::from_float(f).expect("finite"))
        .to_integer()
        .try_into()
        .unwrap_or(u128::MAX)
    // .expect("overflow")
}
