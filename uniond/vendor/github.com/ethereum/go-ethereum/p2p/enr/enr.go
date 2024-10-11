// Copyright 2017 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

// Package enr implements Ethereum Node Records as defined in EIP-778. A node record holds
// arbitrary information about a node on the peer-to-peer network. Node information is
// stored in key/value pairs. To store and retrieve key/values in a record, use the Entry
// interface.
//
// # Signature Handling
//
// Records must be signed before transmitting them to another node.
//
// Decoding a record doesn't check its signature. Code working with records from an
// untrusted source must always verify two things: that the record uses an identity scheme
// deemed secure, and that the signature is valid according to the declared scheme.
//
// When creating a record, set the entries you want and use a signing function provided by
// the identity scheme to add the signature. Modifying a record invalidates the signature.
//
// Package enr supports the "secp256k1-keccak" identity scheme.
package enr

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"sort"

	"github.com/ethereum/go-ethereum/rlp"
)

const SizeLimit = 300 // maximum encoded size of a node record in bytes

var (
	ErrInvalidSig     = errors.New("invalid signature on node record")
	errNotSorted      = errors.New("record key/value pairs are not sorted by key")
	errDuplicateKey   = errors.New("record contains duplicate key")
	errIncompletePair = errors.New("record contains incomplete k/v pair")
	errIncompleteList = errors.New("record contains less than two list elements")
	errTooBig         = fmt.Errorf("record bigger than %d bytes", SizeLimit)
	errEncodeUnsigned = errors.New("can't encode unsigned record")
	errNotFound       = errors.New("no such key in record")
)

// An IdentityScheme is capable of verifying record signatures and
// deriving node addresses.
type IdentityScheme interface {
	Verify(r *Record, sig []byte) error
	NodeAddr(r *Record) []byte
}

// SchemeMap is a registry of named identity schemes.
type SchemeMap map[string]IdentityScheme

func (m SchemeMap) Verify(r *Record, sig []byte) error {
	s := m[r.IdentityScheme()]
	if s == nil {
		return ErrInvalidSig
	}
	return s.Verify(r, sig)
}

func (m SchemeMap) NodeAddr(r *Record) []byte {
	s := m[r.IdentityScheme()]
	if s == nil {
		return nil
	}
	return s.NodeAddr(r)
}

// Record represents a node record. The zero value is an empty record.
type Record struct {
	seq       uint64 // sequence number
	signature []byte // the signature
	raw       []byte // RLP encoded record
	pairs     []pair // sorted list of all key/value pairs
}

// pair is a key/value pair in a record.
type pair struct {
	k string
	v rlp.RawValue
}

// Size returns the encoded size of the record.
func (r *Record) Size() uint64 {
	if r.raw != nil {
		return uint64(len(r.raw))
	}
	return computeSize(r)
}

func computeSize(r *Record) uint64 {
	size := uint64(rlp.IntSize(r.seq))
	size += rlp.BytesSize(r.signature)
	for _, p := range r.pairs {
		size += rlp.StringSize(p.k)
		size += uint64(len(p.v))
	}
	return rlp.ListSize(size)
}

// Seq returns the sequence number.
func (r *Record) Seq() uint64 {
	return r.seq
}

// SetSeq updates the record sequence number. This invalidates any signature on the record.
// Calling SetSeq is usually not required because setting any key in a signed record
// increments the sequence number.
func (r *Record) SetSeq(s uint64) {
	r.signature = nil
	r.raw = nil
	r.seq = s
}

// Load retrieves the value of a key/value pair. The given Entry must be a pointer and will
// be set to the value of the entry in the record.
//
// Errors returned by Load are wrapped in KeyError. You can distinguish decoding errors
// from missing keys using the IsNotFound function.
func (r *Record) Load(e Entry) error {
	i := sort.Search(len(r.pairs), func(i int) bool { return r.pairs[i].k >= e.ENRKey() })
	if i < len(r.pairs) && r.pairs[i].k == e.ENRKey() {
		if err := rlp.DecodeBytes(r.pairs[i].v, e); err != nil {
			return &KeyError{Key: e.ENRKey(), Err: err}
		}
		return nil
	}
	return &KeyError{Key: e.ENRKey(), Err: errNotFound}
}

// Set adds or updates the given entry in the record. It panics if the value can't be
// encoded. If the record is signed, Set increments the sequence number and invalidates
// the sequence number.
func (r *Record) Set(e Entry) {
	blob, err := rlp.EncodeToBytes(e)
	if err != nil {
		panic(fmt.Errorf("enr: can't encode %s: %v", e.ENRKey(), err))
	}
	r.invalidate()

	pairs := make([]pair, len(r.pairs))
	copy(pairs, r.pairs)
	i := sort.Search(len(pairs), func(i int) bool { return pairs[i].k >= e.ENRKey() })
	switch {
	case i < len(pairs) && pairs[i].k == e.ENRKey():
		// element is present at r.pairs[i]
		pairs[i].v = blob
	case i < len(r.pairs):
		// insert pair before i-th elem
		el := pair{e.ENRKey(), blob}
		pairs = append(pairs, pair{})
		copy(pairs[i+1:], pairs[i:])
		pairs[i] = el
	default:
		// element should be placed at the end of r.pairs
		pairs = append(pairs, pair{e.ENRKey(), blob})
	}
	r.pairs = pairs
}

