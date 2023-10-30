use libc::*;
use *;

extern "C" {
    #[cfg(ossl300)]
    pub fn OSSL_PROVIDER_load(ctx: *mut OSSL_LIB_CTX, name: *const c_char) -> *mut OSSL_PROVIDER;
    #[cfg(ossl300)]
    pub fn OSSL_PROVIDER_try_load(
        ctx: *mut OSSL_LIB_CTX,
        name: *const c_char,
        retain_fallbacks: c_int,
    ) -> *mut OSSL_PROVIDER;
    #[cfg(ossl300)]
    pub fn OSSL_PROVIDER_unload(prov: *mut OSSL_PROVIDER) -> c_int;
}
