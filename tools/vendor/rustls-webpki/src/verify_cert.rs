// Copyright 2015 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use crate::{
    cert::{self, Cert, EndEntityOrCa},
    der, signed_data, subject_name, time, CertRevocationList, Error, SignatureAlgorithm,
    TrustAnchor,
};

pub(crate) struct ChainOptions<'a> {
    pub(crate) required_eku_if_present: KeyPurposeId,
    pub(crate) supported_sig_algs: &'a [&'a SignatureAlgorithm],
    pub(crate) trust_anchors: &'a [TrustAnchor<'a>],
    pub(crate) intermediate_certs: &'a [&'a [u8]],
    pub(crate) crls: &'a [&'a dyn CertRevocationList],
}

pub(crate) fn build_chain(opts: &ChainOptions, cert: &Cert, time: time::Time) -> Result<(), Error> {
    build_chain_inner(opts, cert, time, 0)
}

fn build_chain_inner(
    opts: &ChainOptions,
    cert: &Cert,
    time: time::Time,
    sub_ca_count: usize,
) -> Result<(), Error> {
    let used_as_ca = used_as_ca(&cert.ee_or_ca);

    check_issuer_independent_properties(
        cert,
        time,
        used_as_ca,
        sub_ca_count,
        opts.required_eku_if_present,
    )?;

    // TODO: HPKP checks.

    match used_as_ca {
        UsedAsCa::Yes => {
            const MAX_SUB_CA_COUNT: usize = 6;

            if sub_ca_count >= MAX_SUB_CA_COUNT {
                // TODO(XXX): Candidate for a more specific error - Error::PathTooDeep?
                return Err(Error::UnknownIssuer);
            }
        }
        UsedAsCa::No => {
            assert_eq!(0, sub_ca_count);
        }
    }

    // for the purpose of name constraints checking, only end-entity server certificates
    // could plausibly have a DNS name as a subject commonName that could contribute to
    // path validity
    let subject_common_name_contents =
        if opts.required_eku_if_present == EKU_SERVER_AUTH && used_as_ca == UsedAsCa::No {
            subject_name::SubjectCommonNameContents::DnsName
        } else {
            subject_name::SubjectCommonNameContents::Ignore
        };

    let result = loop_while_non_fatal_error(
        Error::UnknownIssuer,
        opts.trust_anchors,
        |trust_anchor: &TrustAnchor| {
            let trust_anchor_subject = untrusted::Input::from(trust_anchor.subject);
            if cert.issuer != trust_anchor_subject {
                return Err(Error::UnknownIssuer);
            }

            let name_constraints = trust_anchor.name_constraints.map(untrusted::Input::from);

            untrusted::read_all_optional(name_constraints, Error::BadDer, |value| {
                subject_name::check_name_constraints(value, cert, subject_common_name_contents)
            })?;

            // TODO: check_distrust(trust_anchor_subject, trust_anchor_spki)?;

            check_signatures(opts.supported_sig_algs, cert, trust_anchor, opts.crls)?;

            Ok(())
        },
    );

    let err = match result {
        Ok(()) => return Ok(()),
        Err(err) => err,
    };

    loop_while_non_fatal_error(err, opts.intermediate_certs, |cert_der| {
        let potential_issuer =
            cert::parse_cert(untrusted::Input::from(cert_der), EndEntityOrCa::Ca(cert))?;

        if potential_issuer.subject != cert.issuer {
            return Err(Error::UnknownIssuer);
        }

        // Prevent loops; see RFC 4158 section 5.2.
        let mut prev = cert;
        loop {
            if potential_issuer.spki.value() == prev.spki.value()
                && potential_issuer.subject == prev.subject
            {
                return Err(Error::UnknownIssuer);
            }
            match &prev.ee_or_ca {
                EndEntityOrCa::EndEntity => {
                    break;
                }
                EndEntityOrCa::Ca(child_cert) => {
                    prev = child_cert;
                }
            }
        }

        untrusted::read_all_optional(potential_issuer.name_constraints, Error::BadDer, |value| {
            subject_name::check_name_constraints(value, cert, subject_common_name_contents)
        })?;

        let next_sub_ca_count = match used_as_ca {
            UsedAsCa::No => sub_ca_count,
            UsedAsCa::Yes => sub_ca_count + 1,
        };

        build_chain_inner(opts, &potential_issuer, time, next_sub_ca_count)
    })
}

