package types

import (
	fmt "fmt"

	"github.com/cosmos/gogoproto/proto"
	protov2 "google.golang.org/protobuf/proto"

	errorsmod "cosmossdk.io/errors"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// nolint:revive // XXX is reqired for proto compatibility
type Any struct {
	// A URL/resource name that uniquely identifies the type of the serialized
	// protocol buffer message. This string must contain at least
	// one "/" character. The last segment of the URL's path must represent
	// the fully qualified name of the type (as in
	// `path/google.protobuf.Duration`). The name should be in a canonical form
	// (e.g., leading "." is not accepted).
	//
	// In practice, teams usually precompile into the binary all types that they
	// expect it to use in the context of Any. However, for URLs which use the
	// scheme `http`, `https`, or no scheme, one can optionally set up a type
	// server that maps type URLs to message definitions as follows:
	//
	// * If no scheme is provided, `https` is assumed.
	// * An HTTP GET on the URL must yield a [google.protobuf.Type][]
	//   value in binary format, or produce an error.
	// * Applications are allowed to cache lookup results based on the
	//   URL, or have them precompiled into a binary to avoid any
	//   lookup. Therefore, binary compatibility needs to be preserved
	//   on changes to types. (Use versioned type names to manage
	//   breaking changes.)
	//
	// Note: this functionality is not currently available in the official
	// protobuf release, and it is not used for type URLs beginning with
	// type.googleapis.com.
	//
	// Schemes other than `http`, `https` (or the empty scheme) might be
	// used with implementation specific semantics.

	TypeUrl string `protobuf:"bytes,1,opt,name=type_url,json=typeUrl,proto3" json:"type_url,omitempty"`

	// Must be a valid serialized protocol buffer of the above specified type.
	Value []byte `protobuf:"bytes,2,opt,name=value,proto3" json:"value,omitempty"`

	XXX_NoUnkeyedLiteral struct{} `json:"-"`
	XXX_unrecognized     []byte   `json:"-"`
	XXX_sizecache        int32    `json:"-"`

	cachedValue interface{}

	compat *anyCompat
}

// NewAnyWithValue constructs a new Any packed with the value provided or
// returns an error if that value couldn't be packed. This also caches
// the packed value so that it can be retrieved from GetCachedValue without
// unmarshaling
func NewAnyWithValue(v proto.Message) (*Any, error) {
	if v == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrPackAny, "Expecting non nil value to create a new Any")
	}

	var (
		bz  []byte
		err error
	)
	if msg, ok := v.(protov2.Message); ok {
		protov2MarshalOpts := protov2.MarshalOptions{Deterministic: true}
		bz, err = protov2MarshalOpts.Marshal(msg)
	} else {
		bz, err = proto.Marshal(v)
	}
	if err != nil {
		return nil, err
	}

	return &Any{
		TypeUrl:     MsgTypeURL(v),
		Value:       bz,
		cachedValue: v,
	}, nil
}

// UnsafePackAny packs the value x in the Any and instead of returning the error
// in the case of a packing failure, keeps the cached value. This should only
// be used in situations where compatibility is needed with amino. Amino-only
// values can safely be packed using this method when they will only be
// marshaled with amino and not protobuf.
func UnsafePackAny(x interface{}) *Any {
	if msg, ok := x.(proto.Message); ok {
		any, err := NewAnyWithValue(msg)
		if err == nil {
			return any
		}
	}
	return &Any{cachedValue: x}
}

// pack packs the value x in the Any or returns an error. This also caches
// the packed value so that it can be retrieved from GetCachedValue without
// unmarshaling
func (any *Any) pack(x proto.Message) error {
	any.TypeUrl = MsgTypeURL(x)

	var (
		bz  []byte
		err error
	)
	if msg, ok := x.(protov2.Message); ok {
		protov2MarshalOpts := protov2.MarshalOptions{Deterministic: true}
		bz, err = protov2MarshalOpts.Marshal(msg)
	} else {
		bz, err = proto.Marshal(x)
	}
	if err != nil {
		return err
	}

	any.Value = bz
	any.cachedValue = x

	return nil
}

// GetCachedValue returns the cached value from the Any if present
func (any *Any) GetCachedValue() interface{} {
	return any.cachedValue
}

// GoString returns a string representing valid go code to reproduce the current state of
// the struct.
func (any *Any) GoString() string {
	if any == nil {
		return "nil"
	}
	extra := ""
	if any.XXX_unrecognized != nil {
		extra = fmt.Sprintf(",\n  XXX_unrecognized: %#v,\n", any.XXX_unrecognized)
	}
	return fmt.Sprintf("&Any{TypeUrl: %#v,\n  Value: %#v%s\n}",
		any.TypeUrl, any.Value, extra)
}

// String implements the stringer interface
func (any *Any) String() string {
	if any == nil {
		return "nil"
	}
	return fmt.Sprintf("&Any{TypeUrl:%v,Value:%v,XXX_unrecognized:%v}",
		any.TypeUrl, any.Value, any.XXX_unrecognized)
}
