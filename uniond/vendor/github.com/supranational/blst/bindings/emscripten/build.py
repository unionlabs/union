#!/usr/bin/env python3
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0

import os
import re
import sys
import shutil
import subprocess

emcc = shutil.which("emcc")
if emcc is None:
    git_root = os.path.join(os.path.dirname(sys.argv[0]), "..", "..")
    if os.name != 'posix' or not sys.stdin.isatty() or \
       not os.path.isdir(os.path.join(git_root, ".git")) or \
       not shutil.which("docker"):
        print("FATAL: no 'emcc' on the program search path", file=sys.stderr)
        sys.exit(2)

    try:
        answer = input("No 'emcc' found, do you want to try to build in docker [y/N]? ")
        if not answer.startswith("y"):
            sys.exit(2)
    except KeyboardInterrupt:
        sys.exit(2)

    cmd = "git clone -q /blst /tmp/blst && " + \
          "/tmp/blst/bindings/emscripten/build.py " + " ".join(sys.argv[1:])
    subprocess.check_call([
          "docker", "run",
          "--volume={}:/blst".format(os.path.abspath(git_root)),
          "--volume={}:/dst".format(os.getcwd()), "--workdir=/dst",
          "--user={}:{}".format(os.getuid(), os.getgid()), "--userns=host",
          "--network=none", "--rm",
          "emscripten/emsdk", "sh", "-c", cmd
    ])
    sys.exit(0)

common_cpp = """
#include <emscripten.h>
#include <blst.hpp>
using namespace blst;
#pragma GCC diagnostic ignored "-Wreturn-stack-address"
extern "C" {

EM_JS(void, blst_exception, (BLST_ERROR code), {
    throw new Error(BLST_ERROR_str[code]);
});
"""
common_js = """
const BLST_ERROR_str = [
    "BLST_ERROR: success",
    "BLST_ERROR: bad point encoding",
    "BLST_ERROR: point is not on curve",
    "BLST_ERROR: point is not in group",
    "BLST_ERROR: context type mismatch",
    "BLST_ERROR: verify failed",
    "BLST_ERROR: public key is infinite",
    "BLST_ERROR: bad scalar",
];
"""

common_js += """
function unsupported(type, extra)
{   if (typeof extra === 'undefined')
        return `${type ? type.constructor.name : 'none'}: unsupported type`;
    else
        return `${type ? type.constructor.name : 'none'}/${extra ? extra.constructor.name : 'none'}: unsupported types or combination thereof`;
}

function ensureAny(value)
{   if (value === null)
        return [0, 0];

    switch (value.constructor) {
        case String:
            return [ensureString(value), lengthBytesUTF8(value)];
        case ArrayBuffer:
            return [ensureInt8(new Uint8Array(value)), value.byteLength];
        case BigInt:
            if (value < 0)
                throw new Error("expecting unsigned BigInt value");
            var temp = [];
            while (value != 0) {
                temp.push(Number(value & 255n));
                value >>= 8n;
            }
            return [ensureInt8(temp), temp.length];
        case Uint8Array: case Buffer:
            return [ensureInt8(value), value.length];
        default:
            throw new Error(unsupported(value));
    }
}
"""

# ###### Constants
common_cpp += """
const P1_Affine* EMSCRIPTEN_KEEPALIVE const_G1()
{   return reinterpret_cast<const P1_Affine*>(&BLS12_381_G1);   }
const P2_Affine* EMSCRIPTEN_KEEPALIVE const_G2()
{   return reinterpret_cast<const P2_Affine*>(&BLS12_381_G2);   }
const P1_Affine* EMSCRIPTEN_KEEPALIVE const_NEG_G1()
{   return reinterpret_cast<const P1_Affine*>(&BLS12_381_NEG_G1);   }
const P2_Affine* EMSCRIPTEN_KEEPALIVE const_NEG_G2()
{   return reinterpret_cast<const P2_Affine*>(&BLS12_381_NEG_G2);   }
"""
common_js += """
(function() {
    function setupConsts() {
        var i = 0;
        Module['BLST_SUCCESS'] = i++;
        Module['BLST_BAD_ENCODING'] = i++;
        Module['BLST_POINT_NOT_ON_CURVE'] = i++;
        Module['BLST_POINT_NOT_IN_GROUP'] = i++;
        Module['BLST_AGGR_TYPE_MISMATCH'] = i++;
        Module['BLST_VERIFY_FAIL'] = i++;
        Module['BLST_PK_IS_INFINITY'] = i++;
        Module['BLST_BAD_SCALAR'] = i++;
        Module['BLS12_381_G1'] = wrapPointer(_const_G1(), P1_Affine);
        Module['BLS12_381_G2'] = wrapPointer(_const_G2(), P2_Affine);
        Module['BLS12_381_NEG_G1'] = wrapPointer(_const_NEG_G1(), P1_Affine);
        Module['BLS12_381_NEG_G2'] = wrapPointer(_const_NEG_G2(), P2_Affine);
    }
    if (runtimeInitialized) setupConsts();
    else addOnInit(setupConsts);
})();
"""

