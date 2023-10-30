/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/WebCryptoAPI/
 */

typedef DOMString KeyType;
typedef DOMString KeyUsage;
typedef DOMString NamedCurve;
typedef Uint8Array BigInteger;

/***** Algorithm dictionaries *****/

dictionary Algorithm {
  required DOMString name;
};

dictionary AesCbcParams : Algorithm {
  required BufferSource iv;
};

dictionary AesCtrParams : Algorithm {
  required BufferSource counter;
  [EnforceRange] required octet length;
};

dictionary AesGcmParams : Algorithm {
  required BufferSource iv;
  BufferSource additionalData;
  [EnforceRange] octet tagLength;
};

dictionary HmacImportParams : Algorithm {
  required AlgorithmIdentifier hash;
};

dictionary Pbkdf2Params : Algorithm {
  required BufferSource salt;
  [EnforceRange] required unsigned long iterations;
  required AlgorithmIdentifier hash;
};

dictionary RsaHashedImportParams {
  required AlgorithmIdentifier hash;
};

dictionary AesKeyGenParams : Algorithm {
  [EnforceRange] required unsigned short length;
};

dictionary HmacKeyGenParams : Algorithm {
  required AlgorithmIdentifier hash;
  [EnforceRange] unsigned long length;
};

dictionary RsaHashedKeyGenParams : Algorithm {
  [EnforceRange] required unsigned long modulusLength;
  required BigInteger publicExponent;
  required AlgorithmIdentifier hash;
};

dictionary RsaOaepParams : Algorithm {
  BufferSource label;
};

dictionary RsaPssParams : Algorithm {
  [EnforceRange] required unsigned long saltLength;
};

dictionary DhKeyGenParams : Algorithm {
  required BigInteger prime;
  required BigInteger generator;
};

dictionary EcKeyGenParams : Algorithm {
  required NamedCurve namedCurve;
};

dictionary AesDerivedKeyParams : Algorithm {
  [EnforceRange] required unsigned long length;
};

dictionary HmacDerivedKeyParams : HmacImportParams {
  [EnforceRange] unsigned long length;
};

dictionary EcdhKeyDeriveParams : Algorithm {
  required CryptoKey public;
};

dictionary DhKeyDeriveParams : Algorithm {
  required CryptoKey public;
};

dictionary DhImportKeyParams : Algorithm {
  required BigInteger prime;
  required BigInteger generator;
};

dictionary EcdsaParams : Algorithm {
  required AlgorithmIdentifier hash;
};

dictionary EcKeyImportParams : Algorithm {
  NamedCurve namedCurve;
};

dictionary HkdfParams : Algorithm {
  required AlgorithmIdentifier hash;
  required BufferSource salt;
  required BufferSource info;
};

/***** JWK *****/

dictionary RsaOtherPrimesInfo {
  // The following fields are defined in Section 6.3.2.7 of JSON Web Algorithms
  required DOMString r;
  required DOMString d;
  required DOMString t;
};

dictionary JsonWebKey {
  // The following fields are defined in Section 3.1 of JSON Web Key
  required DOMString kty;
  DOMString use;
  sequence<DOMString> key_ops;
  DOMString alg;

  // The following fields are defined in JSON Web Key Parameters Registration
  boolean ext;

  // The following fields are defined in Section 6 of JSON Web Algorithms
  DOMString crv;
  DOMString x;
  DOMString y;
  DOMString d;
  DOMString n;
  DOMString e;
  DOMString p;
  DOMString q;
  DOMString dp;
  DOMString dq;
  DOMString qi;
  sequence<RsaOtherPrimesInfo> oth;
  DOMString k;
};


/***** The Main API *****/

interface CryptoKey {
  readonly attribute KeyType type;
  readonly attribute boolean extractable;
  [Cached, Constant, Throws] readonly attribute object algorithm;
  [Cached, Constant, Frozen] readonly attribute sequence<KeyUsage> usages;
};

dictionary CryptoKeyPair {
  required CryptoKey publicKey;
  required CryptoKey privateKey;
};

typedef DOMString KeyFormat;
typedef (object or DOMString) AlgorithmIdentifier;

[Exposed=(Window,Worker)]
interface SubtleCrypto {
  [Throws]
  Promise<any> encrypt(AlgorithmIdentifier algorithm,
                       CryptoKey key,
                       BufferSource data);
  [Throws]
  Promise<any> decrypt(AlgorithmIdentifier algorithm,
                       CryptoKey key,
                       BufferSource data);
  [Throws]
  Promise<any> sign(AlgorithmIdentifier algorithm,
                     CryptoKey key,
                     BufferSource data);
  [Throws]
  Promise<any> verify(AlgorithmIdentifier algorithm,
                      CryptoKey key,
                      BufferSource signature,
                      BufferSource data);
  [Throws]
  Promise<any> digest(AlgorithmIdentifier algorithm,
                      BufferSource data);

  [Throws]
  Promise<any> generateKey(AlgorithmIdentifier algorithm,
                           boolean extractable,
                           sequence<KeyUsage> keyUsages );
  [Throws]
  Promise<any> deriveKey(AlgorithmIdentifier algorithm,
                         CryptoKey baseKey,
                         AlgorithmIdentifier derivedKeyType,
                         boolean extractable,
                         sequence<KeyUsage> keyUsages );
  [Throws]
  Promise<any> deriveBits(AlgorithmIdentifier algorithm,
                          CryptoKey baseKey,
                          unsigned long length);

  [Throws]
  Promise<any> importKey(KeyFormat format,
                         object keyData,
                         AlgorithmIdentifier algorithm,
                         boolean extractable,
                         sequence<KeyUsage> keyUsages );
  [Throws]
  Promise<any> exportKey(KeyFormat format, CryptoKey key);

  [Throws]
  Promise<any> wrapKey(KeyFormat format,
                       CryptoKey key,
                       CryptoKey wrappingKey,
                       AlgorithmIdentifier wrapAlgorithm);

  [Throws]
  Promise<any> unwrapKey(KeyFormat format,
                         BufferSource wrappedKey,
                         CryptoKey unwrappingKey,
                         AlgorithmIdentifier unwrapAlgorithm,
                         AlgorithmIdentifier unwrappedKeyAlgorithm,
                         boolean extractable,
                         sequence<KeyUsage> keyUsages );
};

