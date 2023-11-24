package exported

// ChannelI defines the standard interface for a channel end.
type ChannelI interface {
	GetState() int32
	GetOrdering() int32
	GetCounterparty() CounterpartyChannelI
	GetConnectionHops() []string
	GetVersion() string
	ValidateBasic() error
}

// CounterpartyChannelI defines the standard interface for a channel end's
// counterparty.
type CounterpartyChannelI interface {
	GetPortID() string
	GetChannelID() string
	ValidateBasic() error
}