# ###### P1_Affine
p1_cpp = """
P1_Affine* EMSCRIPTEN_KEEPALIVE P1_Affine_0()
{   return new P1_Affine();   }
P1_Affine* EMSCRIPTEN_KEEPALIVE P1_Affine_1(const P1* input)
{   return new P1_Affine(*input);   }
P1_Affine* EMSCRIPTEN_KEEPALIVE P1_Affine_2(const byte* input, size_t len)
{   try                         { return new P1_Affine(input, len); }
    catch (const BLST_ERROR& e) { blst_exception(e);                }
    return nullptr;
}
void EMSCRIPTEN_KEEPALIVE P1_Affine__destroy__0(P1_Affine* self)
{   delete self;   }
"""
p1_js = """
/** @this{Object} */
function P1_Affine(input)
{   ensureCache.prepare();
    if (typeof input === 'undefined' || input === null)
        this.ptr = _P1_Affine_0();
    else if (input instanceof Uint8Array)
        this.ptr = _P1_Affine_2(ensureInt8(input), input.length);
    else if (input instanceof P1)
        this.ptr = _P1_Affine_1(input.ptr);
    else
        throw new Error(unsupported(input));
    getCache(P1_Affine)[this.ptr] = this;
}
P1_Affine.prototype = Object.create(WrapperObject.prototype);
P1_Affine.prototype.constructor = P1_Affine;
P1_Affine.prototype.__class__ = P1_Affine;
P1_Affine.__cache__ = {};
Module['P1_Affine'] = P1_Affine;
P1_Affine.prototype['__destroy__'] = P1_Affine.prototype.__destroy__ = /** @this{Object} */
function()
{   _P1_Affine__destroy__0(this.ptr); this.ptr = 0;   };;
"""
p1_cpp += """
P1_Affine* EMSCRIPTEN_KEEPALIVE P1_Affine_dup_0(const P1_Affine* self)
{   return new P1_Affine(self->dup());   }
"""
p1_js += """
P1_Affine.prototype['dup'] = P1_Affine.prototype.dup = /** @this{Object} */
function()
{   return wrapPointer(_P1_Affine_dup_0(this.ptr), P1_Affine);   };;
"""
p1_cpp += """
P1* EMSCRIPTEN_KEEPALIVE P1_Affine_to_jacobian_0(const P1_Affine* self)
{   return new P1(self->to_jacobian());   }
"""
p1_js += """
P1_Affine.prototype['to_jacobian'] = P1_Affine.prototype.to_jacobian = /** @this{Object} */
function()
{   return wrapPointer(_P1_Affine_to_jacobian_0(this.ptr), P1);   };;
"""
p1_cpp += """
byte* EMSCRIPTEN_KEEPALIVE P1_Affine_serialize_0(const P1_Affine* self)
{   byte out[96*1];
    self->serialize(out);
    return out;
}
"""
p1_js += """
P1_Affine.prototype['serialize'] = P1_Affine.prototype.serialize = /** @this{Object} */
function()
{   var out = _P1_Affine_serialize_0(this.ptr);
    return new Uint8Array(HEAPU8.subarray(out, out + 96*1));
};;
"""
p1_cpp += """
byte* EMSCRIPTEN_KEEPALIVE P1_Affine_compress_0(const P1_Affine* self)
{   byte out[48*1];
    self->compress(out);
    return out;
}
"""
p1_js += """
P1_Affine.prototype['compress'] = P1_Affine.prototype.compress = /** @this{Object} */
function()
{   var out = _P1_Affine_compress_0(this.ptr);
    return new Uint8Array(HEAPU8.subarray(out, out + 48*1));
};;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_Affine_on_curve_0(const P1_Affine* self)
{   return self->on_curve();   }
"""
p1_js += """
P1_Affine.prototype['on_curve'] = P1_Affine.prototype.on_curve = /** @this{Object} */
function()
{   return !!(_P1_Affine_on_curve_0(this.ptr));   };;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_Affine_in_group_0(const P1_Affine* self)
{   return self->in_group();   }
"""
p1_js += """
P1_Affine.prototype['in_group'] = P1_Affine.prototype.in_group = /** @this{Object} */
function()
{   return !!(_P1_Affine_in_group_0(this.ptr));   };;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_Affine_is_inf_0(const P1_Affine* self)
{   return self->is_inf();   }
"""
p1_js += """
P1_Affine.prototype['is_inf'] = P1_Affine.prototype.is_inf = /** @this{Object} */
function()
{   return !!(_P1_Affine_is_inf_0(this.ptr));   };;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_Affine_is_equal_1(const P1_Affine* self, const P1_Affine* p)
{   return self->is_equal(*p);   }
"""
p1_js += """
P1_Affine.prototype['is_equal'] = P1_Affine.prototype.is_equal = /** @this{Object} */
function(p)
{   if (p instanceof P1_Affine)
        return !!(_P1_Affine_is_equal_1(this.ptr, p.ptr));
    throw new Error(unsupported(p));
};;
"""
p1_cpp += """
int EMSCRIPTEN_KEEPALIVE P1_Affine_core_verify_7(const P1_Affine* self,
                                const P2_Affine* pk, bool hash_or_encode,
                                const byte* msg, size_t msg_len,
                                const char* DST,
                                const byte* aug, size_t aug_len)
{ return self->core_verify(*pk, hash_or_encode, msg, msg_len, DST ? DST : "", aug, aug_len); }
"""
p1_js += """
P1_Affine.prototype['core_verify'] = P1_Affine.prototype.core_verify = /** @this{Object} */
function(pk, hash_or_encode, msg, DST, aug)
{   if (!(pk instanceof P2_Affine))
        throw new Error(unsupported(pk));
    ensureCache.prepare();
    const [_msg, msg_len] = ensureAny(msg);
    DST = ensureString(DST);
    const [_aug, aug_len] = ensureAny(aug);
    return _P1_Affine_core_verify_7(this.ptr, pk.ptr, !!hash_or_encode, _msg, msg_len, DST, _aug, aug_len);
};;
"""
p1_cpp += """
P1_Affine* EMSCRIPTEN_KEEPALIVE P1_Affine_generator_0()
{   return new P1_Affine(P1_Affine::generator());   }
"""
p1_js += """
P1_Affine['generator'] = P1_Affine.generator =
function()
{   return wrapPointer(_P1_Affine_generator_0(), P1_Affine);   };;
"""

