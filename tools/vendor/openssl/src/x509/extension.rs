//! Add extensions to an `X509` certificate or certificate request.
//!
//! The extensions defined for X.509 v3 certificates provide methods for
//! associating additional attributes with users or public keys and for
//! managing relationships between CAs. The extensions created using this
//! module can be used with `X509v3Context` objects.
//!
//! # Example
//!
//! ```rust
//! use openssl::x509::extension::BasicConstraints;
//! use openssl::x509::X509Extension;
//!
//! let mut bc = BasicConstraints::new();
//! let bc = bc.critical().ca().pathlen(1);
//!
//! let extension: X509Extension = bc.build().unwrap();
//! ```
use std::fmt::Write;

use crate::error::ErrorStack;
use crate::nid::Nid;
use crate::x509::{X509Extension, X509v3Context};

/// An extension which indicates whether a certificate is a CA certificate.
pub struct BasicConstraints {
    critical: bool,
    ca: bool,
    pathlen: Option<u32>,
}

impl Default for BasicConstraints {
    fn default() -> BasicConstraints {
        BasicConstraints::new()
    }
}

impl BasicConstraints {
    /// Construct a new `BasicConstraints` extension.
    pub fn new() -> BasicConstraints {
        BasicConstraints {
            critical: false,
            ca: false,
            pathlen: None,
        }
    }

    /// Sets the `critical` flag to `true`. The extension will be critical.
    pub fn critical(&mut self) -> &mut BasicConstraints {
        self.critical = true;
        self
    }

    /// Sets the `ca` flag to `true`.
    pub fn ca(&mut self) -> &mut BasicConstraints {
        self.ca = true;
        self
    }

    /// Sets the `pathlen` to an optional non-negative value. The `pathlen` is the
    /// maximum number of CAs that can appear below this one in a chain.
    pub fn pathlen(&mut self, pathlen: u32) -> &mut BasicConstraints {
        self.pathlen = Some(pathlen);
        self
    }

    /// Return the `BasicConstraints` extension as an `X509Extension`.
    pub fn build(&self) -> Result<X509Extension, ErrorStack> {
        let mut value = String::new();
        if self.critical {
            value.push_str("critical,");
        }
        value.push_str("CA:");
        if self.ca {
            value.push_str("TRUE");
        } else {
            value.push_str("FALSE");
        }
        if let Some(pathlen) = self.pathlen {
            write!(value, ",pathlen:{}", pathlen).unwrap();
        }
        X509Extension::new_nid(None, None, Nid::BASIC_CONSTRAINTS, &value)
    }
}

/// An extension consisting of a list of names of the permitted key usages.
pub struct KeyUsage {
    critical: bool,
    digital_signature: bool,
    non_repudiation: bool,
    key_encipherment: bool,
    data_encipherment: bool,
    key_agreement: bool,
    key_cert_sign: bool,
    crl_sign: bool,
    encipher_only: bool,
    decipher_only: bool,
}

impl Default for KeyUsage {
    fn default() -> KeyUsage {
        KeyUsage::new()
    }
}

impl KeyUsage {
    /// Construct a new `KeyUsage` extension.
    pub fn new() -> KeyUsage {
        KeyUsage {
            critical: false,
            digital_signature: false,
            non_repudiation: false,
            key_encipherment: false,
            data_encipherment: false,
            key_agreement: false,
            key_cert_sign: false,
            crl_sign: false,
            encipher_only: false,
            decipher_only: false,
        }
    }

    /// Sets the `critical` flag to `true`. The extension will be critical.
    pub fn critical(&mut self) -> &mut KeyUsage {
        self.critical = true;
        self
    }

    /// Sets the `digitalSignature` flag to `true`.
    pub fn digital_signature(&mut self) -> &mut KeyUsage {
        self.digital_signature = true;
        self
    }

    /// Sets the `nonRepudiation` flag to `true`.
    pub fn non_repudiation(&mut self) -> &mut KeyUsage {
        self.non_repudiation = true;
        self
    }

    /// Sets the `keyEncipherment` flag to `true`.
    pub fn key_encipherment(&mut self) -> &mut KeyUsage {
        self.key_encipherment = true;
        self
    }

    /// Sets the `dataEncipherment` flag to `true`.
    pub fn data_encipherment(&mut self) -> &mut KeyUsage {
        self.data_encipherment = true;
        self
    }

