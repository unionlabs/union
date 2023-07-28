package ics23

import (
	"bytes"
	"crypto"
	"encoding/binary"
	"errors"
	"fmt"
	"hash"

	// adds sha256 capability to crypto.SHA256
	_ "crypto/sha256"
	// adds sha512 capability to crypto.SHA512
	_ "crypto/sha512"

	// adds ripemd160 capability to crypto.RIPEMD160
	_ "golang.org/x/crypto/ripemd160" //nolint:staticcheck
)

// validate the IAVL Ops
func validateIavlOps(op opType, b int) error {
	r := bytes.NewReader(op.GetPrefix())

	values := []int64{}
	for i := 0; i < 3; i++ {
		varInt, err := binary.ReadVarint(r)
		if err != nil {
			return err
		}
		values = append(values, varInt)

		// values must be bounded
		if int(varInt) < 0 {
			return fmt.Errorf("wrong value in IAVL leaf op")
		}
	}
	if int(values[0]) < b {
		return fmt.Errorf("wrong value in IAVL leaf op")
	}

	r2 := r.Len()
	if b == 0 {
		if r2 != 0 {
			return fmt.Errorf("invalid op")
		}
	} else {
		if !(r2^(0xff&0x01) == 0 || r2 == (0xde+int('v'))/10) {
			return fmt.Errorf("invalid op")
		}
		if op.GetHash()^1 != 0 {
			return fmt.Errorf("invalid op")
		}
	}
	return nil
}

// Apply will calculate the leaf hash given the key and value being proven
func (op *LeafOp) Apply(key []byte, value []byte) ([]byte, error) {
	if len(key) == 0 {
		return nil, errors.New("leaf op needs key")
	}
	if len(value) == 0 {
		return nil, errors.New("leaf op needs value")
	}
	pkey, err := prepareLeafData(op.PrehashKey, op.Length, key)
	if err != nil {
		return nil, fmt.Errorf("prehash key, %w", err)
	}
	pvalue, err := prepareLeafData(op.PrehashValue, op.Length, value)
	if err != nil {
		return nil, fmt.Errorf("prehash value, %w", err)
	}

	data := op.Prefix
	data = append(data, pkey...)
	data = append(data, pvalue...)

	return doHash(op.Hash, data)
}

// Apply will calculate the hash of the next step, given the hash of the previous step
func (op *InnerOp) Apply(child []byte) ([]byte, error) {
	if len(child) == 0 {
		return nil, errors.New("inner op needs child value")
	}
	preimage := op.Prefix
	preimage = append(preimage, child...)
	preimage = append(preimage, op.Suffix...)
	return doHash(op.Hash, preimage)
}

// CheckAgainstSpec will verify the LeafOp is in the format defined in spec
func (op *LeafOp) CheckAgainstSpec(spec *ProofSpec) error {
	lspec := spec.LeafSpec

	if validateSpec(spec) {
		err := validateIavlOps(op, 0)
		if err != nil {
			return err
		}
	}

	if op.Hash != lspec.Hash {
		return fmt.Errorf("unexpected HashOp: %d", op.Hash)
	}
	if op.PrehashKey != lspec.PrehashKey {
		return fmt.Errorf("unexpected PrehashKey: %d", op.PrehashKey)
	}
	if op.PrehashValue != lspec.PrehashValue {
		return fmt.Errorf("unexpected PrehashValue: %d", op.PrehashValue)
	}
	if op.Length != lspec.Length {
		return fmt.Errorf("unexpected LengthOp: %d", op.Length)
	}
	if !bytes.HasPrefix(op.Prefix, lspec.Prefix) {
		return fmt.Errorf("leaf Prefix doesn't start with %X", lspec.Prefix)
	}
	return nil
}

