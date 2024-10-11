package execution

import "github.com/pkg/errors"

var (
	// ErrParse corresponds to JSON-RPC code -32700.
	ErrParse = errors.New("invalid JSON was received by the server")
	// ErrInvalidRequest corresponds to JSON-RPC code -32600.
	ErrInvalidRequest = errors.New("JSON sent is not valid request object")
	// ErrMethodNotFound corresponds to JSON-RPC code -32601.
	ErrMethodNotFound = errors.New("method not found")
	// ErrInvalidParams corresponds to JSON-RPC code -32602.
	ErrInvalidParams = errors.New("invalid method parameter(s)")
	// ErrInternal corresponds to JSON-RPC code -32603.
	ErrInternal = errors.New("internal JSON-RPC error")
	// ErrServer corresponds to JSON-RPC code -32000.
	ErrServer = errors.New("client error while processing request")
	// ErrUnknownPayload corresponds to JSON-RPC code -38001.
	ErrUnknownPayload = errors.New("payload does not exist or is not available")
	// ErrInvalidForkchoiceState corresponds to JSON-RPC code -38002.
	ErrInvalidForkchoiceState = errors.New("invalid forkchoice state")
	// ErrInvalidPayloadAttributes corresponds to JSON-RPC code -38003.
	ErrInvalidPayloadAttributes = errors.New("payload attributes are invalid / inconsistent")
	// ErrUnknownPayloadStatus when the payload status is unknown.
	ErrUnknownPayloadStatus = errors.New("unknown payload status")
	// ErrConfigMismatch when the execution node's terminal total difficulty or
	// terminal block hash received via the API mismatches Prysm's configuration value.
	ErrConfigMismatch = errors.New("execution client configuration mismatch")
	// ErrAcceptedSyncingPayloadStatus when the status of the payload is syncing or accepted.
	ErrAcceptedSyncingPayloadStatus = errors.New("payload status is SYNCING or ACCEPTED")
	// ErrInvalidPayloadStatus when the status of the payload is invalid.
	ErrInvalidPayloadStatus = errors.New("payload status is INVALID")
	// ErrInvalidBlockHashPayloadStatus when the status of the payload fails to validate block hash.
	ErrInvalidBlockHashPayloadStatus = errors.New("payload status is INVALID_BLOCK_HASH")
	// ErrNilResponse when the response is nil.
	ErrNilResponse = errors.New("nil response")
	// ErrRequestTooLarge when the request is too large
	ErrRequestTooLarge = errors.New("request too large")
	// ErrUnsupportedVersion represents a case where a payload is requested for a block type that doesn't have a known mapping.
	ErrUnsupportedVersion = errors.New("unknown ExecutionPayload schema for block version")
)
