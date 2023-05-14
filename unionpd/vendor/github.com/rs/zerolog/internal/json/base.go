package json

// JSONMarshalFunc is used to marshal interface to JSON encoded byte slice.
// Making it package level instead of embedded in Encoder brings
// some extra efforts at importing, but avoids value copy when the functions
// of Encoder being invoked.
// DO REMEMBER to set this variable at importing, or
// you might get a nil pointer dereference panic at runtime.
var JSONMarshalFunc func(v interface{}) ([]byte, error)

type Encoder struct{}

// AppendKey appends a new key to the output JSON.
func (e Encoder) AppendKey(dst []byte, key string) []byte {
	if dst[len(dst)-1] != '{' {
		dst = append(dst, ',')
	}
	return append(e.AppendString(dst, key), ':')
}
