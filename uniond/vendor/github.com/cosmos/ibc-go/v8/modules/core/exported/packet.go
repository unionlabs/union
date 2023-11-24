package exported

// PacketI defines the standard interface for IBC packets
type PacketI interface {
	GetSequence() uint64
	GetTimeoutHeight() Height
	GetTimeoutTimestamp() uint64
	GetSourcePort() string
	GetSourceChannel() string
	GetDestPort() string
	GetDestChannel() string
	GetData() []byte
	ValidateBasic() error
}

// Acknowledgement defines the interface used to return acknowledgements in the OnRecvPacket callback.
// The Acknowledgement interface is used by core IBC to ensure partial state changes are not committed
// when packet receives have not properly succeeded (typically resulting in an error acknowledgement being returned).
// The interface also allows core IBC to obtain the acknowledgement bytes whose encoding is determined by each IBC application or middleware.
// Each custom acknowledgement type must implement this interface.
type Acknowledgement interface {
	// Success determines if the IBC application state should be persisted when handling `RecvPacket`.
	// During `OnRecvPacket` IBC application callback execution, all state changes are held in a cache store and committed if:
	// - the acknowledgement.Success() returns true
	// - a nil acknowledgement is returned (asynchronous acknowledgements)
	//
	// Note 1: IBC application callback events are always persisted so long as `RecvPacket` succeeds without error.
	//
	// Note 2: The return value should account for the success of the underlying IBC application or middleware. Thus the  `acknowledgement.Success` is representative of the entire IBC stack's success when receiving a packet. The individual success of each acknowledgement associated with an IBC application or middleware must be determined by obtaining the actual acknowledgement type after decoding the acknowledgement bytes.
	//
	// See https://github.com/cosmos/ibc-go/blob/v7.0.0/docs/ibc/apps.md for further explanations.
	Success() bool
	Acknowledgement() []byte
}

// PacketData defines an optional interface which an application's packet data structure may implement.
type PacketData interface {
	// GetPacketSender returns the sender address of the packet data.
	// If the packet sender is unknown or undefined, an empty string should be returned.
	GetPacketSender(sourcePortID string) string
}

// PacketDataProvider defines an optional interfaces for retrieving custom packet data stored on behalf of another application.
// An existing problem in the IBC middleware design is the inability for a middleware to define its own packet data type and insert packet sender provided information.
// A short term solution was introduced into several application's packet data to utilize a memo field to carry this information on behalf of another application.
// This interfaces standardizes that behaviour. Upon realization of the ability for middleware's to define their own packet data types, this interface will be deprecated and removed with time.
type PacketDataProvider interface {
	// GetCustomPacketData returns the packet data held on behalf of another application.
	// The name the information is stored under should be provided as the key.
	// If no custom packet data exists for the key, nil should be returned.
	GetCustomPacketData(key string) interface{}
}
