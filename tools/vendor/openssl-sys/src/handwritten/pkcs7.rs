use libc::*;
use *;

pub enum PKCS7_SIGNED {}
pub enum PKCS7_ENVELOPE {}
pub enum PKCS7_SIGN_ENVELOPE {}
pub enum PKCS7_DIGEST {}
pub enum PKCS7_ENCRYPT {}
pub enum PKCS7 {}

extern "C" {
    pub fn d2i_PKCS7(a: *mut *mut PKCS7, pp: *mut *const c_uchar, length: c_long) -> *mut PKCS7;
}

const_ptr_api! {
    extern "C" {
        pub fn i2d_PKCS7(a: #[const_ptr_if(ossl300)] PKCS7, buf: *mut *mut u8) -> c_int;
    }
}

extern "C" {
    pub fn PKCS7_encrypt(
        certs: *mut stack_st_X509,
        b: *mut BIO,
        cipher: *const EVP_CIPHER,
        flags: c_int,
    ) -> *mut PKCS7;

    pub fn PKCS7_verify(
        pkcs7: *mut PKCS7,
        certs: *mut stack_st_X509,
        store: *mut X509_STORE,
        indata: *mut BIO,
        out: *mut BIO,
        flags: c_int,
    ) -> c_int;

    pub fn PKCS7_get0_signers(
        pkcs7: *mut PKCS7,
        certs: *mut stack_st_X509,
        flags: c_int,
    ) -> *mut stack_st_X509;

    pub fn PKCS7_sign(
        signcert: *mut X509,
        pkey: *mut EVP_PKEY,
        certs: *mut stack_st_X509,
        data: *mut BIO,
        flags: c_int,
    ) -> *mut PKCS7;

    pub fn PKCS7_decrypt(
        pkcs7: *mut PKCS7,
        pkey: *mut EVP_PKEY,
        cert: *mut X509,
        data: *mut BIO,
        flags: c_int,
    ) -> c_int;

    pub fn PKCS7_free(pkcs7: *mut PKCS7);

    pub fn SMIME_write_PKCS7(
        out: *mut BIO,
        pkcs7: *mut PKCS7,
        data: *mut BIO,
        flags: c_int,
    ) -> c_int;

    pub fn SMIME_read_PKCS7(bio: *mut BIO, bcont: *mut *mut BIO) -> *mut PKCS7;
}