# ###### P1
p1_cpp += """
P1* EMSCRIPTEN_KEEPALIVE P1_0()
{   return new P1();   }
P1* EMSCRIPTEN_KEEPALIVE P1_affine_1(const P1_Affine* p)
{   return new P1(*p);   }
P1* EMSCRIPTEN_KEEPALIVE P1_secretkey_1(const SecretKey* sk)
{   return new P1(*sk);   }
P1* EMSCRIPTEN_KEEPALIVE P1_2(const byte* input, size_t len)
{   try                         { return new P1(input, len); }
    catch (const BLST_ERROR& e) { blst_exception(e);         }
    return nullptr;
}
void EMSCRIPTEN_KEEPALIVE P1__destroy__0(P1* self)
{   delete self;   }
"""
p1_js += """
/** @this{Object} */
function P1(input)
{   ensureCache.prepare();
    if (typeof input === 'undefined' || input === null)
        this.ptr = _P1_0();
    else if (input instanceof Uint8Array)
        this.ptr = _P1_2(ensureInt8(input), input.length);
    else if (input instanceof P1_Affine)
        this.ptr = _P1_affine_1(input.ptr);
    else if (input instanceof SecretKey)
        this.ptr = _P1_secretkey_1(input.ptr);
    else
        throw new Error(unsupported(input));
    getCache(P1)[this.ptr] = this;
}
P1.prototype = Object.create(WrapperObject.prototype);
P1.prototype.constructor = P1;
P1.prototype.__class__ = P1;
P1.__cache__ = {};
Module['P1'] = P1;
P1.prototype['__destroy__'] = P1.prototype.__destroy__ = /** @this{Object} */
function()
{   _P1__destroy__0(this.ptr); this.ptr = 0;  };;
"""
p1_cpp += """
P1* EMSCRIPTEN_KEEPALIVE P1_dup_0(const P1* self)
{   return new P1(self->dup());   }
"""
p1_js += """
P1.prototype['dup'] = P1.prototype.dup = /** @this{Object} */
function()
{   return wrapPointer(_P1_dup_0(this.ptr), P1);   };;
"""
p1_cpp += """
P1* EMSCRIPTEN_KEEPALIVE P1_to_affine_0(const P1* self)
{   return new P1(self->to_affine());   }
"""
p1_js += """
P1.prototype['to_affine'] = P1.prototype.to_affine = /** @this{Object} */
function()
{   return wrapPointer(_P1_to_affine_0(this.ptr), P1_Affine);   };;
"""
p1_cpp += """
byte* EMSCRIPTEN_KEEPALIVE P1_serialize_0(const P1* self)
{   byte out[96*1];
    self->serialize(out);
    return out;
}
"""
p1_js += """
P1.prototype['serialize'] = P1.prototype.serialize = /** @this{Object} */
function()
{   var out = _P1_serialize_0(this.ptr);
    return new Uint8Array(HEAPU8.subarray(out, out + 96*1));
};;
"""
p1_cpp += """
byte* EMSCRIPTEN_KEEPALIVE P1_compress_0(const P1* self)
{   byte out[48*1];
    self->compress(out);
    return out;
}
"""
p1_js += """
P1.prototype['compress'] = P1.prototype.compress = /** @this{Object} */
function()
{   var out = _P1_compress_0(this.ptr);
    return new Uint8Array(HEAPU8.subarray(out, out + 48*1));
};;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_on_curve_0(const P1* self)
{   return self->on_curve();   }
"""
p1_js += """
P1.prototype['on_curve'] = P1.prototype.on_curve = /** @this{Object} */
function()
{   return !!(_P1_on_curve_0(this.ptr));   };;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_in_group_0(const P1* self)
{   return self->in_group();   }
"""
p1_js += """
P1.prototype['in_group'] = P1.prototype.in_group = /** @this{Object} */
function()
{   return !!(_P1_in_group_0(this.ptr));   };;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_is_inf_0(const P1* self)
{   return self->is_inf();   }
"""
p1_js += """
P1.prototype['is_inf'] = P1.prototype.is_inf = /** @this{Object} */
function()
{   return !!(_P1_is_inf_0(this.ptr));   };;
"""
p1_cpp += """
bool EMSCRIPTEN_KEEPALIVE P1_is_equal_1(const P1* self, const P1* p)
{   return self->is_equal(*p);   }
"""
p1_js += """
P1.prototype['is_equal'] = P1.prototype.is_equal = /** @this{Object} */
function(p)
{   if (p instanceof P1)
        return !!(_P1_is_equal_1(this.ptr, p.ptr));
    throw new Error(unsupported(p));
};;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_aggregate_1(P1* self, const P1_Affine* p)
{   try                         { self->aggregate(*p); }
    catch (const BLST_ERROR& e) { blst_exception(e);   }
}
"""
p1_js += """
P1.prototype['aggregate'] = P1.prototype.aggregate = /** @this{Object} */
function(p)
{   if (p instanceof P1_Affine)
        _P1_aggregate_1(this.ptr, p.ptr);
    else
        throw new Error(unsupported(p));
};;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_sign_with_1(P1* self, const SecretKey* sk)
{   (void)self->sign_with(*sk);   }
"""
p1_js += """
P1.prototype['sign_with'] = P1.prototype.sign_with = /** @this{Object} */
function(sk)
{   if (sk instanceof SecretKey)
        _P1_sign_with_1(this.ptr, sk.ptr);
    else
        throw new Error(unsupported(sk));
    return this;
};;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_hash_to_5(P1* self, const byte* msg, size_t msg_len,
                                                 const char* DST,
                                                 const byte* aug, size_t aug_len)
{   (void)self->hash_to(msg, msg_len, DST ? DST : "", aug, aug_len);   }
"""
p1_js += """
P1.prototype['hash_to'] = P1.prototype.hash_to = /** @this{Object} */
function(msg, DST, aug)
{   ensureCache.prepare();
    const [_msg, msg_len] = ensureAny(msg);
    DST = ensureString(DST);
    const [_aug, aug_len] = ensureAny(aug);
    _P1_hash_to_5(this.ptr, _msg, msg_len, DST, _aug, aug_len);
    return this;
};;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_encode_to_5(P1* self, const byte* msg, size_t msg_len,
                                                   const char* DST,
                                                   const byte* aug, size_t aug_len)
{   (void)self->encode_to(msg, msg_len, DST ? DST : "", aug, aug_len);   }
"""
p1_js += """
P1.prototype['encode_to'] = P1.prototype.encode_to = /** @this{Object} */
function(msg, DST, aug)
{   ensureCache.prepare();
    const [_msg, msg_len] = ensureAny(msg);
    DST = ensureString(DST);
    const [_aug, aug_len] = ensureAny(aug);
    _P1_encode_to_5(this.ptr, _msg, msg_len, DST, _aug, aug_len);
    return this;
};;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_mult_1(P1* self, const Scalar* scalar)
{   (void)self->mult(*scalar);   }
void EMSCRIPTEN_KEEPALIVE P1_mult_2(P1* self, const byte* scalar, size_t nbits)
{   (void)self->mult(scalar, nbits);   }
"""
p1_js += """
P1.prototype['mult'] = P1.prototype.mult = /** @this{Object} */
function(scalar)
{   if (scalar instanceof Scalar) {
        _P1_mult_1(this.ptr, scalar.ptr);
    } else if (typeof scalar !== 'string') {
        ensureCache.prepare();
        const [_scalar, len] = ensureAny(scalar);
        _P1_mult_2(this.ptr, _scalar, len*8);
    } else {
        throw new Error(unsupported(scalar));
    }
    return this;
};;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_cneg_1(P1* self, bool flag)
{   (void)self->cneg(flag);   }
"""
p1_js += """
P1.prototype['cneg'] = P1.prototype.cneg = /** @this{Object} */
function(flag)
{   _P1_cneg_1(this.ptr, !!flag); return this;   };;
P1.prototype['neg'] = P1.prototype.neg = /** @this{Object} */
function()
{   _P1_cneg_1(this.ptr, true); return this;   };;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_add_1(P1* self, const P1* a)
{   (void)self->add(*a);   }
void EMSCRIPTEN_KEEPALIVE P1_add_affine_1(P1* self, const P1_Affine* a)
{   (void)self->add(*a);   }
"""
p1_js += """
P1.prototype['add'] = P1.prototype.add = /** @this{Object} */
function(p)
{   if (p instanceof P1)
        _P1_add_1(this.ptr, p.ptr);
    else if (p instanceof P1_Affine)
        _P1_add_affine_1(this.ptr, p.ptr);
    else
        throw new Error(unsupported(p));
    return this;
};;
"""
p1_cpp += """
void EMSCRIPTEN_KEEPALIVE P1_dbl_0(P1* self)
{   (void)self->dbl();   }
"""
p1_js += """
P1.prototype['dbl'] = P1.prototype.dbl = /** @this{Object} */
function()
{   _P1_dbl_0(this.ptr); return this;   };;
"""
p1_cpp += """
P1* EMSCRIPTEN_KEEPALIVE P1_generator_0()
{   return new P1(P1::generator());   }
"""
p1_js += """
Module['G1'] = P1['generator'] = P1.generator =
function()
{   return wrapPointer(_P1_generator_0(), P1);   };;
"""

