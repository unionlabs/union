package codec

// NewAltValueCodec returns a new AltValueCodec. canonicalValueCodec is the codec that you want the value
// to be encoded and decoded as, alternativeDecoder is a function that will attempt to decode the value
// in case the canonicalValueCodec fails to decode it.
func NewAltValueCodec[V any](canonicalValueCodec ValueCodec[V], alternativeDecoder func([]byte) (V, error)) ValueCodec[V] {
	return AltValueCodec[V]{
		canonicalValueCodec: canonicalValueCodec,
		alternativeDecoder:  alternativeDecoder,
	}
}

// AltValueCodec is a codec that can decode a value from state in an alternative format.
// This is useful for migrating data from one format to another. For example, in x/bank
// balances were initially encoded as sdk.Coin, now they are encoded as math.Int.
// The AltValueCodec will be trying to decode the value as math.Int, and if that fails,
// it will attempt to decode it as sdk.Coin.
// NOTE: if the canonical format can also decode the alternative format, then this codec
// will produce undefined and undesirable behavior.
type AltValueCodec[V any] struct {
	canonicalValueCodec ValueCodec[V]
	alternativeDecoder  func([]byte) (V, error)
}

// Decode will attempt to decode the value from state using the canonical value codec.
// If it fails to decode, it will attempt to decode the value using the alternative decoder.
func (a AltValueCodec[V]) Decode(b []byte) (V, error) {
	v, err := a.canonicalValueCodec.Decode(b)
	if err != nil {
		return a.alternativeDecoder(b)
	}
	return v, nil
}

// Below there is the implementation of ValueCodec relying on the canonical value codec.

func (a AltValueCodec[V]) Encode(value V) ([]byte, error) { return a.canonicalValueCodec.Encode(value) }

func (a AltValueCodec[V]) EncodeJSON(value V) ([]byte, error) {
	return a.canonicalValueCodec.EncodeJSON(value)
}

func (a AltValueCodec[V]) DecodeJSON(b []byte) (V, error) { return a.canonicalValueCodec.DecodeJSON(b) }

func (a AltValueCodec[V]) Stringify(value V) string { return a.canonicalValueCodec.Stringify(value) }

func (a AltValueCodec[V]) ValueType() string { return a.canonicalValueCodec.ValueType() }