fn check_signatures(
    supported_sig_algs: &[&SignatureAlgorithm],
    cert_chain: &Cert,
    trust_anchor: &TrustAnchor,
    crls: &[&dyn CertRevocationList],
) -> Result<(), Error> {
    let mut spki_value = untrusted::Input::from(trust_anchor.spki);
    let mut issuer_subject = untrusted::Input::from(trust_anchor.subject);
    let mut issuer_key_usage = None; // TODO(XXX): Consider whether to track TrustAnchor KU.
    let mut cert = cert_chain;
    loop {
        signed_data::verify_signed_data(supported_sig_algs, spki_value, &cert.signed_data)?;

        if !crls.is_empty() {
            check_crls(
                supported_sig_algs,
                cert,
                issuer_subject,
                spki_value,
                issuer_key_usage,
                crls,
            )?;
        }

        match &cert.ee_or_ca {
            EndEntityOrCa::Ca(child_cert) => {
                spki_value = cert.spki.value();
                issuer_subject = cert.subject;
                issuer_key_usage = cert.key_usage;
                cert = child_cert;
            }
            EndEntityOrCa::EndEntity => {
                break;
            }
        }
    }

    Ok(())
}

// Zero-sized marker type representing positive assertion that revocation status was checked
// for a certificate and the result was that the certificate is not revoked.
struct CertNotRevoked(());

impl CertNotRevoked {
    // Construct a CertNotRevoked marker.
    fn assertion() -> Self {
        Self(())
    }
}

fn check_crls(
    supported_sig_algs: &[&SignatureAlgorithm],
    cert: &Cert,
    issuer_subject: untrusted::Input,
    issuer_spki: untrusted::Input,
    issuer_ku: Option<untrusted::Input>,
    crls: &[&dyn CertRevocationList],
) -> Result<Option<CertNotRevoked>, Error> {
    assert_eq!(cert.issuer, issuer_subject);

    let crl = match crls
        .iter()
        .find(|candidate_crl| candidate_crl.issuer() == cert.issuer())
    {
        Some(crl) => crl,
        None => return Ok(None),
    };

    // Verify the CRL signature with the issuer SPKI.
    // TODO(XXX): consider whether we can refactor so this happens once up-front, instead
    //            of per-lookup.
    //            https://github.com/rustls/webpki/issues/81
    crl.verify_signature(supported_sig_algs, issuer_spki.as_slice_less_safe())
        .map_err(crl_signature_err)?;

    // Verify that if the issuer has a KeyUsage bitstring it asserts cRLSign.
    check_key_usage(issuer_ku, KeyUsageMode::CrlSign)?;

    // Try to find the cert serial in the verified CRL contents.
    let cert_serial = cert.serial.as_slice_less_safe();
    match crl.find_serial(cert_serial)? {
        None => Ok(Some(CertNotRevoked::assertion())),
        Some(_) => Err(Error::CertRevoked),
    }
}

// When verifying CRL signed data we want to disambiguate the context of possible errors by mapping
// them to CRL specific variants that a consumer can use to tell the issue was with the CRL's
// signature, not a certificate.
fn crl_signature_err(err: Error) -> Error {
    match err {
        Error::UnsupportedSignatureAlgorithm => Error::UnsupportedCrlSignatureAlgorithm,
        Error::UnsupportedSignatureAlgorithmForPublicKey => {
            Error::UnsupportedCrlSignatureAlgorithmForPublicKey
        }
        Error::InvalidSignatureForPublicKey => Error::InvalidCrlSignatureForPublicKey,
        _ => err,
    }
}

