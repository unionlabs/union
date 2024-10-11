// Copyright (c) 2014 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

/*
Package base58 provides an API for working with modified base58 and Base58Check
encodings.

Modified Base58 Encoding

Standard base58 encoding is similar to standard base64 encoding except, as the
name implies, it uses a 58 character alphabet which results in an alphanumeric
string and allows some characters which are problematic for humans to be
excluded.  Due to this, there can be various base58 alphabets.

The modified base58 alphabet used by Bitcoin, and hence this package, omits the
0, O, I, and l characters that look the same in many fonts and are therefore
hard to humans to distinguish.

Base58Check Encoding Scheme

The Base58Check encoding scheme is primarily used for Bitcoin addresses at the
time of this writing, however it can be used to generically encode arbitrary
byte arrays into human-readable strings along with a version byte that can be
used to differentiate the same payload.  For Bitcoin addresses, the extra
version is used to differentiate the network of otherwise identical public keys
which helps prevent using an address intended for one network on another.
*/
package base58