func (r *Record) invalidate() {
	if r.signature != nil {
		r.seq++
	}
	r.signature = nil
	r.raw = nil
}

// Signature returns the signature of the record.
func (r *Record) Signature() []byte {
	if r.signature == nil {
		return nil
	}
	cpy := make([]byte, len(r.signature))
	copy(cpy, r.signature)
	return cpy
}

// EncodeRLP implements rlp.Encoder. Encoding fails if
// the record is unsigned.
func (r Record) EncodeRLP(w io.Writer) error {
	if r.signature == nil {
		return errEncodeUnsigned
	}
	_, err := w.Write(r.raw)
	return err
}

// DecodeRLP implements rlp.Decoder. Decoding doesn't verify the signature.
func (r *Record) DecodeRLP(s *rlp.Stream) error {
	dec, raw, err := decodeRecord(s)
	if err != nil {
		return err
	}
	*r = dec
	r.raw = raw
	return nil
}

func decodeRecord(s *rlp.Stream) (dec Record, raw []byte, err error) {
	raw, err = s.Raw()
	if err != nil {
		return dec, raw, err
	}
	if len(raw) > SizeLimit {
		return dec, raw, errTooBig
	}

	// Decode the RLP container.
	s = rlp.NewStream(bytes.NewReader(raw), 0)
	if _, err := s.List(); err != nil {
		return dec, raw, err
	}
	if err = s.Decode(&dec.signature); err != nil {
		if err == rlp.EOL {
			err = errIncompleteList
		}
		return dec, raw, err
	}
	if err = s.Decode(&dec.seq); err != nil {
		if err == rlp.EOL {
			err = errIncompleteList
		}
		return dec, raw, err
	}
	// The rest of the record contains sorted k/v pairs.
	var prevkey string
	for i := 0; ; i++ {
		var kv pair
		if err := s.Decode(&kv.k); err != nil {
			if err == rlp.EOL {
				break
			}
			return dec, raw, err
		}
		if err := s.Decode(&kv.v); err != nil {
			if err == rlp.EOL {
				return dec, raw, errIncompletePair
			}
			return dec, raw, err
		}
		if i > 0 {
			if kv.k == prevkey {
				return dec, raw, errDuplicateKey
			}
			if kv.k < prevkey {
				return dec, raw, errNotSorted
			}
		}
		dec.pairs = append(dec.pairs, kv)
		prevkey = kv.k
	}
	return dec, raw, s.ListEnd()
}

// IdentityScheme returns the name of the identity scheme in the record.
func (r *Record) IdentityScheme() string {
	var id ID
	r.Load(&id)
	return string(id)
}

// VerifySignature checks whether the record is signed using the given identity scheme.
func (r *Record) VerifySignature(s IdentityScheme) error {
	return s.Verify(r, r.signature)
}

// SetSig sets the record signature. It returns an error if the encoded record is larger
// than the size limit or if the signature is invalid according to the passed scheme.
//
// You can also use SetSig to remove the signature explicitly by passing a nil scheme
// and signature.
//
// SetSig panics when either the scheme or the signature (but not both) are nil.
func (r *Record) SetSig(s IdentityScheme, sig []byte) error {
	switch {
	// Prevent storing invalid data.
	case s == nil && sig != nil:
		panic("enr: invalid call to SetSig with non-nil signature but nil scheme")
	case s != nil && sig == nil:
		panic("enr: invalid call to SetSig with nil signature but non-nil scheme")
	// Verify if we have a scheme.
	case s != nil:
		if err := s.Verify(r, sig); err != nil {
			return err
		}
		raw, err := r.encode(sig)
		if err != nil {
			return err
		}
		r.signature, r.raw = sig, raw
	// Reset otherwise.
	default:
		r.signature, r.raw = nil, nil
	}
	return nil
}

// AppendElements appends the sequence number and entries to the given slice.
func (r *Record) AppendElements(list []interface{}) []interface{} {
	list = append(list, r.seq)
	for _, p := range r.pairs {
		list = append(list, p.k, p.v)
	}
	return list
}

func (r *Record) encode(sig []byte) (raw []byte, err error) {
	list := make([]interface{}, 1, 2*len(r.pairs)+2)
	list[0] = sig
	list = r.AppendElements(list)
	if raw, err = rlp.EncodeToBytes(list); err != nil {
		return nil, err
	}
	if len(raw) > SizeLimit {
		return nil, errTooBig
	}
	return raw, nil
}
