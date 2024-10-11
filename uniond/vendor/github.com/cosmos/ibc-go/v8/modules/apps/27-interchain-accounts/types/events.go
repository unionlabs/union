package types

// ICS27 Interchain Accounts events
const (
	EventTypePacket = "ics27_packet"

	AttributeKeyAckError            = "error"
	AttributeKeyHostChannelID       = "host_channel_id"
	AttributeKeyControllerChannelID = "controller_channel_id"
	AttributeKeyAckSuccess          = "success"
)
