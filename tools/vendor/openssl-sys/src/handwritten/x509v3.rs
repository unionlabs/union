use libc::*;
use *;

pub enum CONF_METHOD {}

extern "C" {
    pub fn GENERAL_NAME_free(name: *mut GENERAL_NAME);
}

#[repr(C)]
pub struct ACCESS_DESCRIPTION {
    pub method: *mut ASN1_OBJECT,
    pub location: *mut GENERAL_NAME,
}

stack!(stack_st_ACCESS_DESCRIPTION);

extern "C" {
    pub fn ACCESS_DESCRIPTION_free(ad: *mut ACCESS_DESCRIPTION);
}

#[repr(C)]
pub struct AUTHORITY_KEYID {
    pub keyid: *mut ASN1_OCTET_STRING,
    pub issuer: *mut stack_st_GENERAL_NAME,
    pub serial: *mut ASN1_INTEGER,
}

extern "C" {
    pub fn AUTHORITY_KEYID_free(akid: *mut AUTHORITY_KEYID);
}

const_ptr_api! {
    extern "C" {
        pub fn X509V3_EXT_nconf_nid(
            conf: *mut CONF,
            ctx: *mut X509V3_CTX,
            ext_nid: c_int,
            value: #[const_ptr_if(any(ossl110, libressl280))] c_char,
        ) -> *mut X509_EXTENSION;
        pub fn X509V3_EXT_nconf(
            conf: *mut CONF,
            ctx: *mut X509V3_CTX,
            name: #[const_ptr_if(any(ossl110, libressl280))] c_char,
            value: #[const_ptr_if(any(ossl110, libressl280))] c_char,
        ) -> *mut X509_EXTENSION;
    }
}

extern "C" {
    pub fn X509_check_issued(issuer: *mut X509, subject: *mut X509) -> c_int;
    pub fn X509_verify(req: *mut X509, pkey: *mut EVP_PKEY) -> c_int;

    pub fn X509V3_set_nconf(ctx: *mut X509V3_CTX, conf: *mut CONF);

    pub fn X509V3_set_ctx(
        ctx: *mut X509V3_CTX,
        issuer: *mut X509,
        subject: *mut X509,
        req: *mut X509_REQ,
        crl: *mut X509_CRL,
        flags: c_int,
    );

    pub fn X509_get1_ocsp(x: *mut X509) -> *mut stack_st_OPENSSL_STRING;
}

const_ptr_api! {
    extern "C" {
        pub fn X509V3_get_d2i(
            x: #[const_ptr_if(any(ossl110, libressl280))] stack_st_X509_EXTENSION,
            nid: c_int,
            crit: *mut c_int,
            idx: *mut c_int,
        ) -> *mut c_void;
        pub fn X509V3_extensions_print(out: *mut BIO, title: #[const_ptr_if(any(ossl110, libressl280))] c_char, exts: #[const_ptr_if(any(ossl110, libressl280))] stack_st_X509_EXTENSION, flag: c_ulong, indent: c_int) -> c_int;
    }
}

extern "C" {
    pub fn X509V3_EXT_add_alias(nid_to: c_int, nid_from: c_int) -> c_int;
    pub fn X509V3_EXT_d2i(ext: *mut X509_EXTENSION) -> *mut c_void;
    pub fn X509V3_EXT_i2d(ext_nid: c_int, crit: c_int, ext: *mut c_void) -> *mut X509_EXTENSION;
    pub fn X509V3_add1_i2d(
        x: *mut *mut stack_st_X509_EXTENSION,
        nid: c_int,
        value: *mut c_void,
        crit: c_int,
        flags: c_ulong,
    ) -> c_int;
    pub fn X509V3_EXT_print(
        out: *mut BIO,
        ext: *mut X509_EXTENSION,
        flag: c_ulong,
        indent: c_int,
    ) -> c_int;

    #[cfg(ossl110)]
    pub fn X509_get_extension_flags(x: *mut X509) -> u32;
    #[cfg(ossl110)]
    pub fn X509_get_key_usage(x: *mut X509) -> u32;
    #[cfg(ossl110)]
    pub fn X509_get_extended_key_usage(x: *mut X509) -> u32;
}
