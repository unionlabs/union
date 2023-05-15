package address

// Codec defines an interface to convert addresses from and to string/bytes.
type Codec interface {
	// StringToBytes decodes text to bytes
	StringToBytes(text string) ([]byte, error)
	// BytesToString encodes bytes to text
	BytesToString(bz []byte) (string, error)
}
