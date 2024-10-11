package types

import (
	gogoproto "github.com/cosmos/gogoproto/types/any"
)

// Any is an alias for gogoproto.Any. It represents a protocol buffer message
// that can contain any arbitrary data. This is used for encoding and decoding
// unknown or dynamic content in a type-safe manner.
type Any = gogoproto.Any

// AminoPacker is an alias for gogoproto.AminoPacker. It provides functionality
// for packing and unpacking data using the Amino encoding.
type AminoPacker = gogoproto.AminoPacker

// AminoUnpacker is an alias for gogoproto.AminoUnpacker. It is used for
// unpacking Amino-encoded data into Go types.
type AminoUnpacker = gogoproto.AminoUnpacker

// AminoJSONPacker is an alias for gogoproto.AminoJSONPacker. It allows for
// packing data into a JSON format using Amino encoding rules.
type AminoJSONPacker = gogoproto.AminoJSONPacker

// AminoJSONUnpacker is an alias for gogoproto.AminoJSONUnpacker. It provides
// the ability to unpack JSON data encoded with Amino encoding rules.
type AminoJSONUnpacker = gogoproto.AminoJSONUnpacker

// ProtoJSONPacker is an alias for gogoproto.ProtoJSONPacker. This is used for
// packing protocol buffer messages into a JSON format.
type ProtoJSONPacker = gogoproto.ProtoJSONPacker

// NewAnyWithValue is an alias for gogoproto.NewAnyWithCacheWithValue. This function
// creates a new Any instance containing the provided value, with caching
// mechanisms to improve performance.
var NewAnyWithValue = gogoproto.NewAnyWithCacheWithValue

// UnsafePackAny is an alias for gogoproto.UnsafePackAnyWithCache. This function
// packs a given message into an Any type without performing safety checks.
var UnsafePackAny = gogoproto.UnsafePackAnyWithCache
