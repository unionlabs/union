package errors

import (
	errorsmod "cosmossdk.io/errors"
)

// Type Aliases to errors module
//
// Deprecated: functionality of this package has been moved to it's own module:
//
//	cosmossdk.io/errors
//
// Please use the above module instead of this package.
var (
	SuccessABCICode    = errorsmod.SuccessABCICode
	ABCIInfo           = errorsmod.ABCIInfo
	UndefinedCodespace = errorsmod.UndefinedCodespace
	Register           = errorsmod.Register
	ABCIError          = errorsmod.ABCIError
	New                = errorsmod.New
	Wrap               = errorsmod.Wrap
	Wrapf              = errorsmod.Wrapf
	Recover            = errorsmod.Recover
	WithType           = errorsmod.WithType
	IsOf               = errorsmod.IsOf
	AssertNil          = errorsmod.AssertNil
)

// Error type alias for errorsmod.Error
//
// Deprecated: the type has been moved to cosmossdk.io/errors
// module. Please use the above module instead of this package.
type Error = errorsmod.Error

// RootCodespace is the codespace for all errors defined in this package
const RootCodespace = "sdk"

var (
	// ErrTxDecode is returned if we cannot parse a transaction
	ErrTxDecode = Register(RootCodespace, 2, "tx parse error")

	// ErrInvalidSequence is used the sequence number (nonce) is incorrect
	// for the signature
	ErrInvalidSequence = Register(RootCodespace, 3, "invalid sequence")

	// ErrUnauthorized is used whenever a request without sufficient
	// authorization is handled.
	ErrUnauthorized = Register(RootCodespace, 4, "unauthorized")

	// ErrInsufficientFunds is used when the account cannot pay requested amount.
	ErrInsufficientFunds = Register(RootCodespace, 5, "insufficient funds")

	// ErrUnknownRequest to doc
	ErrUnknownRequest = Register(RootCodespace, 6, "unknown request")

	// ErrInvalidAddress to doc
	ErrInvalidAddress = Register(RootCodespace, 7, "invalid address")

	// ErrInvalidPubKey to doc
	ErrInvalidPubKey = Register(RootCodespace, 8, "invalid pubkey")

	// ErrUnknownAddress to doc
	ErrUnknownAddress = Register(RootCodespace, 9, "unknown address")

	// ErrInvalidCoins to doc
	ErrInvalidCoins = Register(RootCodespace, 10, "invalid coins")

	// ErrOutOfGas to doc
	ErrOutOfGas = Register(RootCodespace, 11, "out of gas")

	// ErrMemoTooLarge to doc
	ErrMemoTooLarge = Register(RootCodespace, 12, "memo too large")

	// ErrInsufficientFee to doc
	ErrInsufficientFee = Register(RootCodespace, 13, "insufficient fee")

	// ErrTooManySignatures to doc
	ErrTooManySignatures = Register(RootCodespace, 14, "maximum number of signatures exceeded")

	// ErrNoSignatures to doc
	ErrNoSignatures = Register(RootCodespace, 15, "no signatures supplied")

	// ErrJSONMarshal defines an ABCI typed JSON marshalling error
	ErrJSONMarshal = Register(RootCodespace, 16, "failed to marshal JSON bytes")

	// ErrJSONUnmarshal defines an ABCI typed JSON unmarshalling error
	ErrJSONUnmarshal = Register(RootCodespace, 17, "failed to unmarshal JSON bytes")

	// ErrInvalidRequest defines an ABCI typed error where the request contains
	// invalid data.
	ErrInvalidRequest = Register(RootCodespace, 18, "invalid request")

	// ErrTxInMempoolCache defines an ABCI typed error where a tx already exists
	// in the mempool.
	ErrTxInMempoolCache = Register(RootCodespace, 19, "tx already in mempool")

	// ErrMempoolIsFull defines an ABCI typed error where the mempool is full.
	ErrMempoolIsFull = Register(RootCodespace, 20, "mempool is full")

	// ErrTxTooLarge defines an ABCI typed error where tx is too large.
	ErrTxTooLarge = Register(RootCodespace, 21, "tx too large")

	// ErrKeyNotFound defines an error when the key doesn't exist
	ErrKeyNotFound = Register(RootCodespace, 22, "key not found")

	// ErrWrongPassword defines an error when the key password is invalid.
	ErrWrongPassword = Register(RootCodespace, 23, "invalid account password")

	// ErrorInvalidSigner defines an error when the tx intended signer does not match the given signer.
	ErrorInvalidSigner = Register(RootCodespace, 24, "tx intended signer does not match the given signer")

	// ErrorInvalidGasAdjustment defines an error for an invalid gas adjustment
	ErrorInvalidGasAdjustment = Register(RootCodespace, 25, "invalid gas adjustment")

	// ErrInvalidHeight defines an error for an invalid height
	ErrInvalidHeight = Register(RootCodespace, 26, "invalid height")

	// ErrInvalidVersion defines a general error for an invalid version
	ErrInvalidVersion = Register(RootCodespace, 27, "invalid version")

	// ErrInvalidChainID defines an error when the chain-id is invalid.
	ErrInvalidChainID = Register(RootCodespace, 28, "invalid chain-id")

	// ErrInvalidType defines an error an invalid type.
	ErrInvalidType = Register(RootCodespace, 29, "invalid type")

	// ErrTxTimeoutHeight defines an error for when a tx is rejected out due to an
	// explicitly set timeout height.
	ErrTxTimeoutHeight = Register(RootCodespace, 30, "tx timeout height")

	// ErrUnknownExtensionOptions defines an error for unknown extension options.
	ErrUnknownExtensionOptions = Register(RootCodespace, 31, "unknown extension options")

	// ErrWrongSequence defines an error where the account sequence defined in
	// the signer info doesn't match the account's actual sequence number.
	ErrWrongSequence = Register(RootCodespace, 32, "incorrect account sequence")

	// ErrPackAny defines an error when packing a protobuf message to Any fails.
	ErrPackAny = Register(RootCodespace, 33, "failed packing protobuf message to Any")

	// ErrUnpackAny defines an error when unpacking a protobuf message from Any fails.
	ErrUnpackAny = Register(RootCodespace, 34, "failed unpacking protobuf message from Any")

	// ErrLogic defines an internal logic error, e.g. an invariant or assertion
	// that is violated. It is a programmer error, not a user-facing error.
	ErrLogic = Register(RootCodespace, 35, "internal logic error")

	// ErrConflict defines a conflict error, e.g. when two goroutines try to access
	// the same resource and one of them fails.
	ErrConflict = Register(RootCodespace, 36, "conflict")

	// ErrNotSupported is returned when we call a branch of a code which is currently not
	// supported.
	ErrNotSupported = Register(RootCodespace, 37, "feature not supported")

	// ErrNotFound defines an error when requested entity doesn't exist in the state.
	ErrNotFound = Register(RootCodespace, 38, "not found")

	// ErrIO should be used to wrap internal errors caused by external operation.
	// Examples: not DB domain error, file writing etc...
	ErrIO = Register(RootCodespace, 39, "Internal IO error")

	// ErrAppConfig defines an error occurred if min-gas-prices field in BaseConfig is empty.
	ErrAppConfig = Register(RootCodespace, 40, "error in app.toml")

	// ErrInvalidGasLimit defines an error when an invalid GasWanted value is
	// supplied.
	ErrInvalidGasLimit = Register(RootCodespace, 41, "invalid gas limit")

	// ErrPanic should only be set when we recovering from a panic
	ErrPanic = errorsmod.ErrPanic
)
