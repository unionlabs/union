package types

var (
	// 128K - 1
	MaxKeyLength = (1 << 17) - 1
	// 2G - 1
	MaxValueLength = (1 << 31) - 1
)

// AssertValidKey checks if the key is valid(key is not nil, not empty and within length limit)
func AssertValidKey(key []byte) {
	if len(key) == 0 {
		panic("key is nil or empty")
	}
	if len(key) > MaxKeyLength {
		panic("key is too large")
	}
}

// AssertValidValue checks if the value is valid(value is not nil and within length limit)
func AssertValidValue(value []byte) {
	if value == nil {
		panic("value is nil")
	}
	if len(value) > MaxValueLength {
		panic("value is too large")
	}
}
