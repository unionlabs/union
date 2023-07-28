package cbor

// JSONMarshalFunc is used to marshal interface to JSON encoded byte slice.
// Making it package level instead of embedded in Encoder brings
// some extra efforts at importing, but avoids value copy when the functions
// of Encoder being invoked.
// DO REMEMBER to set this variable at importing, or
// you might get a nil pointer dereference panic at runtime.
var JSONMarshalFunc func(v interface{}) ([]byte, error)

type Encoder struct{}

// AppendKey adds a key (string) to the binary encoded log message
func (e Encoder) AppendKey(dst []byte, key string) []byte {
	if len(dst) < 1 {
		dst = e.AppendBeginMarker(dst)
	}
	return e.AppendString(dst, key)
}