    /// Sets the `keyAgreement` flag to `true`.
    pub fn key_agreement(&mut self) -> &mut KeyUsage {
        self.key_agreement = true;
        self
    }

    /// Sets the `keyCertSign` flag to `true`.
    pub fn key_cert_sign(&mut self) -> &mut KeyUsage {
        self.key_cert_sign = true;
        self
    }

    /// Sets the `cRLSign` flag to `true`.
    pub fn crl_sign(&mut self) -> &mut KeyUsage {
        self.crl_sign = true;
        self
    }

    /// Sets the `encipherOnly` flag to `true`.
    pub fn encipher_only(&mut self) -> &mut KeyUsage {
        self.encipher_only = true;
        self
    }

    /// Sets the `decipherOnly` flag to `true`.
    pub fn decipher_only(&mut self) -> &mut KeyUsage {
        self.decipher_only = true;
        self
    }

    /// Return the `KeyUsage` extension as an `X509Extension`.
    pub fn build(&self) -> Result<X509Extension, ErrorStack> {
        let mut value = String::new();
        let mut first = true;
        append(&mut value, &mut first, self.critical, "critical");
        append(
            &mut value,
            &mut first,
            self.digital_signature,
            "digitalSignature",
        );
        append(
            &mut value,
            &mut first,
            self.non_repudiation,
            "nonRepudiation",
        );
        append(
            &mut value,
            &mut first,
            self.key_encipherment,
            "keyEncipherment",
        );
        append(
            &mut value,
            &mut first,
            self.data_encipherment,
            "dataEncipherment",
        );
        append(&mut value, &mut first, self.key_agreement, "keyAgreement");
        append(&mut value, &mut first, self.key_cert_sign, "keyCertSign");
        append(&mut value, &mut first, self.crl_sign, "cRLSign");
        append(&mut value, &mut first, self.encipher_only, "encipherOnly");
        append(&mut value, &mut first, self.decipher_only, "decipherOnly");
        X509Extension::new_nid(None, None, Nid::KEY_USAGE, &value)
    }
}

/// An extension consisting of a list of usages indicating purposes
/// for which the certificate public key can be used for.
pub struct ExtendedKeyUsage {
    critical: bool,
    server_auth: bool,
    client_auth: bool,
    code_signing: bool,
    email_protection: bool,
    time_stamping: bool,
    ms_code_ind: bool,
    ms_code_com: bool,
    ms_ctl_sign: bool,
    ms_sgc: bool,
    ms_efs: bool,
    ns_sgc: bool,
    other: Vec<String>,
}

impl Default for ExtendedKeyUsage {
    fn default() -> ExtendedKeyUsage {
        ExtendedKeyUsage::new()
    }
}

impl ExtendedKeyUsage {
    /// Construct a new `ExtendedKeyUsage` extension.
    pub fn new() -> ExtendedKeyUsage {
        ExtendedKeyUsage {
            critical: false,
            server_auth: false,
            client_auth: false,
            code_signing: false,
            email_protection: false,
            time_stamping: false,
            ms_code_ind: false,
            ms_code_com: false,
            ms_ctl_sign: false,
            ms_sgc: false,
            ms_efs: false,
            ns_sgc: false,
            other: vec![],
        }
    }

    /// Sets the `critical` flag to `true`. The extension will be critical.
    pub fn critical(&mut self) -> &mut ExtendedKeyUsage {
        self.critical = true;
        self
    }

    /// Sets the `serverAuth` flag to `true`.
    pub fn server_auth(&mut self) -> &mut ExtendedKeyUsage {
        self.server_auth = true;
        self
    }

    /// Sets the `clientAuth` flag to `true`.
    pub fn client_auth(&mut self) -> &mut ExtendedKeyUsage {
        self.client_auth = true;
        self
    }

    /// Sets the `codeSigning` flag to `true`.
    pub fn code_signing(&mut self) -> &mut ExtendedKeyUsage {
        self.code_signing = true;
        self
    }

    /// Sets the `emailProtection` flag to `true`.
    pub fn email_protection(&mut self) -> &mut ExtendedKeyUsage {
        self.email_protection = true;
        self
    }

    /// Sets the `timeStamping` flag to `true`.
    pub fn time_stamping(&mut self) -> &mut ExtendedKeyUsage {
        self.time_stamping = true;
        self
    }

