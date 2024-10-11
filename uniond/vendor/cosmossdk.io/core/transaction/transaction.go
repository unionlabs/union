package transaction

type (
	// Msg uses structural types to define the interface for a message.
	Msg = interface {
		Reset()
		String() string
		ProtoMessage()
	}
	Identity = []byte
)

// GenericMsg defines a generic version of a Msg.
// The GenericMsg refers to the non pointer version of Msg,
// and is required to allow its instantiations in generic contexts.
type GenericMsg[T any] interface {
	*T
	Msg
}

// Codec defines the TX codec, which converts a TX from bytes to its concrete representation.
type Codec[T Tx] interface {
	// Decode decodes the tx bytes into a DecodedTx, containing
	// both concrete and bytes representation of the tx.
	Decode([]byte) (T, error)
	// DecodeJSON decodes the tx JSON bytes into a DecodedTx
	DecodeJSON([]byte) (T, error)
}

// Tx defines the interface for a transaction.
// All custom transactions must implement this interface.
type Tx interface {
	// Hash returns the unique identifier for the Tx.
	Hash() [32]byte
	// GetMessages returns the list of state transitions of the Tx.
	GetMessages() ([]Msg, error)
	// GetSenders returns the tx state transition sender.
	GetSenders() ([]Identity, error) // TODO reduce this to a single identity if accepted
	// GetGasLimit returns the gas limit of the tx. Must return math.MaxUint64 for infinite gas
	// txs.
	GetGasLimit() (uint64, error)
	// Bytes returns the encoded version of this tx. Note: this is ideally cached
	// from the first instance of the decoding of the tx.
	Bytes() []byte
}
