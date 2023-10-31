/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/WebCryptoAPI/
 */

dictionary KeyAlgorithm {
  required DOMString name;
};

dictionary AesKeyAlgorithm : KeyAlgorithm {
  required unsigned short length;
};

dictionary EcKeyAlgorithm : KeyAlgorithm {
  required DOMString namedCurve;
};

dictionary HmacKeyAlgorithm : KeyAlgorithm {
  required KeyAlgorithm hash;
  required unsigned long length;
};

dictionary RsaHashedKeyAlgorithm : KeyAlgorithm {
  required unsigned short modulusLength;
  required Uint8Array publicExponent;
  required KeyAlgorithm hash;
};

dictionary DhKeyAlgorithm : KeyAlgorithm {
  required Uint8Array prime;
  required Uint8Array generator;
};

