use libc::*;
use *;

cfg_if! {
    if #[cfg(ossl300)] {
        extern "C" {
            pub fn EVP_PKEY_CTX_set_hkdf_mode(ctx: *mut EVP_PKEY_CTX, mode: c_int) -> c_int;
            pub fn EVP_PKEY_CTX_set_hkdf_md(ctx: *mut EVP_PKEY_CTX, md: *const EVP_MD) -> c_int;
            pub fn EVP_PKEY_CTX_set1_hkdf_salt(
                ctx: *mut EVP_PKEY_CTX,
                salt: *const u8,
                saltlen: c_int,
            ) -> c_int;
            pub fn EVP_PKEY_CTX_set1_hkdf_key(
                ctx: *mut EVP_PKEY_CTX,
                key: *const u8,
                keylen: c_int,
            ) -> c_int;
            pub fn EVP_PKEY_CTX_add1_hkdf_info(
                ctx: *mut EVP_PKEY_CTX,
                info: *const u8,
                infolen: c_int,
            ) -> c_int;
        }
    }
}