    /// Sets the `msCodeInd` flag to `true`.
    pub fn ms_code_ind(&mut self) -> &mut ExtendedKeyUsage {
        self.ms_code_ind = true;
        self
    }

    /// Sets the `msCodeCom` flag to `true`.
    pub fn ms_code_com(&mut self) -> &mut ExtendedKeyUsage {
        self.ms_code_com = true;
        self
    }

    /// Sets the `msCTLSign` flag to `true`.
    pub fn ms_ctl_sign(&mut self) -> &mut ExtendedKeyUsage {
        self.ms_ctl_sign = true;
        self
    }

    /// Sets the `msSGC` flag to `true`.
    pub fn ms_sgc(&mut self) -> &mut ExtendedKeyUsage {
        self.ms_sgc = true;
        self
    }

    /// Sets the `msEFS` flag to `true`.
    pub fn ms_efs(&mut self) -> &mut ExtendedKeyUsage {
        self.ms_efs = true;
        self
    }

    /// Sets the `nsSGC` flag to `true`.
    pub fn ns_sgc(&mut self) -> &mut ExtendedKeyUsage {
        self.ns_sgc = true;
        self
    }

    /// Sets a flag not already defined.
    pub fn other(&mut self, other: &str) -> &mut ExtendedKeyUsage {
        self.other.push(other.to_owned());
        self
    }

    /// Return the `ExtendedKeyUsage` extension as an `X509Extension`.
    pub fn build(&self) -> Result<X509Extension, ErrorStack> {
        let mut value = String::new();
        let mut first = true;
        append(&mut value, &mut first, self.critical, "critical");
        append(&mut value, &mut first, self.server_auth, "serverAuth");
        append(&mut value, &mut first, self.client_auth, "clientAuth");
        append(&mut value, &mut first, self.code_signing, "codeSigning");
        append(
            &mut value,
            &mut first,
            self.email_protection,
            "emailProtection",
        );
        append(&mut value, &mut first, self.time_stamping, "timeStamping");
        append(&mut value, &mut first, self.ms_code_ind, "msCodeInd");
        append(&mut value, &mut first, self.ms_code_com, "msCodeCom");
        append(&mut value, &mut first, self.ms_ctl_sign, "msCTLSign");
        append(&mut value, &mut first, self.ms_sgc, "msSGC");
        append(&mut value, &mut first, self.ms_efs, "msEFS");
        append(&mut value, &mut first, self.ns_sgc, "nsSGC");
        for other in &self.other {
            append(&mut value, &mut first, true, other);
        }
        X509Extension::new_nid(None, None, Nid::EXT_KEY_USAGE, &value)
    }
}

/// An extension that provides a means of identifying certificates that contain a
/// particular public key.
pub struct SubjectKeyIdentifier {
    critical: bool,
}

impl Default for SubjectKeyIdentifier {
    fn default() -> SubjectKeyIdentifier {
        SubjectKeyIdentifier::new()
    }
}

impl SubjectKeyIdentifier {
    /// Construct a new `SubjectKeyIdentifier` extension.
    pub fn new() -> SubjectKeyIdentifier {
        SubjectKeyIdentifier { critical: false }
    }

    /// Sets the `critical` flag to `true`. The extension will be critical.
    pub fn critical(&mut self) -> &mut SubjectKeyIdentifier {
        self.critical = true;
        self
    }

    /// Return a `SubjectKeyIdentifier` extension as an `X509Extension`.
    pub fn build(&self, ctx: &X509v3Context<'_>) -> Result<X509Extension, ErrorStack> {
        let mut value = String::new();
        let mut first = true;
        append(&mut value, &mut first, self.critical, "critical");
        append(&mut value, &mut first, true, "hash");
        X509Extension::new_nid(None, Some(ctx), Nid::SUBJECT_KEY_IDENTIFIER, &value)
    }
}

/// An extension that provides a means of identifying the public key corresponding
/// to the private key used to sign a CRL.
pub struct AuthorityKeyIdentifier {
    critical: bool,
    keyid: Option<bool>,
    issuer: Option<bool>,
}

impl Default for AuthorityKeyIdentifier {
    fn default() -> AuthorityKeyIdentifier {
        AuthorityKeyIdentifier::new()
    }
}

