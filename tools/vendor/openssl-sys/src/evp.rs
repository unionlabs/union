use libc::*;
use *;

pub const EVP_MAX_MD_SIZE: c_uint = 64;

pub const PKCS5_SALT_LEN: c_int = 8;
pub const PKCS12_DEFAULT_ITER: c_int = 2048;

pub const EVP_PKEY_RSA: c_int = NID_rsaEncryption;
pub const EVP_PKEY_DSA: c_int = NID_dsa;
pub const EVP_PKEY_DH: c_int = NID_dhKeyAgreement;
pub const EVP_PKEY_EC: c_int = NID_X9_62_id_ecPublicKey;
#[cfg(ossl111)]
pub const EVP_PKEY_X25519: c_int = NID_X25519;
#[cfg(ossl111)]
pub const EVP_PKEY_ED25519: c_int = NID_ED25519;
#[cfg(ossl111)]
pub const EVP_PKEY_X448: c_int = NID_X448;
#[cfg(ossl111)]
pub const EVP_PKEY_ED448: c_int = NID_ED448;
pub const EVP_PKEY_HMAC: c_int = NID_hmac;
pub const EVP_PKEY_CMAC: c_int = NID_cmac;
#[cfg(ossl110)]
pub const EVP_PKEY_HKDF: c_int = NID_hkdf;

pub const EVP_CTRL_GCM_SET_IVLEN: c_int = 0x9;
pub const EVP_CTRL_GCM_GET_TAG: c_int = 0x10;
pub const EVP_CTRL_GCM_SET_TAG: c_int = 0x11;

pub unsafe fn EVP_get_digestbynid(type_: c_int) -> *const EVP_MD {
    EVP_get_digestbyname(OBJ_nid2sn(type_))
}

cfg_if! {
    if #[cfg(ossl300)] {
        #[inline]
        pub unsafe fn EVP_MD_block_size(md: *const EVP_MD) -> c_int {
            EVP_MD_get_block_size(md)
        }

        #[inline]
        pub unsafe fn EVP_MD_size(md: *const EVP_MD) -> c_int {
            EVP_MD_get_size(md)
        }

        #[inline]
        pub unsafe fn EVP_MD_type(md: *const EVP_MD) -> c_int {
            EVP_MD_get_type(md)
        }

        #[inline]
        pub unsafe fn EVP_CIPHER_key_length(cipher: *const EVP_CIPHER) -> c_int {
            EVP_CIPHER_get_key_length(cipher)
        }

        #[inline]
        pub unsafe fn EVP_CIPHER_block_size(cipher: *const EVP_CIPHER) -> c_int {
            EVP_CIPHER_get_block_size(cipher)
        }

        #[inline]
        pub unsafe fn EVP_CIPHER_iv_length(cipher: *const EVP_CIPHER) -> c_int {
            EVP_CIPHER_get_iv_length(cipher)
        }

        #[inline]
        pub unsafe fn EVP_CIPHER_nid(cipher: *const EVP_CIPHER) -> c_int {
            EVP_CIPHER_get_nid(cipher)
        }

        #[inline]
        pub unsafe fn EVP_CIPHER_CTX_block_size(ctx: *const EVP_CIPHER_CTX) -> c_int {
            EVP_CIPHER_CTX_get_block_size(ctx)
        }

        #[inline]
        pub unsafe fn EVP_CIPHER_CTX_key_length(ctx: *const EVP_CIPHER_CTX) -> c_int {
            EVP_CIPHER_CTX_get_key_length(ctx)
        }

        #[inline]
        pub unsafe fn EVP_CIPHER_CTX_iv_length(ctx: *const EVP_CIPHER_CTX) -> c_int {
            EVP_CIPHER_CTX_get_iv_length(ctx)
        }
    }
}
#[cfg(not(ossl300))]
#[inline]
pub unsafe fn EVP_DigestSignUpdate(
    ctx: *mut EVP_MD_CTX,
    data: *const c_void,
    dsize: size_t,
) -> c_int {
    EVP_DigestUpdate(ctx, data, dsize)
}
#[cfg(not(ossl300))]
#[inline]
pub unsafe fn EVP_DigestVerifyUpdate(
    ctx: *mut EVP_MD_CTX,
    data: *const c_void,
    dsize: size_t,
) -> c_int {
    EVP_DigestUpdate(ctx, data, dsize)
}
#[cfg(ossl300)]
#[inline]
pub unsafe fn EVP_PKEY_size(pkey: *const EVP_PKEY) -> c_int {
    EVP_PKEY_get_size(pkey)
}

cfg_if! {
    if #[cfg(ossl300)] {
        #[inline]
        pub unsafe fn EVP_PKEY_id(pkey: *const EVP_PKEY) -> c_int {
            EVP_PKEY_get_id(pkey)
        }

        #[inline]
        pub unsafe fn EVP_PKEY_bits(pkey: *const EVP_PKEY) -> c_int {
            EVP_PKEY_get_bits(pkey)
        }
    }
}

