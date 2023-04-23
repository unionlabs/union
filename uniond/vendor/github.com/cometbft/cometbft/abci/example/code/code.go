package code

// Return codes for the examples
const (
	CodeTypeOK            uint32 = 0
	CodeTypeEncodingError uint32 = 1
	CodeTypeBadNonce      uint32 = 2
	CodeTypeUnauthorized  uint32 = 3
	CodeTypeUnknownError  uint32 = 4
	CodeTypeExecuted      uint32 = 5
	CodeTypeRejected      uint32 = 6
)