impl AuthorityKeyIdentifier {
    /// Construct a new `AuthorityKeyIdentifier` extension.
    pub fn new() -> AuthorityKeyIdentifier {
        AuthorityKeyIdentifier {
            critical: false,
            keyid: None,
            issuer: None,
        }
    }

    /// Sets the `critical` flag to `true`. The extension will be critical.
    pub fn critical(&mut self) -> &mut AuthorityKeyIdentifier {
        self.critical = true;
        self
    }

    /// Sets the `keyid` flag.
    pub fn keyid(&mut self, always: bool) -> &mut AuthorityKeyIdentifier {
        self.keyid = Some(always);
        self
    }

    /// Sets the `issuer` flag.
    pub fn issuer(&mut self, always: bool) -> &mut AuthorityKeyIdentifier {
        self.issuer = Some(always);
        self
    }

    /// Return a `AuthorityKeyIdentifier` extension as an `X509Extension`.
    pub fn build(&self, ctx: &X509v3Context<'_>) -> Result<X509Extension, ErrorStack> {
        let mut value = String::new();
        let mut first = true;
        append(&mut value, &mut first, self.critical, "critical");
        match self.keyid {
            Some(true) => append(&mut value, &mut first, true, "keyid:always"),
            Some(false) => append(&mut value, &mut first, true, "keyid"),
            None => {}
        }
        match self.issuer {
            Some(true) => append(&mut value, &mut first, true, "issuer:always"),
            Some(false) => append(&mut value, &mut first, true, "issuer"),
            None => {}
        }
        X509Extension::new_nid(None, Some(ctx), Nid::AUTHORITY_KEY_IDENTIFIER, &value)
    }
}

/// An extension that allows additional identities to be bound to the subject
/// of the certificate.
pub struct SubjectAlternativeName {
    critical: bool,
    names: Vec<String>,
}

impl Default for SubjectAlternativeName {
    fn default() -> SubjectAlternativeName {
        SubjectAlternativeName::new()
    }
}

impl SubjectAlternativeName {
    /// Construct a new `SubjectAlternativeName` extension.
    pub fn new() -> SubjectAlternativeName {
        SubjectAlternativeName {
            critical: false,
            names: vec![],
        }
    }

    /// Sets the `critical` flag to `true`. The extension will be critical.
    pub fn critical(&mut self) -> &mut SubjectAlternativeName {
        self.critical = true;
        self
    }

    /// Sets the `email` flag.
    pub fn email(&mut self, email: &str) -> &mut SubjectAlternativeName {
        self.names.push(format!("email:{}", email));
        self
    }

    /// Sets the `uri` flag.
    pub fn uri(&mut self, uri: &str) -> &mut SubjectAlternativeName {
        self.names.push(format!("URI:{}", uri));
        self
    }

    /// Sets the `dns` flag.
    pub fn dns(&mut self, dns: &str) -> &mut SubjectAlternativeName {
        self.names.push(format!("DNS:{}", dns));
        self
    }

    /// Sets the `rid` flag.
    pub fn rid(&mut self, rid: &str) -> &mut SubjectAlternativeName {
        self.names.push(format!("RID:{}", rid));
        self
    }

    /// Sets the `ip` flag.
    pub fn ip(&mut self, ip: &str) -> &mut SubjectAlternativeName {
        self.names.push(format!("IP:{}", ip));
        self
    }

    /// Sets the `dirName` flag.
    pub fn dir_name(&mut self, dir_name: &str) -> &mut SubjectAlternativeName {
        self.names.push(format!("dirName:{}", dir_name));
        self
    }

    /// Sets the `otherName` flag.
    pub fn other_name(&mut self, other_name: &str) -> &mut SubjectAlternativeName {
        self.names.push(format!("otherName:{}", other_name));
        self
    }

    /// Return a `SubjectAlternativeName` extension as an `X509Extension`.
    pub fn build(&self, ctx: &X509v3Context<'_>) -> Result<X509Extension, ErrorStack> {
        let mut value = String::new();
        let mut first = true;
        append(&mut value, &mut first, self.critical, "critical");
        for name in &self.names {
            append(&mut value, &mut first, true, name);
        }
        X509Extension::new_nid(None, Some(ctx), Nid::SUBJECT_ALT_NAME, &value)
    }
}

fn append(value: &mut String, first: &mut bool, should: bool, element: &str) {
    if !should {
        return;
    }

    if !*first {
        value.push(',');
    }
    *first = false;
    value.push_str(element);
}
