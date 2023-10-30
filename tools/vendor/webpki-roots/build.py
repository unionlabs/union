# -*- coding: utf-8 -*-
import base64
import binascii
import codecs
import subprocess
import sys
import urllib.request
import urllib.parse
import urllib.error

import extra_constraints

HEADER = """//!
//! This library is automatically generated from the Mozilla certificate
//! store via mkcert.org.  Don't edit it.
//!
//! The generation is done deterministically so you can verify it
//! yourself by inspecting and re-running the generation process.
//!

#![forbid(unsafe_code,
          unstable_features)]
#![deny(trivial_casts,
        trivial_numeric_casts,
        unused_import_braces,
        unused_extern_crates,
        unused_qualifications)]
"""

CERT = """
  %(comment)s
  %(code)s,"""

excluded_cas = [
    # See https://bugzilla.mozilla.org/show_bug.cgi?id=1266574.
    "Buypass Class 2 CA 1",

    # https://blog.mozilla.org/security/2015/04/02/distrusting-new-cnnic-certificates/
    # https://security.googleblog.com/2015/03/maintaining-digital-certificate-security.html
    "China Internet Network Information Center",
    "CNNIC",

    # See https://bugzilla.mozilla.org/show_bug.cgi?id=1283326.
    "RSA Security 2048 v3",

    # https://bugzilla.mozilla.org/show_bug.cgi?id=1272158
    "Root CA Generalitat Valenciana",

    # See https://wiki.mozilla.org/CA:WoSign_Issues.
    "StartCom",
    "WoSign",

    # See https://cabforum.org/pipermail/public/2016-September/008475.html.
    # Both the ASCII and non-ASCII names are required.
    "TÜRKTRUST",
    "TURKTRUST",
]


def fetch_bundle():
    proc = subprocess.Popen(['curl',
                             'https://mkcert.org/generate/all/except/' +
                                "+".join([urllib.parse.quote(x) for x in excluded_cas])],
            stdout = subprocess.PIPE)
    stdout, _ = proc.communicate()
    return stdout.decode('utf-8')


def split_bundle(bundle):
    cert = ''
    for line in bundle.splitlines():
        if line.strip() != '':
            cert += line + '\n'
        if '-----END CERTIFICATE-----' in line:
            yield cert
            cert = ''


def calc_spki_hash(cert):
    """
    Use openssl to sha256 hash the public key in the certificate.
    """
    proc = subprocess.Popen(
            ['openssl', 'x509', '-noout', '-sha256', '-fingerprint'],
            stdin = subprocess.PIPE,
            stdout = subprocess.PIPE)
    stdout, _ = proc.communicate(cert.encode('utf-8'))
    stdout = stdout.decode('utf-8').lower()
    assert proc.returncode == 0
    assert stdout.startswith('sha256 fingerprint=')
    hash = stdout.replace('sha256 fingerprint=', '').replace(':', '')
    hash = hash.strip()
    assert len(hash) == 64
    return hash.lower()


def extract_header_spki_hash(cert):
    """
    Extract the sha256 hash of the public key in the header, for
    cross-checking.
    """
    line = [ll for ll in cert.splitlines() if ll.startswith('# SHA256 Fingerprint: ')][0]
    return line.replace('# SHA256 Fingerprint: ', '').replace(':', '').lower()


def unwrap_pem(cert):
    start = '-----BEGIN CERTIFICATE-----\n'
    end = '-----END CERTIFICATE-----\n'
    body = cert[cert.index(start)+len(start):cert.rindex(end)]
    return base64.b64decode(body)


def extract(msg, name):
    lines = msg.splitlines()
    value = [ll for ll in lines if ll.startswith(name + ': ')][0]
    return value[len(name) + 2:].strip()


def convert_cert(cert_der):
    proc = subprocess.Popen(
            ['target/debug/process_cert'],
            stdin = subprocess.PIPE,
            stdout = subprocess.PIPE)
    stdout, _ = proc.communicate(cert_der)
    stdout = stdout.decode('utf-8')
    assert proc.returncode == 0
    return dict(
            subject = extract(stdout, 'Subject'),
            spki = extract(stdout, 'SPKI'),
            name_constraints = extract(stdout, 'Name-Constraints'))


def commentify(cert):
    lines = cert.splitlines()
    lines = [ll[2:] if ll.startswith('# ') else ll for ll in lines]
    return '/*\n   * ' + ('\n   * '.join(lines)) + '\n   */'


def convert_bytes(hex):
    bb = binascii.a2b_hex(hex)
    encoded, _ = codecs.escape_encode(bb)
    return encoded.decode('utf-8').replace('"', '\\"')


def print_root(cert, data):
    subject = convert_bytes(data['subject'])
    spki = convert_bytes(data['spki'])
    nc = data['name_constraints']
    nc = ('Some(b"{}")'.format(convert_bytes(nc))) if nc != 'None' else nc

    print("""  {}
  webpki::TrustAnchor {{
    subject: b"{}",
    spki: b"{}",
    name_constraints: {}
  }},
""".format(commentify(cert), subject, spki, nc))


if __name__ == '__main__':
    if sys.platform == "win32":
        import os, msvcrt
        msvcrt.setmode(sys.stdout.fileno(), os.O_BINARY)

    bundle = fetch_bundle()
    open('fetched.pem', 'w').write(bundle)

    certs = {}

    for cert in split_bundle(bundle):
        our_hash = calc_spki_hash(cert)
        their_hash = extract_header_spki_hash(cert)
        assert our_hash == their_hash

        cert_der = unwrap_pem(cert)
        data = convert_cert(cert_der)

        imposed_nc = extra_constraints.get_imposed_name_constraints(data['subject'])
        if imposed_nc:
            data['name_constraints'] = binascii.b2a_hex(imposed_nc)

        assert our_hash not in certs, 'duplicate cert'
        certs[our_hash] = (cert, data)

    print(HEADER)
    print("""pub static TLS_SERVER_ROOTS: webpki::TlsServerTrustAnchors = webpki::TlsServerTrustAnchors(&[""")

    # emit in sorted hash order for deterministic builds
    for hash in sorted(certs):
        cert, data = certs[hash]
        print_root(cert, data)

    print(']);')
