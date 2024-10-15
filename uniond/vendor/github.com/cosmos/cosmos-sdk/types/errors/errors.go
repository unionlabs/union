package errors

import (
	errorsmod "cosmossdk.io/errors"
)

// RootCodespace is the codespace for all errors defined in this package
const RootCodespace = "sdk"

var (
	// ErrTxDecode is returned if we cannot parse a transaction
	ErrTxDecode = errorsmod.Register(RootCodespace, 2, "tx parse error")

	// ErrInvalidSequence is used the sequence number (nonce) is incorrect
	// for the signature
	ErrInvalidSequence = errorsmod.Register(RootCodespace, 3, "invalid sequence")

	// ErrUnauthorized is used whenever a request without sufficient
	// authorization is handled.
	ErrUnauthorized = errorsmod.Register(RootCodespace, 4, "unauthorized")

	// ErrInsufficientFunds is used when the account cannot pay requested amount.
	ErrInsufficientFunds = errorsmod.Register(RootCodespace, 5, "insufficient funds")

	// ErrUnknownRequest to doc
	ErrUnknownRequest = errorsmod.Register(RootCodespace, 6, "unknown request")

	// ErrInvalidAddress to doc
	ErrInvalidAddress = errorsmod.Register(RootCodespace, 7, "invalid address")

	// ErrInvalidPubKey to doc
	ErrInvalidPubKey = errorsmod.Register(RootCodespace, 8, "invalid pubkey")

	// ErrUnknownAddress to doc
	ErrUnknownAddress = errorsmod.Register(RootCodespace, 9, "unknown address")

	// ErrInvalidCoins to doc
	ErrInvalidCoins = errorsmod.Register(RootCodespace, 10, "invalid coins")

	// ErrOutOfGas to doc
	ErrOutOfGas = errorsmod.Register(RootCodespace, 11, "out of gas")

	// ErrMemoTooLarge to doc
	ErrMemoTooLarge = errorsmod.Register(RootCodespace, 12, "memo too large")

	// ErrInsufficientFee to doc
	ErrInsufficientFee = errorsmod.Register(RootCodespace, 13, "insufficient fee")

	// ErrTooManySignatures to doc
	ErrTooManySignatures = errorsmod.Register(RootCodespace, 14, "maximum number of signatures exceeded")

	// ErrNoSignatures to doc
	ErrNoSignatures = errorsmod.Register(RootCodespace, 15, "no signatures supplied")

	// ErrJSONMarshal defines an ABCI typed JSON marshaling error
	ErrJSONMarshal = errorsmod.Register(RootCodespace, 16, "failed to marshal JSON bytes")

	// ErrJSONUnmarshal defines an ABCI typed JSON unmarshalling error
	ErrJSONUnmarshal = errorsmod.Register(RootCodespace, 17, "failed to unmarshal JSON bytes")

	// ErrInvalidRequest defines an ABCI typed error where the request contains
	// invalid data.
	ErrInvalidRequest = errorsmod.Register(RootCodespace, 18, "invalid request")

	// ErrTxInMempoolCache defines an ABCI typed error where a tx already exists
	// in the mempool.
	ErrTxInMempoolCache = errorsmod.Register(RootCodespace, 19, "tx already in mempool")

	// ErrMempoolIsFull defines an ABCI typed error where the mempool is full.
	ErrMempoolIsFull = errorsmod.Register(RootCodespace, 20, "mempool is full")

	// ErrTxTooLarge defines an ABCI typed error where tx is too large.
	ErrTxTooLarge = errorsmod.Register(RootCodespace, 21, "tx too large")

	// ErrKeyNotFound defines an error when the key doesn't exist
	ErrKeyNotFound = errorsmod.Register(RootCodespace, 22, "key not found")

	// ErrWrongPassword defines an error when the key password is invalid.
	ErrWrongPassword = errorsmod.Register(RootCodespace, 23, "invalid account password")

	// ErrorInvalidSigner defines an error when the tx intended signer does not match the given signer.
	ErrorInvalidSigner = errorsmod.Register(RootCodespace, 24, "tx intended signer does not match the given signer")

	// ErrorInvalidGasAdjustment defines an error for an invalid gas adjustment
	ErrorInvalidGasAdjustment = errorsmod.Register(RootCodespace, 25, "invalid gas adjustment")

	// ErrInvalidHeight defines an error for an invalid height
	ErrInvalidHeight = errorsmod.Register(RootCodespace, 26, "invalid height")

	// ErrInvalidVersion defines a general error for an invalid version
	ErrInvalidVersion = errorsmod.Register(RootCodespace, 27, "invalid version")

	// ErrInvalidChainID defines an error when the chain-id is invalid.
	ErrInvalidChainID = errorsmod.Register(RootCodespace, 28, "invalid chain-id")

	// ErrInvalidType defines an error an invalid type.
	ErrInvalidType = errorsmod.Register(RootCodespace, 29, "invalid type")

	// ErrTxTimeoutHeight defines an error for when a tx is rejected out due to an
	// explicitly set timeout height.
	ErrTxTimeoutHeight = errorsmod.Register(RootCodespace, 30, "tx timeout height")

	// ErrUnknownExtensionOptions defines an error for unknown extension options.
	ErrUnknownExtensionOptions = errorsmod.Register(RootCodespace, 31, "unknown extension options")

	// ErrWrongSequence defines an error where the account sequence defined in
	// the signer info doesn't match the account's actual sequence number.
	ErrWrongSequence = errorsmod.Register(RootCodespace, 32, "incorrect account sequence")

	// ErrPackAny defines an error when packing a protobuf message to Any fails.
	ErrPackAny = errorsmod.Register(RootCodespace, 33, "failed packing protobuf message to Any")

	// ErrUnpackAny defines an error when unpacking a protobuf message from Any fails.
	ErrUnpackAny = errorsmod.Register(RootCodespace, 34, "failed unpacking protobuf message from Any")

	// ErrLogic defines an internal logic error, e.g. an invariant or assertion
	// that is violated. It is a programmer error, not a user-facing error.
	ErrLogic = errorsmod.Register(RootCodespace, 35, "internal logic error")

	// ErrConflict defines a conflict error, e.g. when two goroutines try to access
	// the same resource and one of them fails.
	ErrConflict = errorsmod.Register(RootCodespace, 36, "conflict")

	// ErrNotSupported is returned when we call a branch of a code which is currently not
	// supported.
	ErrNotSupported = errorsmod.Register(RootCodespace, 37, "feature not supported")

	// ErrNotFound defines an error when requested entity doesn't exist in the state.
	ErrNotFound = errorsmod.Register(RootCodespace, 38, "not found")

	// ErrIO should be used to wrap internal errors caused by external operation.
	// Examples: not DB domain error, file writing etc...
	ErrIO = errorsmod.Register(RootCodespace, 39, "Internal IO error")

	// ErrAppConfig defines an error occurred if application configuration is
	// misconfigured.
	ErrAppConfig = errorsmod.Register(RootCodespace, 40, "error in app.toml")

	// ErrInvalidGasLimit defines an error when an invalid GasWanted value is
	// supplied.
	ErrInvalidGasLimit = errorsmod.Register(RootCodespace, 41, "invalid gas limit")

	// ErrPanic should only be set when we recovering from a panic
	ErrPanic = errorsmod.ErrPanic

	// ErrTxTimeout defines an error for when a tx is rejected out due to an
	// explicitly set timeout timestamp.
	ErrTxTimeout = errorsmod.Register(RootCodespace, 42, "tx timeout")
)