fn check_issuer_independent_properties(
    cert: &Cert,
    time: time::Time,
    used_as_ca: UsedAsCa,
    sub_ca_count: usize,
    required_eku_if_present: KeyPurposeId,
) -> Result<(), Error> {
    // TODO: check_distrust(trust_anchor_subject, trust_anchor_spki)?;
    // TODO: Check signature algorithm like mozilla::pkix.
    // TODO: Check SPKI like mozilla::pkix.
    // TODO: check for active distrust like mozilla::pkix.

    // For cert validation, we ignore the KeyUsage extension. For CA
    // certificates, BasicConstraints.cA makes KeyUsage redundant. Firefox
    // and other common browsers do not check KeyUsage for end-entities,
    // though it would be kind of nice to ensure that a KeyUsage without
    // the keyEncipherment bit could not be used for RSA key exchange.

    cert.validity
        .read_all(Error::BadDer, |value| check_validity(value, time))?;
    untrusted::read_all_optional(cert.basic_constraints, Error::BadDer, |value| {
        check_basic_constraints(value, used_as_ca, sub_ca_count)
    })?;
    untrusted::read_all_optional(cert.eku, Error::BadDer, |value| {
        check_eku(value, required_eku_if_present)
    })?;

    Ok(())
}

// https://tools.ietf.org/html/rfc5280#section-4.1.2.5
fn check_validity(input: &mut untrusted::Reader, time: time::Time) -> Result<(), Error> {
    let not_before = der::time_choice(input)?;
    let not_after = der::time_choice(input)?;

    if not_before > not_after {
        return Err(Error::InvalidCertValidity);
    }
    if time < not_before {
        return Err(Error::CertNotValidYet);
    }
    if time > not_after {
        return Err(Error::CertExpired);
    }

    // TODO: mozilla::pkix allows the TrustDomain to check not_before and
    // not_after, to enforce things like a maximum validity period. We should
    // do something similar.

    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum UsedAsCa {
    Yes,
    No,
}

fn used_as_ca(ee_or_ca: &EndEntityOrCa) -> UsedAsCa {
    match ee_or_ca {
        EndEntityOrCa::EndEntity => UsedAsCa::No,
        EndEntityOrCa::Ca(..) => UsedAsCa::Yes,
    }
}

// https://tools.ietf.org/html/rfc5280#section-4.2.1.9
fn check_basic_constraints(
    input: Option<&mut untrusted::Reader>,
    used_as_ca: UsedAsCa,
    sub_ca_count: usize,
) -> Result<(), Error> {
    let (is_ca, path_len_constraint) = match input {
        Some(input) => {
            let is_ca = der::optional_boolean(input)?;

            // https://bugzilla.mozilla.org/show_bug.cgi?id=985025: RFC 5280
            // says that a certificate must not have pathLenConstraint unless
            // it is a CA certificate, but some real-world end-entity
            // certificates have pathLenConstraint.
            let path_len_constraint = if !input.at_end() {
                let value = der::small_nonnegative_integer(input)?;
                Some(usize::from(value))
            } else {
                None
            };

            (is_ca, path_len_constraint)
        }
        None => (false, None),
    };

    match (used_as_ca, is_ca, path_len_constraint) {
        (UsedAsCa::No, true, _) => Err(Error::CaUsedAsEndEntity),
        (UsedAsCa::Yes, false, _) => Err(Error::EndEntityUsedAsCa),
        (UsedAsCa::Yes, true, Some(len)) if sub_ca_count > len => {
            Err(Error::PathLenConstraintViolated)
        }
        _ => Ok(()),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeyPurposeId {
    oid_value: untrusted::Input<'static>,
}

// id-pkix            OBJECT IDENTIFIER ::= { 1 3 6 1 5 5 7 }
// id-kp              OBJECT IDENTIFIER ::= { id-pkix 3 }

// id-kp-serverAuth   OBJECT IDENTIFIER ::= { id-kp 1 }
#[allow(clippy::identity_op)] // TODO: Make this clearer
pub(crate) static EKU_SERVER_AUTH: KeyPurposeId = KeyPurposeId {
    oid_value: untrusted::Input::from(&[(40 * 1) + 3, 6, 1, 5, 5, 7, 3, 1]),
};

// id-kp-clientAuth   OBJECT IDENTIFIER ::= { id-kp 2 }
#[allow(clippy::identity_op)] // TODO: Make this clearer
pub(crate) static EKU_CLIENT_AUTH: KeyPurposeId = KeyPurposeId {
    oid_value: untrusted::Input::from(&[(40 * 1) + 3, 6, 1, 5, 5, 7, 3, 2]),
};

// id-kp-OCSPSigning  OBJECT IDENTIFIER ::= { id-kp 9 }
#[allow(clippy::identity_op)] // TODO: Make this clearer
pub(crate) static EKU_OCSP_SIGNING: KeyPurposeId = KeyPurposeId {
    oid_value: untrusted::Input::from(&[(40 * 1) + 3, 6, 1, 5, 5, 7, 3, 9]),
};

// https://tools.ietf.org/html/rfc5280#section-4.2.1.12
fn check_eku(
    input: Option<&mut untrusted::Reader>,
    required_eku_if_present: KeyPurposeId,
) -> Result<(), Error> {
    match input {
        Some(input) => {
            loop {
                let value = der::expect_tag_and_get_value(input, der::Tag::OID)?;
                if value == required_eku_if_present.oid_value {
                    input.skip_to_end();
                    break;
                }
                if input.at_end() {
                    return Err(Error::RequiredEkuNotFound);
                }
            }
            Ok(())
        }
        None => {
            // http://tools.ietf.org/html/rfc6960#section-4.2.2.2:
            // "OCSP signing delegation SHALL be designated by the inclusion of
            // id-kp-OCSPSigning in an extended key usage certificate extension
            // included in the OCSP response signer's certificate."
            //
            // A missing EKU extension generally means "any EKU", but it is
            // important that id-kp-OCSPSigning is explicit so that a normal
            // end-entity certificate isn't able to sign trusted OCSP responses
            // for itself or for other certificates issued by its issuing CA.
            if required_eku_if_present.oid_value == EKU_OCSP_SIGNING.oid_value {
                return Err(Error::RequiredEkuNotFound);
            }

            Ok(())
        }
    }
}

// https://www.rfc-editor.org/rfc/rfc5280#section-4.2.1.3
#[repr(u8)]
enum KeyUsageMode {
    // DigitalSignature = 0,
    // ContentCommitment = 1,
    // KeyEncipherment = 2,
    // DataEncipherment = 3,
    // KeyAgreement = 4,
    // CertSign = 5,
    CrlSign = 6,
    // EncipherOnly = 7,
    // DecipherOnly = 8,
}

// https://www.rfc-editor.org/rfc/rfc5280#section-4.2.1.3
fn check_key_usage(
    input: Option<untrusted::Input>,
    required_ku_bit_if_present: KeyUsageMode,
) -> Result<(), Error> {
    let bit_string = match input {
        Some(input) => input,
        // While RFC 5280 requires KeyUsage be present, historically the absence of a KeyUsage
        // has been treated as "Any Usage". We follow that convention here and assume the absence
        // of KeyUsage implies the required_ku_bit_if_present we're checking for.
        None => return Ok(()),
    };

    let flags = der::bit_string_flags(&mut untrusted::Reader::new(bit_string))?;
    #[allow(clippy::as_conversions)] // u8 always fits in usize.
    match flags.bit_set(required_ku_bit_if_present as usize) {
        true => Ok(()),
        false => Err(Error::IssuerNotCrlSigner),
    }
}

fn loop_while_non_fatal_error<V>(
    default_error: Error,
    values: V,
    f: impl Fn(V::Item) -> Result<(), Error>,
) -> Result<(), Error>
where
    V: IntoIterator,
{
    let mut error = default_error;
    for v in values {
        match f(v) {
            Ok(()) => return Ok(()),
            Err(new_error) => error = error.most_specific(new_error),
        }
    }
    Err(error)
}