# ###### SecretKey
common_cpp += """
SecretKey* EMSCRIPTEN_KEEPALIVE SecretKey_0()
{   return new SecretKey();   }
void EMSCRIPTEN_KEEPALIVE SecretKey__destroy__0(SecretKey* self)
{   explicit_bzero((void*)self, sizeof(*self)); delete self;   }
"""
common_js += """
/** @this{Object} */
function SecretKey()
{   this.ptr = _SecretKey_0();
    getCache(SecretKey)[this.ptr] = this;
}
SecretKey.prototype = Object.create(WrapperObject.prototype);
SecretKey.prototype.constructor = SecretKey;
SecretKey.prototype.__class__ = SecretKey;
SecretKey.__cache__ = {};
Module['SecretKey'] = SecretKey;
SecretKey.prototype['__destroy__'] = SecretKey.prototype.__destroy__ = /** @this{Object} */
function()
{   _SecretKey__destroy__0(this.ptr); this.ptr = 0;  };;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE SecretKey_keygen_3(SecretKey* self, const byte* IKM, size_t IKM_len, const char* info)
{   self->keygen(IKM, IKM_len, info ? info : "");   }
"""
common_js += """
SecretKey.prototype['keygen'] = SecretKey.prototype.keygen = /** @this{Object} */
function(IKM, info)
{   ensureCache.prepare();
    const [_IKM, IKM_len] = ensureAny(IKM);
    if (IKM_len < 32)
        throw new Error("BLST_ERROR: bad scalar");
    info = ensureString(info);
    _SecretKey_keygen_3(this.ptr, _IKM, IKM_len, info);
    HEAP8.fill(0, _IKM, _IKM + IKM_len);
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE SecretKey_derive_master_eip2333_2(SecretKey* self, const byte* IKM, size_t IKM_len)
{   self->derive_master_eip2333(IKM, IKM_len);   }
"""
common_js += """
SecretKey.prototype['derive_master_eip2333'] = SecretKey.prototype.derive_master_eip2333 = /** @this{Object} */
function(IKM)
{   ensureCache.prepare();
    const [_IKM, IKM_len] = ensureAny(IKM);
    if (IKM_len < 32)
        throw new Error("BLST_ERROR: bad scalar");
    _SecretKey_derive_master_eip2333_2(this.ptr, _IKM, IKM_len);
    HEAP8.fill(0, _IKM, _IKM + IKM_len);
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE SecretKey_derive_child_eip2333_2(SecretKey* self, const SecretKey* sk, unsigned int child_index)
{   self->derive_child_eip2333(*sk, child_index);   }
"""
common_js += """
SecretKey.prototype['derive_child_eip2333'] = SecretKey.prototype.derive_child_eip2333 = /** @this{Object} */
function(sk, child_index)
{   if (!(sk instanceof SecretKey))
        throw new Error(unsupported(sk));
    _SecretKey_derive_child_eip2333_2(this.ptr, sk.ptr, child_index);
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE SecretKey_from_bendian_1(SecretKey* self, const byte* sk)
{   self->from_bendian(sk);   }
"""
common_js += """
SecretKey.prototype['from_bendian'] = SecretKey.prototype.from_bendian = /** @this{Object} */
function(sk)
{   if (!(sk instanceof Uint8Array) || sk.length != 32)
        throw new Error(unsupported(sk));
    ensureCache.prepare();
    sk = ensureInt8(sk);
    _SecretKey_from_bendian_1(this.ptr, sk);
    HEAP8.fill(0, sk, sk + 32);
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE SecretKey_from_lendian_1(SecretKey* self, const byte* sk)
{   self->from_lendian(sk);   }
"""
common_js += """
SecretKey.prototype['from_lendian'] = SecretKey.prototype.from_lendian = /** @this{Object} */
function(sk)
{   if (!(sk instanceof Uint8Array) || sk.length != 32)
        throw new Error(unsupported(sk));
    ensureCache.prepare();
    sk = ensureInt8(sk);
    _SecretKey_from_lendian_1(this.ptr, sk);
    HEAP8.fill(0, sk, sk + 32);
};;
"""
common_cpp += """
byte* EMSCRIPTEN_KEEPALIVE SecretKey_to_bendian_0(const SecretKey* self)
{   byte out[32];
    self->to_bendian(out);
    return out;
}
"""
common_js += """
SecretKey.prototype['to_bendian'] = SecretKey.prototype.to_bendian = /** @this{Object} */
function()
{   var out = _SecretKey_to_bendian_0(this.ptr);
    var ret = new Uint8Array(HEAPU8.subarray(out, out + 32));
    HEAP8.fill(0, out, out + 32);
    return ret;
};;
"""
common_cpp += """
byte* EMSCRIPTEN_KEEPALIVE SecretKey_to_lendian_0(const SecretKey* self)
{   byte out[32];
    self->to_lendian(out);
    return out;
}
"""
common_js += """
SecretKey.prototype['to_lendian'] = SecretKey.prototype.to_lendian = /** @this{Object} */
function()
{   var out = _SecretKey_to_lendian_0(this.ptr);
    var ret = new Uint8Array(HEAPU8.subarray(out, out + 32));
    HEAP8.fill(0, out, out + 32);
    return ret;
};;
"""

