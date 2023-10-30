#[cfg(all(feature = "std", not(miri)))]
mod iter;
#[cfg(all(feature = "std", not(miri)))]
mod memchr;
mod simple;
#[cfg(all(feature = "std", not(miri)))]
mod testdata;
