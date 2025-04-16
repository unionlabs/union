use num_rational::BigRational;
use unionlabs::cosmos::tx::fee::Fee;

pub mod any;
pub mod feemarket;
pub mod fixed;
pub mod osmosis_eip1559_feemarket;

pub trait GasFillerT {
    async fn max_gas(&self) -> u64;

    async fn mk_fee(&self, gas: u64) -> Fee;
}

impl<T: GasFillerT> GasFillerT for &T {
    async fn max_gas(&self) -> u64 {
        (*self).max_gas().await
    }

    async fn mk_fee(&self, gas: u64) -> Fee {
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