pub const EVP_PKEY_OP_KEYGEN: c_int = 1 << 2;
cfg_if! {
    if #[cfg(ossl300)] {
        pub const EVP_PKEY_OP_SIGN: c_int = 1 << 4;
        pub const EVP_PKEY_OP_VERIFY: c_int = 1 << 5;
        pub const EVP_PKEY_OP_VERIFYRECOVER: c_int = 1 << 6;
        pub const EVP_PKEY_OP_SIGNCTX: c_int = 1 << 7;
        pub const EVP_PKEY_OP_VERIFYCTX: c_int = 1 << 8;
        pub const EVP_PKEY_OP_ENCRYPT: c_int = 1 << 9;
        pub const EVP_PKEY_OP_DECRYPT: c_int = 1 << 10;
        pub const EVP_PKEY_OP_DERIVE: c_int = 1 << 11;
    } else {
        pub const EVP_PKEY_OP_SIGN: c_int = 1 << 3;
        pub const EVP_PKEY_OP_VERIFY: c_int = 1 << 4;
        pub const EVP_PKEY_OP_VERIFYRECOVER: c_int = 1 << 5;
        pub const EVP_PKEY_OP_SIGNCTX: c_int = 1 << 6;
        pub const EVP_PKEY_OP_VERIFYCTX: c_int = 1 << 7;
        pub const EVP_PKEY_OP_ENCRYPT: c_int = 1 << 8;
        pub const EVP_PKEY_OP_DECRYPT: c_int = 1 << 9;
        pub const EVP_PKEY_OP_DERIVE: c_int = 1 << 10;
    }
}

pub const EVP_PKEY_OP_TYPE_SIG: c_int = EVP_PKEY_OP_SIGN
    | EVP_PKEY_OP_VERIFY
    | EVP_PKEY_OP_VERIFYRECOVER
    | EVP_PKEY_OP_SIGNCTX
    | EVP_PKEY_OP_VERIFYCTX;

pub const EVP_PKEY_OP_TYPE_CRYPT: c_int = EVP_PKEY_OP_ENCRYPT | EVP_PKEY_OP_DECRYPT;

pub const EVP_PKEY_CTRL_SET_MAC_KEY: c_int = 6;

pub const EVP_PKEY_CTRL_CIPHER: c_int = 12;

pub const EVP_PKEY_ALG_CTRL: c_int = 0x1000;

#[cfg(ossl111)]
pub const EVP_PKEY_HKDEF_MODE_EXTRACT_AND_EXPAND: c_int = 0;

#[cfg(ossl111)]
pub const EVP_PKEY_HKDEF_MODE_EXTRACT_ONLY: c_int = 1;

#[cfg(ossl111)]
pub const EVP_PKEY_HKDEF_MODE_EXPAND_ONLY: c_int = 2;

#[cfg(ossl110)]
pub const EVP_PKEY_CTRL_HKDF_MD: c_int = EVP_PKEY_ALG_CTRL + 3;

#[cfg(ossl110)]
pub const EVP_PKEY_CTRL_HKDF_SALT: c_int = EVP_PKEY_ALG_CTRL + 4;

#[cfg(ossl110)]
pub const EVP_PKEY_CTRL_HKDF_KEY: c_int = EVP_PKEY_ALG_CTRL + 5;

#[cfg(ossl110)]
pub const EVP_PKEY_CTRL_HKDF_INFO: c_int = EVP_PKEY_ALG_CTRL + 6;

#[cfg(ossl111)]
pub const EVP_PKEY_CTRL_HKDF_MODE: c_int = EVP_PKEY_ALG_CTRL + 7;

#[cfg(all(ossl111, not(ossl300)))]
pub unsafe fn EVP_PKEY_CTX_set_hkdf_mode(ctx: *mut EVP_PKEY_CTX, mode: c_int) -> c_int {
    EVP_PKEY_CTX_ctrl(
        ctx,
        -1,
        EVP_PKEY_OP_DERIVE,
        EVP_PKEY_CTRL_HKDF_MODE,
        mode,
        std::ptr::null_mut(),
    )
}

#[cfg(all(ossl110, not(ossl300)))]
pub unsafe fn EVP_PKEY_CTX_set_hkdf_md(ctx: *mut EVP_PKEY_CTX, md: *const EVP_MD) -> c_int {
    EVP_PKEY_CTX_ctrl(
        ctx,
        -1,
        EVP_PKEY_OP_DERIVE,
        EVP_PKEY_CTRL_HKDF_MD,
        0,
        md as *mut c_void,
    )
}

#[cfg(all(ossl110, not(ossl300)))]
pub unsafe fn EVP_PKEY_CTX_set1_hkdf_salt(
    ctx: *mut EVP_PKEY_CTX,
    salt: *const u8,
    saltlen: c_int,
) -> c_int {
    EVP_PKEY_CTX_ctrl(
        ctx,
        -1,
        EVP_PKEY_OP_DERIVE,
        EVP_PKEY_CTRL_HKDF_SALT,
        saltlen,
        salt as *mut c_void,
    )
}

#[cfg(all(ossl110, not(ossl300)))]
pub unsafe fn EVP_PKEY_CTX_set1_hkdf_key(
    ctx: *mut EVP_PKEY_CTX,
    key: *const u8,
    keylen: c_int,
) -> c_int {
    EVP_PKEY_CTX_ctrl(
        ctx,
        -1,
        EVP_PKEY_OP_DERIVE,
        EVP_PKEY_CTRL_HKDF_KEY,
        keylen,
        key as *mut c_void,
    )
}

#[cfg(all(ossl110, not(ossl300)))]
pub unsafe fn EVP_PKEY_CTX_add1_hkdf_info(
    ctx: *mut EVP_PKEY_CTX,
    info: *const u8,
    infolen: c_int,
) -> c_int {
    EVP_PKEY_CTX_ctrl(
        ctx,
        -1,
        EVP_PKEY_OP_DERIVE,
        EVP_PKEY_CTRL_HKDF_INFO,
        infolen,
        info as *mut c_void,
    )
}
