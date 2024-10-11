package types

// IBC transfer events
const (
	EventTypeTimeout      = "timeout"
	EventTypePacket       = "fungible_token_packet"
	EventTypeTransfer     = "ibc_transfer"
	EventTypeChannelClose = "channel_closed"
	EventTypeDenomTrace   = "denomination_trace"

	AttributeKeyReceiver       = "receiver"
	AttributeKeyDenom          = "denom"
	AttributeKeyAmount         = "amount"
	AttributeKeyRefundReceiver = "refund_receiver"
	AttributeKeyRefundDenom    = "refund_denom"
	AttributeKeyRefundAmount   = "refund_amount"
	AttributeKeyAckSuccess     = "success"
	AttributeKeyAck            = "acknowledgement"
	AttributeKeyAckError       = "error"
	AttributeKeyTraceHash      = "trace_hash"
	AttributeKeyMemo           = "memo"
)