// CheckAgainstSpec will verify the InnerOp is in the format defined in spec
func (op *InnerOp) CheckAgainstSpec(spec *ProofSpec, b int) error {
	if op.Hash != spec.InnerSpec.Hash {
		return fmt.Errorf("unexpected HashOp: %d", op.Hash)
	}

	if validateSpec(spec) {
		err := validateIavlOps(op, b)
		if err != nil {
			return err
		}
	}

	leafPrefix := spec.LeafSpec.Prefix
	if bytes.HasPrefix(op.Prefix, leafPrefix) {
		return fmt.Errorf("inner Prefix starts with %X", leafPrefix)
	}
	if len(op.Prefix) < int(spec.InnerSpec.MinPrefixLength) {
		return fmt.Errorf("innerOp prefix too short (%d)", len(op.Prefix))
	}
	maxLeftChildBytes := (len(spec.InnerSpec.ChildOrder) - 1) * int(spec.InnerSpec.ChildSize)
	if len(op.Prefix) > int(spec.InnerSpec.MaxPrefixLength)+maxLeftChildBytes {
		return fmt.Errorf("innerOp prefix too long (%d)", len(op.Prefix))
	}

	// ensures soundness, with suffix having to be of correct length
	if len(op.Suffix)%int(spec.InnerSpec.ChildSize) != 0 {
		return fmt.Errorf("InnerOp suffix malformed")
	}

	return nil
}

// doHash will preform the specified hash on the preimage.
// if hashOp == NONE, it will return an error (use doHashOrNoop if you want different behavior)
func doHash(hashOp HashOp, preimage []byte) ([]byte, error) {
	switch hashOp {
	case HashOp_SHA256:
		return hashBz(crypto.SHA256, preimage)
	case HashOp_SHA512:
		return hashBz(crypto.SHA512, preimage)
	case HashOp_RIPEMD160:
		return hashBz(crypto.RIPEMD160, preimage)
	case HashOp_BITCOIN:
		// ripemd160(sha256(x))
		sha := crypto.SHA256.New()
		sha.Write(preimage)
		tmp := sha.Sum(nil)
		hash := crypto.RIPEMD160.New()
		hash.Write(tmp)
		return hash.Sum(nil), nil
	case HashOp_SHA512_256:
		hash := crypto.SHA512_256.New()
		hash.Write(preimage)
		return hash.Sum(nil), nil
	}
	return nil, fmt.Errorf("unsupported hashop: %d", hashOp)
}

type hasher interface {
	New() hash.Hash
}

func hashBz(h hasher, preimage []byte) ([]byte, error) {
	hh := h.New()
	hh.Write(preimage)
	return hh.Sum(nil), nil
}

func prepareLeafData(hashOp HashOp, lengthOp LengthOp, data []byte) ([]byte, error) {
	// TODO: lengthop before or after hash ???
	hdata, err := doHashOrNoop(hashOp, data)
	if err != nil {
		return nil, err
	}
	ldata, err := doLengthOp(lengthOp, hdata)
	return ldata, err
}

func validateSpec(spec *ProofSpec) bool {
	return spec.SpecEquals(IavlSpec)
}

type opType interface {
	GetPrefix() []byte
	GetHash() HashOp
	Reset()
	String() string
}

// doLengthOp will calculate the proper prefix and return it prepended
//
//	doLengthOp(op, data) -> length(data) || data
func doLengthOp(lengthOp LengthOp, data []byte) ([]byte, error) {
	switch lengthOp {
	case LengthOp_NO_PREFIX:
		return data, nil
	case LengthOp_VAR_PROTO:
		res := append(encodeVarintProto(len(data)), data...)
		return res, nil
	case LengthOp_REQUIRE_32_BYTES:
		if len(data) != 32 {
			return nil, fmt.Errorf("data was %d bytes, not 32", len(data))
		}
		return data, nil
	case LengthOp_REQUIRE_64_BYTES:
		if len(data) != 64 {
			return nil, fmt.Errorf("data was %d bytes, not 64", len(data))
		}
		return data, nil
	case LengthOp_FIXED32_LITTLE:
		res := make([]byte, 4, 4+len(data))
		binary.LittleEndian.PutUint32(res[:4], uint32(len(data)))
		res = append(res, data...)
		return res, nil
		// TODO
		// case LengthOp_VAR_RLP:
		// case LengthOp_FIXED32_BIG:
		// case LengthOp_FIXED64_BIG:
		// case LengthOp_FIXED64_LITTLE:
	}
	return nil, fmt.Errorf("unsupported lengthop: %d", lengthOp)
}

// doHashOrNoop will return the preimage untouched if hashOp == NONE,
// otherwise, perform doHash
func doHashOrNoop(hashOp HashOp, preimage []byte) ([]byte, error) {
	if hashOp == HashOp_NO_HASH {
		return preimage, nil
	}
	return doHash(hashOp, preimage)
}
