export interface Blst {
  SecretKey: SecretKeyConstructor;
  Scalar: ScalarConstructor;
  P1_Affine: P1_AffineConstructor;
  P2_Affine: P2_AffineConstructor;
  P1: P1Constructor;
  P2: P2Constructor;
  PT: PTConstructor;
  Pairing: PairingConstructor;
  G1(): P1;
  G2(): P2;
}

// blst.hpp types

type bytes = Uint8Array;
type scalar = Scalar | bigint | bytes;
type binary_string = string | bytes;

// SecretKey

export interface SecretKeyConstructor {
  new (): SecretKey;
}

export interface SecretKey {
  keygen(IKM: binary_string, info?: string): void;
  derive_master_eip2333(IKM: binary_string): void;
  derive_child_eip2333(SK: SecretKey, child_index: number): void;
  from_bendian(_32: bytes): void;
  from_lendian(_32: bytes): void;
  to_bendian(): bytes;
  to_lendian(): bytes;
}

// Scalar

export interface ScalarConstructor {
  new (): Scalar;
  new (le: bytes): Scalar;
  new (msg: binary_string, DST?: string): Scalar;
}

export interface Scalar {
  hash_to (msg: binary_string, DST?: string): this;
  dup(): Scalar;
  from_bendian(be: bytes): this;
  from_lendian(le: bytes): this;
  to_bendian(): bytes;
  to_lendian(): bytes;
  add(s: this | SecretKey): this;
  sub(s: this): this;
  mul(s: this): this;
  inverse(): this;
}

// P1

export interface P1_AffineConstructor {
  new (): P1_Affine;
  new (_48_or_96: bytes): P1_Affine;
  new (jacobian: P1): P1_Affine;
}

export interface P1_Affine {
  dup(): P1_Affine;
  to_jacobian(): P1;
  serialize(): bytes;
  compress(): bytes;
  on_curve(): boolean;
  in_group(): boolean;
  is_inf(): boolean;
  is_equal(p: P1_Affine): boolean;
  core_verify(
    pk: P2_Affine,
    hash_or_encode: boolean,
    msg: binary_string,
    DST?: string,
    aug?: bytes
  ): BLST_ERROR;
}

export interface P1Constructor {
  new (): P1;
  new (sk: SecretKey): P1;
  new (_48_or_96: bytes): P1;
  new (affine: P1_Affine): P1;
  generator(): P1;
}

export interface P1 {
  dup(): P1;
  to_affine(): P1_Affine;
  serialize(): bytes;
  compress(): bytes;
  on_curve(): boolean;
  in_group(): boolean;
  is_inf(): boolean;
  is_equal(p: P1): boolean;
  aggregate(affine: P1_Affine): void;
  sign_with(sk: SecretKey): this;
  hash_to(msg: binary_string, DST?: string, aug?: bytes): this;
  encode_to(msg: binary_string, DST?: string, aug?: bytes): this;
  mult(scalar: scalar): this;
  cneg(flag: boolean): this;
  neg(): this;
  add(a: this): this;
  add(a: P1_Affine): this;
  dbl(): this;
}

// P2

export interface P2_AffineConstructor {
  new (): P2_Affine;
  new (_96_or_192: bytes): P2_Affine;
  new (jacobian: P2): P2_Affine;
}

export interface P2_Affine {
  dup(): P2_Affine;
  to_jacobian(): P2;
  serialize(): bytes;
  compress(): bytes;
  on_curve(): boolean;
  in_group(): boolean;
  is_inf(): boolean;
  is_equal(p: P2_Affine): boolean;
  core_verify(
    pk: P1_Affine,
    hash_or_encode: boolean,
    msg: binary_string,
    DST?: string,
    aug?: bytes
  ): BLST_ERROR;
}

export interface P2Constructor {
  new (): P2;
  new (sk: SecretKey): P2;
  new (_96_192: bytes): P2;
  new (affine: P2_Affine): P2;
  generator(): P2;
}

export interface P2 {
  dup(): P2;
  to_affine(): P2_Affine;
  serialize(): bytes;
  compress(): bytes;
  on_curve(): boolean;
  in_group(): boolean;
  is_inf(): boolean;
  is_equal(p: P2): boolean;
  aggregate(affine: P2_Affine): void;
  sign_with(sk: SecretKey): this;
  hash_to(msg: binary_string, DST?: string, aug?: bytes): this;
  encode_to(msg: binary_string, DST?: string, aug?: bytes): this;
  mult(scalar: scalar): this;
  cneg(flag: boolean): this;
  neg(): this;
  add(a: this): this;
  add(a: P2_Affine): this;
  dbl(): this;
}

// PT

export interface PTConstructor {
  new (p: P1_Affine): PT;
  new (q: P2_Affine): PT;
  new (q: P2_Affine | P2, p: P1_Affine | P1): PT;
  new (p: P1_Affine | P1, q: P2_Affine | P2): PT;
  one(): PT;
}

export interface PT {
  dup(): PT;
  is_one(): boolean;
  is_equal(p: PT): boolean;
  sqr(): this;
  mul(p: PT): this;
  final_exp(): this;
  in_group(): boolean;
  to_bendian(): bytes;
}

// Pairing

export interface PairingConstructor {
  new (hash_or_encode: boolean, DST: string): Pairing;
}

export interface Pairing {
  aggregate(
    pk: P1_Affine,
    sig: P2_Affine,
    msg: binary_string,
    aug?: bytes
  ): BLST_ERROR;
  aggregate(
    pk: P2_Affine,
    sig: P1_Affine,
    msg: binary_string,
    aug?: bytes
  ): BLST_ERROR;
  mul_n_aggregate(
    pk: P1_Affine,
    sig: P2_Affine,
    scalar: scalar,
    msg: binary_string,
    aug?: bytes
  ): BLST_ERROR;
  mul_n_aggregate(
    pk: P2_Affine,
    sig: P1_Affine,
    scalar: scalar,
    msg: binary_string,
    aug?: bytes
  ): BLST_ERROR;
  commit(): void;
  merge(ctx: Pairing): BLST_ERROR;
  finalverify(sig?: PT): boolean;
  raw_aggregate(q: P2_Affine, p: P1_Affine): void;
  as_fp12(): PT;
}

// Misc

export enum BLST_ERROR {
  BLST_SUCCESS = 0,
  BLST_BAD_ENCODING = 1,
  BLST_POINT_NOT_ON_CURVE = 2,
  BLST_POINT_NOT_IN_GROUP = 3,
  BLST_AGGR_TYPE_MISMATCH = 4,
  BLST_VERIFY_FAIL = 5,
  BLST_PK_IS_INFINITY = 6,
  BLST_BAD_SCALAR = 7,
}