# ###### Scalar
common_cpp += """
Scalar* EMSCRIPTEN_KEEPALIVE Scalar_0()
{   return new Scalar();   }
Scalar* EMSCRIPTEN_KEEPALIVE Scalar_2(const byte* scalar, size_t nbits)
{   return new Scalar(scalar, nbits);   }
Scalar* EMSCRIPTEN_KEEPALIVE Scalar_3(const byte* msg, size_t msg_len, const char* DST)
{   return new Scalar(msg, msg_len, DST ? DST : "");   }
void EMSCRIPTEN_KEEPALIVE Scalar__destroy__0(Scalar* self)
{   delete self;   }
"""
common_js += """
/** @this{Object} */
function Scalar(scalar, DST)
{   if (typeof scalar === 'undefined' || scalar === null) {
        this.ptr = _Scalar_0();
    } else {
        ensureCache.prepare();
        const [ _scalar, len] = ensureAny(scalar);
        if (typeof DST === 'string' || DST === null) {
            DST = ensureString(DST);
            this.ptr = _Scalar_3(_scalar, len, DST);
        } else {
            this.ptr = _Scalar_2(_scalar, len*8);
        }
    }
    getCache(Scalar)[this.ptr] = this;
}
Scalar.prototype = Object.create(WrapperObject.prototype);
Scalar.prototype.constructor = Scalar;
Scalar.prototype.__class__ = Scalar;
Scalar.__cache__ = {};
Module['Scalar'] = Scalar;
Scalar.prototype['__destroy__'] = Scalar.prototype.__destroy__ = /** @this{Object} */
function()
{   _Scalar__destroy__0(this.ptr); this.ptr = 0;  };;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Scalar_hash_to_3(Scalar* self, const byte* msg, size_t msg_len, const char* DST)
{   (void)self->hash_to(msg, msg_len, DST ? DST : "");   }
"""
common_js += """
Scalar.prototype['hash_to'] = Scalar.prototype.hash_to = /** @this{Object} */
function(msg, DST)
{   ensureCache.prepare();
    const [ _msg, msg_len] = ensureAny(msg);
    DST = ensureString(DST);
    _Scalar_hash_to_3(this.ptr, _msg, msg_len, DST);
    return this;
};;
"""
common_cpp += """
Scalar* EMSCRIPTEN_KEEPALIVE Scalar_dup_0(const Scalar* self)
{   return new Scalar(self->dup());   }
"""
common_js += """
Scalar.prototype['dup'] = Scalar.prototype.dup = /** @this{Object} */
function()
{   return wrapPointer(_Scalar_dup_0(this.ptr), Scalar);   };;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Scalar_from_bendian_2(Scalar* self, const byte* msg, size_t msg_len)
{   (void)self->from_bendian(msg, msg_len);   }
"""
common_js += """
Scalar.prototype['from_bendian'] = Scalar.prototype.from_bendian = /** @this{Object} */
function(msg)
{   ensureCache.prepare();
    const [ _msg, msg_len] = ensureAny(msg);
    _Scalar_from_bendian_2(this.ptr, _msg, msg_len);
    return this;
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Scalar_from_lendian_2(Scalar* self, const byte* msg, size_t msg_len)
{   (void)self->from_lendian(msg, msg_len);   }
"""
common_js += """
Scalar.prototype['from_lendian'] = Scalar.prototype.from_lendian = /** @this{Object} */
function(msg)
{   ensureCache.prepare();
    const [ _msg, msg_len] = ensureAny(msg);
    _Scalar_from_lendian_2(this.ptr, _msg, msg_len);
    return this;
};;
"""
common_cpp += """
byte* EMSCRIPTEN_KEEPALIVE Scalar_to_bendian_0(const Scalar* self)
{   byte out[32];
    self->to_bendian(out);
    return out;
}
"""
common_js += """
Scalar.prototype['to_bendian'] = Scalar.prototype.to_bendian = /** @this{Object} */
function()
{   var out = _Scalar_to_bendian_0(this.ptr);
    return new Uint8Array(HEAPU8.subarray(out, out + 32));
};;
"""
common_cpp += """
byte* EMSCRIPTEN_KEEPALIVE Scalar_to_lendian_0(const Scalar* self)
{   byte out[32];
    self->to_lendian(out);
    return out;
}
"""
common_js += """
Scalar.prototype['to_lendian'] = Scalar.prototype.to_lendian = /** @this{Object} */
function()
{   var out = _Scalar_to_lendian_0(this.ptr);
    return new Uint8Array(HEAPU8.subarray(out, out + 32));
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Scalar_add_1(Scalar* self, const Scalar* a)
{   try                         { (void)self->add(*a); }
    catch (const BLST_ERROR& e) { blst_exception(e);   }
}
"""
common_js += """
Scalar.prototype['add'] = Scalar.prototype.add = /** @this{Object} */
function(a)
{   if (!(a instanceof Scalar || a instanceof SecretKey))
        throw new Error(unsupported(a));
    _Scalar_add_1(this.ptr, a.ptr);
    return this;
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Scalar_sub_1(Scalar* self, const Scalar* a)
{   try                         { (void)self->sub(*a); }
    catch (const BLST_ERROR& e) { blst_exception(e);   }
}
"""
common_js += """
Scalar.prototype['sub'] = Scalar.prototype.sub = /** @this{Object} */
function(a)
{   if (!(a instanceof Scalar))
        throw new Error(unsupported(a));
    _Scalar_sub_1(this.ptr, a.ptr);
    return this;
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Scalar_mul_1(Scalar* self, const Scalar* a)
{   try                         { (void)self->mul(*a); }
    catch (const BLST_ERROR& e) { blst_exception(e);   }
}
"""
common_js += """
Scalar.prototype['mul'] = Scalar.prototype.mul = /** @this{Object} */
function(a)
{   if (!(a instanceof Scalar))
        throw new Error(unsupported(a));
    _Scalar_mul_1(this.ptr, a.ptr);
    return this;
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Scalar_inverse_0(Scalar* self)
{   (void)self->inverse();   }
"""
common_js += """
Scalar.prototype['inverse'] = Scalar.prototype.inverse = /** @this{Object} */
function()
{   _Scalar_inverse_0(this.ptr); return this;   };;
"""

