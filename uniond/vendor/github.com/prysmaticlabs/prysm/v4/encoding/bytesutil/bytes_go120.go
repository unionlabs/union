//go:build go1.20
// +build go1.20

package bytesutil

// These methods use go1.20 syntax to convert a byte slice to a fixed size array.

// ToBytes4 is a convenience method for converting a byte slice to a fix
// sized 4 byte array. This method will truncate the input if it is larger
// than 4 bytes.
func ToBytes4(x []byte) [4]byte {
	return [4]byte(PadTo(x, 4))
}

// ToBytes20 is a convenience method for converting a byte slice to a fix
// sized 20 byte array. This method will truncate the input if it is larger
// than 20 bytes.
func ToBytes20(x []byte) [20]byte {
	return [20]byte(PadTo(x, 20))
}

// ToBytes32 is a convenience method for converting a byte slice to a fix
// sized 32 byte array. This method will truncate the input if it is larger
// than 32 bytes.
func ToBytes32(x []byte) [32]byte {
	return [32]byte(PadTo(x, 32))
}

// ToBytes48 is a convenience method for converting a byte slice to a fix
// sized 48 byte array. This method will truncate the input if it is larger
// than 48 bytes.
func ToBytes48(x []byte) [48]byte {
	return [48]byte(PadTo(x, 48))
}

// ToBytes64 is a convenience method for converting a byte slice to a fix
// sized 64 byte array. This method will truncate the input if it is larger
// than 64 bytes.
func ToBytes64(x []byte) [64]byte {
	return [64]byte(PadTo(x, 64))
}

// ToBytes96 is a convenience method for converting a byte slice to a fix
// sized 96 byte array. This method will truncate the input if it is larger
// than 96 bytes.
func ToBytes96(x []byte) [96]byte {
	return [96]byte(PadTo(x, 96))
}
