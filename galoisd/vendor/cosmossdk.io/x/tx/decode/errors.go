package decode

import "cosmossdk.io/errors"

const (
	txCodespace = "tx"
)

var (
	// ErrTxDecode is returned if we cannot parse a transaction
	ErrTxDecode     = errors.Register(txCodespace, 1, "tx parse error")
	ErrUnknownField = errors.Register(txCodespace, 2, "unknown protobuf field")
)
