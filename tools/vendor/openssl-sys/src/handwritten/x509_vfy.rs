use libc::*;
use *;

#[cfg(any(libressl, all(ossl102, not(ossl110))))]
pub enum X509_VERIFY_PARAM_ID {}

extern "C" {
    #[cfg(ossl110)]
    pub fn X509_LOOKUP_meth_free(method: *mut X509_LOOKUP_METHOD);
}

extern "C" {
    pub fn X509_LOOKUP_free(ctx: *mut X509_LOOKUP);
    pub fn X509_LOOKUP_hash_dir() -> *mut X509_LOOKUP_METHOD;
    pub fn X509_LOOKUP_ctrl(
        ctx: *mut X509_LOOKUP,
        cmd: c_int,
        argc: *const c_char,
        argl: c_long,
        ret: *mut *mut c_char,
    ) -> c_int;
}

extern "C" {
    pub fn X509_STORE_new() -> *mut X509_STORE;
    pub fn X509_STORE_free(store: *mut X509_STORE);

    pub fn X509_STORE_CTX_new() -> *mut X509_STORE_CTX;

    pub fn X509_STORE_CTX_free(ctx: *mut X509_STORE_CTX);
    pub fn X509_STORE_CTX_init(
        ctx: *mut X509_STORE_CTX,
        store: *mut X509_STORE,
        x509: *mut X509,
        chain: *mut stack_st_X509,
    ) -> c_int;
    pub fn X509_STORE_CTX_cleanup(ctx: *mut X509_STORE_CTX);

    pub fn X509_STORE_add_cert(store: *mut X509_STORE, x: *mut X509) -> c_int;

    pub fn X509_STORE_add_lookup(
        store: *mut X509_STORE,
        meth: *mut X509_LOOKUP_METHOD,
    ) -> *mut X509_LOOKUP;

    pub fn X509_STORE_set_default_paths(store: *mut X509_STORE) -> c_int;
    pub fn X509_STORE_set_flags(store: *mut X509_STORE, flags: c_ulong) -> c_int;
}

const_ptr_api! {
    extern "C" {
        pub fn X509_STORE_CTX_get_ex_data(ctx: #[const_ptr_if(ossl300)] X509_STORE_CTX, idx: c_int) -> *mut c_void;
        pub fn X509_STORE_CTX_get_error(ctx: #[const_ptr_if(ossl300)] X509_STORE_CTX) -> c_int;
        pub fn X509_STORE_CTX_get_error_depth(ctx: #[const_ptr_if(ossl300)] X509_STORE_CTX) -> c_int;
        pub fn X509_STORE_CTX_get_current_cert(ctx: #[const_ptr_if(ossl300)] X509_STORE_CTX) -> *mut X509;
    }
}
extern "C" {
    pub fn X509_STORE_CTX_set_error(ctx: *mut X509_STORE_CTX, error: c_int);
}
cfg_if! {
    if #[cfg(any(ossl110, libressl350))] {
        const_ptr_api! {
            extern "C" {
                pub fn X509_STORE_CTX_get0_chain(ctx: #[const_ptr_if(ossl300)] X509_STORE_CTX) -> *mut stack_st_X509;
            }
        }
    } else {
        extern "C" {
            pub fn X509_STORE_CTX_get_chain(ctx: *mut X509_STORE_CTX) -> *mut stack_st_X509;
        }
    }
}

extern "C" {
    #[cfg(any(ossl102, libressl261))]
    pub fn X509_VERIFY_PARAM_free(param: *mut X509_VERIFY_PARAM);

    #[cfg(any(ossl102, libressl261))]
    pub fn X509_VERIFY_PARAM_set_flags(param: *mut X509_VERIFY_PARAM, flags: c_ulong) -> c_int;
    #[cfg(any(ossl102, libressl261))]
    pub fn X509_VERIFY_PARAM_clear_flags(param: *mut X509_VERIFY_PARAM, flags: c_ulong) -> c_int;
}
const_ptr_api! {
    extern "C" {
        #[cfg(any(ossl102, libressl261))]
        pub fn X509_VERIFY_PARAM_get_flags(param: #[const_ptr_if(ossl300)] X509_VERIFY_PARAM) -> c_ulong;
    }
}

extern "C" {
    #[cfg(any(ossl102, libressl261))]
    pub fn X509_VERIFY_PARAM_set1_host(
        param: *mut X509_VERIFY_PARAM,
        name: *const c_char,
        namelen: size_t,
    ) -> c_int;
    #[cfg(any(ossl102, libressl261))]
    pub fn X509_VERIFY_PARAM_set_hostflags(param: *mut X509_VERIFY_PARAM, flags: c_uint);
    #[cfg(any(ossl102, libressl261))]
    pub fn X509_VERIFY_PARAM_set1_ip(
        param: *mut X509_VERIFY_PARAM,
        ip: *const c_uchar,
        iplen: size_t,
    ) -> c_int;
}