# ###### PT
common_cpp += """
PT* EMSCRIPTEN_KEEPALIVE PT_p_affine_1(const P1_Affine* p)
{   return new PT(*p);   }
PT* EMSCRIPTEN_KEEPALIVE PT_q_affine_1(const P2_Affine* q)
{   return new PT(*q);   }
PT* EMSCRIPTEN_KEEPALIVE PT_pq_affine_2(const P1_Affine* p, const P2_Affine* q)
{   return new PT(*p, *q);   }
PT* EMSCRIPTEN_KEEPALIVE PT_pq_2(const P1* p, const P2* q)
{   return new PT(*p, *q);   }
void EMSCRIPTEN_KEEPALIVE PT__destroy__0(PT* self)
{   delete self;   }
"""
common_js += """
/** @this{Object} */
function PT(p, q)
{   if (typeof q === 'undefined' || q === null) {
        if (p instanceof P1_Affine)
            this.ptr = _PT_p_affine_1(p.ptr);
        else if (p instanceof P2_Affine)
            this.ptr = _PT_q_affine_1(p.ptr);
        else
            throw new Error(unsupported(p));
    } else if (p instanceof P1_Affine && q instanceof P2_Affine) {
        this.ptr = _PT_pq_affine_2(p.ptr, q.ptr);
    } else if (p instanceof P2_Affine && q instanceof P1_Affine) {
        this.ptr = _PT_pq_affine_2(q.ptr, p.ptr);
    } else if (p instanceof P1 && q instanceof P2) {
        this.ptr = _PT_pq_2(p.ptr, q.ptr);
    } else if (p instanceof P2 && q instanceof P1) {
        this.ptr = _PT_pq_2(q.ptr, p.ptr);
    } else {
        throw new Error(unsupported(p, q));
    }
    getCache(PT)[this.ptr] = this;
}
PT.prototype = Object.create(WrapperObject.prototype);
PT.prototype.constructor = PT;
PT.prototype.__class__ = PT;
PT.__cache__ = {};
Module['PT'] = PT;
PT.prototype['__destroy__'] = PT.prototype.__destroy__ = /** @this{Object} */
function()
{   _PT__destroy__0(this.ptr); this.ptr = 0;  };;
"""
common_cpp += """
PT* EMSCRIPTEN_KEEPALIVE PT_dup_0(const PT* self)
{   return new PT(self->dup());   }
"""
common_js += """
PT.prototype['dup'] = PT.prototype.dup = /** @this{Object} */
function()
{   return wrapPointer(_PT_dup_0(this.ptr), PT);   };;
"""
common_cpp += """
bool EMSCRIPTEN_KEEPALIVE PT_is_one_0(const PT* self)
{   return self->is_one();   }
"""
common_js += """
PT.prototype['is_one'] = PT.prototype.is_one = /** @this{Object} */
function()
{   return !!(_PT_is_one_0(this.ptr));   };;
"""
common_cpp += """
bool EMSCRIPTEN_KEEPALIVE PT_is_equal_1(const PT* self, const PT* p)
{   return self->is_equal(*p);   }
"""
common_js += """
PT.prototype['is_equal'] = PT.prototype.is_equal = /** @this{Object} */
function(p)
{   if (p instanceof PT)
        return !!(_PT_is_equal_1(this.ptr, p.ptr));
    throw new Error(unsupported(p));
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE PT_sqr_0(PT* self)
{   (void)self->sqr();   }
"""
common_js += """
PT.prototype['sqr'] = PT.prototype.sqr = /** @this{Object} */
function()
{   _PT_sqr_0(this.ptr); return this;   };;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE PT_mul_1(PT* self, const PT* p)
{   (void)self->mul(*p);   }
"""
common_js += """
PT.prototype['mul'] = PT.prototype.mul = /** @this{Object} */
function(p)
{   if (p instanceof PT)
        _PT_mul_1(this.ptr, p.ptr);
    else
        throw new Error(unsupported(p));
    return this;
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE PT_final_exp_0(PT* self)
{   (void)self->final_exp();   }
"""
common_js += """
PT.prototype['final_exp'] = PT.prototype.final_exp = /** @this{Object} */
function()
{   _PT_final_exp_0(this.ptr); return this;   };;
"""
common_cpp += """
bool EMSCRIPTEN_KEEPALIVE PT_in_group_0(const PT* self)
{   return self->in_group();   }
"""
common_js += """
PT.prototype['in_group'] = PT.prototype.in_group = /** @this{Object} */
function()
{   return !!(_PT_in_group_0(this.ptr));   };;
"""
common_cpp += """
byte* EMSCRIPTEN_KEEPALIVE PT_to_bendian_0(const PT* self)
{   byte out[48*12];
    self->to_bendian(out);
    return out;
}
"""
common_js += """
PT.prototype['to_bendian'] = PT.prototype.to_bendian = /** @this{Object} */
function()
{   var out = _PT_to_bendian_0(this.ptr);
    return new Uint8Array(HEAPU8.subarray(out, out + 48*12));
};;
"""
common_cpp += """
bool EMSCRIPTEN_KEEPALIVE PT_finalverify_2(const PT* gt1, const PT* gt2)
{   return PT::finalverify(*gt1, *gt2);   }
"""
common_js += """
PT['finalverify'] = PT.finalverify =
function(gt1, gt2)
{   if (gt1 instanceof PT && gt2 instanceof PT)
        return !!(_PT_finalverify_2(gt1.ptr, gt2.ptr));
    throw new Error(unsupported(gt1, gt2));
};;
"""
common_cpp += """
PT* EMSCRIPTEN_KEEPALIVE PT_one_0()
{   return new PT(PT::one());   }
"""
common_js += """
PT['one'] = PT.one =
function()
{   return wrapPointer(_PT_one_0(), PT);   };;
"""

