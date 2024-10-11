package types

import (
	"fmt"

	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// IBC connection events
const (
	AttributeKeyConnectionID             = "connection_id"
	AttributeKeyClientID                 = "client_id"
	AttributeKeyCounterpartyClientID     = "counterparty_client_id"
	AttributeKeyCounterpartyConnectionID = "counterparty_connection_id"
)

// IBC connection events vars
var (
	EventTypeConnectionOpenInit    = "connection_open_init"
	EventTypeConnectionOpenTry     = "connection_open_try"
	EventTypeConnectionOpenAck     = "connection_open_ack"
	EventTypeConnectionOpenConfirm = "connection_open_confirm"

	AttributeValueCategory = fmt.Sprintf("%s_%s", ibcexported.ModuleName, SubModuleName)
)
