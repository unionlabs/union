# NOTE

This implementation is direct fork of Kylom's implementation. I claim no authorship over this code apart from some minor modifications.
Please be aware this code **has not yet been reviewed**.

ecies implements the Elliptic Curve Integrated Encryption Scheme.

The package is designed to be compliant with the appropriate NIST
standards, and therefore doesn't support the full SEC 1 algorithm set.


STATUS:

ecies should be ready for use. The ASN.1 support is only complete so
far as to supported the listed algorithms before.


CAVEATS

1. CMAC support is currently not present.


SUPPORTED ALGORITHMS

        SYMMETRIC CIPHERS               HASH FUNCTIONS
             AES128                         SHA-1
             AES192                        SHA-224
             AES256                        SHA-256
                                           SHA-384
        ELLIPTIC CURVE                     SHA-512
             P256
             P384		    KEY DERIVATION FUNCTION
             P521	       NIST SP 800-65a Concatenation KDF

Curve P224 isn't supported because it does not provide a minimum security
level of AES128 with HMAC-SHA1. According to NIST SP 800-57, the security
level of P224 is 112 bits of security. Symmetric ciphers use CTR-mode;
message tags are computed using HMAC-<HASH> function.


CURVE SELECTION

According to NIST SP 800-57, the following curves should be selected:

    +----------------+-------+
    | SYMMETRIC SIZE | CURVE |
    +----------------+-------+
    |     128-bit    |  P256 |
    +----------------+-------+
    |     192-bit    |  P384 |
    +----------------+-------+
    |     256-bit    |  P521 |
    +----------------+-------+


TODO

1. Look at serialising the parameters with the SEC 1 ASN.1 module.
2. Validate ASN.1 formats with SEC 1.


TEST VECTORS

The only test vectors I've found so far date from 1993, predating AES
and including only 163-bit curves. Therefore, there are no published
test vectors to compare to.


LICENSE

ecies is released under the same license as the Go source code. See the
LICENSE file for details.


REFERENCES

* SEC (Standard for Efficient Cryptography) 1, version 2.0: Elliptic
  Curve Cryptography; Certicom, May 2009.
  http://www.secg.org/sec1-v2.pdf
* GEC (Guidelines for Efficient Cryptography) 2, version 0.3: Test
  Vectors for SEC 1; Certicom, September 1999.
  http://read.pudn.com/downloads168/doc/772358/TestVectorsforSEC%201-gec2.pdf
* NIST SP 800-56a: Recommendation for Pair-Wise Key Establishment Schemes
  Using Discrete Logarithm Cryptography. National Institute of Standards
  and Technology, May 2007.
  http://csrc.nist.gov/publications/nistpubs/800-56A/SP800-56A_Revision1_Mar08-2007.pdf
* Suite B Implementer’s Guide to NIST SP 800-56A. National Security
  Agency, July 28, 2009.
  http://www.nsa.gov/ia/_files/SuiteB_Implementer_G-113808.pdf
* NIST SP 800-57: Recommendation for Key Management – Part 1: General
  (Revision 3). National Institute of Standards and Technology, July
  2012.
  http://csrc.nist.gov/publications/nistpubs/800-57/sp800-57_part1_rev3_general.pdf