# ###### Pairing
common_cpp += """
Pairing* EMSCRIPTEN_KEEPALIVE Pairing_2(bool hash_or_encode, const char* DST)
{   return new Pairing(hash_or_encode, std::string{DST ? DST : ""});   }
void EMSCRIPTEN_KEEPALIVE Pairing__destroy__0(Pairing* self)
{   delete self;   }
"""
common_js += """
/** @this{Object} */
function Pairing(hash_or_encode, DST)
{   ensureCache.prepare();
    DST = ensureString(DST);
    this.ptr = _Pairing_2(!!hash_or_encode, DST);
    getCache(SecretKey)[this.ptr] = this;
}
Pairing.prototype = Object.create(WrapperObject.prototype);
Pairing.prototype.constructor = Pairing;
Pairing.prototype.__class__ = Pairing;
Pairing.__cache__ = {};
Module['Pairing'] = Pairing;
Pairing.prototype['__destroy__'] = Pairing.prototype.__destroy__ = /** @this{Object} */
function()
{   _Pairing__destroy__0(this.ptr); this.ptr = 0;  };;
"""
p1_cpp += """
int EMSCRIPTEN_KEEPALIVE Pairing_aggregate_pk_in_g1_6(Pairing* self,
                                const P1_Affine* pk, const P2_Affine* sig,
                                const byte* msg, size_t msg_len,
                                const byte* aug, size_t aug_len)
{   return self->aggregate(pk, sig, msg, msg_len, aug, aug_len);   }
"""
common_js += """
Pairing.prototype['aggregate'] = Pairing.prototype.aggregate = /** @this{Object} */
function(pk, sig, msg, aug)
{   ensureCache.prepare();
    const [_msg, msg_len] = ensureAny(msg);
    const [_aug, aug_len] = ensureAny(aug);
    if (pk instanceof P1_Affine && sig instanceof P2_Affine)
        return _Pairing_aggregate_pk_in_g1_6(this.ptr, pk.ptr, sig.ptr, _msg, msg_len, _aug, aug_len);
    else if (pk instanceof P2_Affine && sig instanceof P1_Affine)
        return _Pairing_aggregate_pk_in_g2_6(this.ptr, pk.ptr, sig.ptr, _msg, msg_len, _aug, aug_len);
    else
        throw new Error(unsupported(pk, sig));
    return -1;
};;
"""
p1_cpp += """
int EMSCRIPTEN_KEEPALIVE Pairing_mul_n_aggregate_pk_in_g1_8(Pairing* self,
                                const P1_Affine* pk, const P2_Affine* sig,
                                const byte* scalar, size_t nbits,
                                const byte* msg, size_t msg_len,
                                const byte* aug, size_t aug_len)
{   return self->mul_n_aggregate(pk, sig, scalar, nbits, msg, msg_len, aug, aug_len);   }
"""
common_js += """
Pairing.prototype['mul_n_aggregate'] = Pairing.prototype.mul_n_aggregate = /** @this{Object} */
function(pk, sig, scalar, msg, aug)
{   if (typeof scalar === 'undefined' || scalar === null)
        throw new Error("missing |scalar| argument");
    ensureCache.prepare();
    const [_scalar, len] = ensureAny(scalar);
    const [_msg, msg_len] = ensureAny(msg);
    const [_aug, aug_len] = ensureAny(aug);
    if (pk instanceof P1_Affine && sig instanceof P2_Affine)
        return _Pairing_mul_n_aggregate_pk_in_g1_8(this.ptr, pk.ptr, sig.ptr, _scalar, len*8, _msg, msg_len, _aug, aug_len);
    else if (pk instanceof P2_Affine && sig instanceof P1_Affine)
        return _Pairing_mul_n_aggregate_pk_in_g2_8(this.ptr, pk.ptr, sig.ptr, _scalar, len*8, _msg, msg_len, _aug, aug_len);
    else
        throw new Error(unsupported(pk, sig));
    return -1;
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Pairing_commit_0(Pairing* self)
{   self->commit();   }
"""
common_js += """
Pairing.prototype['commit'] = Pairing.prototype.commit = /** @this{Object} */
function()
{   _Pairing_commit_0(this.ptr);   };;
"""
common_cpp += """
size_t EMSCRIPTEN_KEEPALIVE Pairing_sizeof_0()
{   return blst_pairing_sizeof();   }
"""
common_js += """
Pairing.prototype['asArrayBuffer'] = Pairing.prototype.asArrayBuffer = /** @this{Object} */
function()
{   return HEAP8.buffer.slice(this.ptr, this.ptr + _Pairing_sizeof_0());   };;
"""
common_cpp += """
int EMSCRIPTEN_KEEPALIVE Pairing_merge_1(Pairing* self, const Pairing* ctx)
{   return self->merge(ctx);   }
"""
common_js += """
Pairing.prototype['merge'] = Pairing.prototype.merge = /** @this{Object} */
function(ctx)
{   if (ctx instanceof Pairing)
        return _Pairing_merge_1(this.ptr, ctx.ptr);
    else if (ctx instanceof ArrayBuffer && ctx.byteLength == _Pairing_sizeof_0())
        return _Pairing_merge_1(this.ptr, ensureAny(ctx)[0]);
    throw new Error(unsupported(ctx));
};;
"""
common_cpp += """
bool EMSCRIPTEN_KEEPALIVE Pairing_finalverify_1(const Pairing* self, const PT* sig)
{   return self->finalverify(sig);   }
"""
common_js += """
Pairing.prototype['finalverify'] = Pairing.prototype.finalverify = /** @this{Object} */
function(sig)
{   if (typeof sig === 'undefined' || sig === null)
        return !!(_Pairing_finalverify_1(this.ptr, 0));
    else if (sig instanceof PT)
        return !!(_Pairing_finalverify_1(this.ptr, sig.ptr));
    else
        throw new Error(unsupported(sig));
};;
"""
common_cpp += """
void EMSCRIPTEN_KEEPALIVE Pairing_raw_aggregate_2(Pairing* self, const P2_Affine* q, const P1_Affine* p)
{   self->raw_aggregate(q, p);   }
"""
common_js += """
Pairing.prototype['raw_aggregate'] = Pairing.prototype.raw_aggregate = /** @this{Object} */
function(q, p)
{   if (q instanceof P2_Affine && p instanceof P1_Affine)
        _Pairing_raw_aggregate_2(this.ptr, q.ptr, p.ptr);
    else
        throw new Error(unsupported(q, p));
};;
"""
common_cpp += """
PT* EMSCRIPTEN_KEEPALIVE Pairing_as_fp12_0(Pairing* self)
{   return new PT(self->as_fp12());   }
"""
common_js += """
Pairing.prototype['as_fp12'] = Pairing.prototype.as_fp12 = /** @this{Object} */
function()
{   return wrapPointer(_Pairing_as_fp12_0(this.ptr), PT);   };;
"""


