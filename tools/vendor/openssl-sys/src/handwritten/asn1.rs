use libc::*;
use *;

#[repr(C)]
pub struct ASN1_ENCODING {
    pub enc: *mut c_uchar,
    pub len: c_long,
    pub modified: c_int,
}

extern "C" {
    pub fn ASN1_OBJECT_free(x: *mut ASN1_OBJECT);
}

stack!(stack_st_ASN1_OBJECT);

extern "C" {
    pub fn ASN1_STRING_type_new(ty: c_int) -> *mut ASN1_STRING;
    #[cfg(any(ossl110, libressl273))]
    pub fn ASN1_STRING_get0_data(x: *const ASN1_STRING) -> *const c_uchar;
    #[cfg(any(all(ossl101, not(ossl110)), libressl))]
    pub fn ASN1_STRING_data(x: *mut ASN1_STRING) -> *mut c_uchar;

    pub fn ASN1_BIT_STRING_free(x: *mut ASN1_BIT_STRING);

    pub fn ASN1_STRING_free(x: *mut ASN1_STRING);
    pub fn ASN1_STRING_length(x: *const ASN1_STRING) -> c_int;

    pub fn ASN1_STRING_set(x: *mut ASN1_STRING, data: *const c_void, len: c_int) -> c_int;

    pub fn ASN1_GENERALIZEDTIME_free(tm: *mut ASN1_GENERALIZEDTIME);
    pub fn ASN1_GENERALIZEDTIME_print(b: *mut BIO, tm: *const ASN1_GENERALIZEDTIME) -> c_int;
    pub fn ASN1_TIME_new() -> *mut ASN1_TIME;
    #[cfg(ossl102)]
    pub fn ASN1_TIME_diff(
        pday: *mut c_int,
        psec: *mut c_int,
        from: *const ASN1_TIME,
        to: *const ASN1_TIME,
    ) -> c_int;
    pub fn ASN1_TIME_free(tm: *mut ASN1_TIME);
    pub fn ASN1_TIME_print(b: *mut BIO, tm: *const ASN1_TIME) -> c_int;
    pub fn ASN1_TIME_set(from: *mut ASN1_TIME, to: time_t) -> *mut ASN1_TIME;

    pub fn ASN1_INTEGER_free(x: *mut ASN1_INTEGER);
    pub fn ASN1_INTEGER_get(dest: *const ASN1_INTEGER) -> c_long;
    pub fn ASN1_INTEGER_set(dest: *mut ASN1_INTEGER, value: c_long) -> c_int;
    pub fn BN_to_ASN1_INTEGER(bn: *const BIGNUM, ai: *mut ASN1_INTEGER) -> *mut ASN1_INTEGER;
    pub fn ASN1_INTEGER_to_BN(ai: *const ASN1_INTEGER, bn: *mut BIGNUM) -> *mut BIGNUM;

    pub fn ASN1_TIME_set_string(s: *mut ASN1_TIME, str: *const c_char) -> c_int;
    #[cfg(ossl111)]
    pub fn ASN1_TIME_set_string_X509(s: *mut ASN1_TIME, str: *const c_char) -> c_int;
}

const_ptr_api! {
    extern "C" {
        pub fn ASN1_STRING_to_UTF8(out: *mut *mut c_uchar, s: #[const_ptr_if(any(ossl110, libressl280))] ASN1_STRING) -> c_int;
    }
}
