package types

// 29-fee events
const (
	EventTypeIncentivizedPacket        = "incentivized_ibc_packet"
	EventTypeRegisterPayee             = "register_payee"
	EventTypeRegisterCounterpartyPayee = "register_counterparty_payee"
	EventTypeDistributeFee             = "distribute_fee"

	AttributeKeyRecvFee           = "recv_fee"
	AttributeKeyAckFee            = "ack_fee"
	AttributeKeyTimeoutFee        = "timeout_fee"
	AttributeKeyChannelID         = "channel_id"
	AttributeKeyRelayer           = "relayer"
	AttributeKeyPayee             = "payee"
	AttributeKeyCounterpartyPayee = "counterparty_payee"
	AttributeKeyReceiver          = "receiver"
	AttributeKeyFee               = "fee"
)