blst_js = os.path.join(os.getcwd(), "blst.js")  # output file
there = re.split(r'[/\\](?=[^/\\]*$)', sys.argv[0])
if len(there) > 1:
    os.chdir(there[0])


def xchg_1vs2(matchobj):
    if matchobj.group(2) == '1':
        return matchobj.group(1) + '2'
    else:
        return matchobj.group(1) + '1'


fd = open("blst_bind.cpp", "w")
print("//!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", file=fd)
print("// DO NOT EDIT THIS FILE!!!",                         file=fd)
print("// The file is auto-generated by " + there[-1],       file=fd)
print("//!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", file=fd)
print(common_cpp, file=fd)
print(p1_cpp, file=fd)
print(re.sub(r'((?<!f)[pgPG\*])([12])', xchg_1vs2, p1_cpp), file=fd)
print("}", file=fd)  # close extern "C" {
fd.close()

fd = open("blst_bind.js", "w")
print("//!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", file=fd)
print("// DO NOT EDIT THIS FILE!!!",                         file=fd)
print("// The file is auto-generated by " + there[-1],       file=fd)
print("//!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", file=fd)
print(common_js, file=fd)
print(p1_js, file=fd)
print(re.sub(r'((?<!f)[pgPG\*])([12])', xchg_1vs2, p1_js), file=fd)
fd.close()

subprocess.check_call([os.path.join(os.path.dirname(emcc), "tools", "webidl_binder"),
                       os.devnull, "null_bind"])
subprocess.check_call([emcc, "-I..", "-fexceptions", "-include", "stddef.h",
                       "-sDEFAULT_LIBRARY_FUNCS_TO_INCLUDE=[$intArrayFromString]",
                       "-sEXPORTED_FUNCTIONS=[_malloc,_free]",
                       "null_bind.cpp", "--post-js", "null_bind.js",
                       "blst_bind.cpp", "--post-js", "blst_bind.js",
                       os.path.normpath("../../src/server.c"),
                       "-o", blst_js] +
                      sys.argv[1:])  # pass through flags, e.g. -O2 --closure 1

for opt in sys.argv[1:]:
    if opt.find("EXPORT_NAME=") >= 0:
        sys.exit(0)

# Rename 'Module' to 'blst' for smoother use in browser. This is done by
# finding the very 1st variable declaration, taking its name and replacing
# its occurrences with 'blst' through the whole script. [Yes, there is
# -sEXPORT_NAME=name, but it doesn't seem to work quite the way I expected.]

with open(blst_js, "r") as fd:
    contents = fd.readlines()

replace = None
for i, line in enumerate(contents):
    if replace is None:
        match = re.search(r'\bvar\s+([A-Z_a-z][0-9A-Z_a-z]*)\b', line)
        if match:
            replace = re.compile(r'\b(?<!\$){}\b'.format(match.group(1)))
            contents[i] = replace.sub('blst', line)
    else:
        contents[i] = replace.sub('blst', line)

with open(blst_js, "w") as fd:
    fd.writelines(contents)
