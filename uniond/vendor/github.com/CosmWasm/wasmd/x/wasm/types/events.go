package types

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

const (
	// WasmModuleEventType is stored with any contract TX that returns non empty EventAttributes
	WasmModuleEventType = "wasm"
	// CustomContractEventPrefix contracts can create custom events. To not mix them with other system events they got the `wasm-` prefix.
	CustomContractEventPrefix = "wasm-"

	EventTypeStoreCode              = "store_code"
	EventTypeInstantiate            = "instantiate"
	EventTypeExecute                = "execute"
	EventTypeMigrate                = "migrate"
	EventTypePinCode                = "pin_code"
	EventTypeUnpinCode              = "unpin_code"
	EventTypeSudo                   = "sudo"
	EventTypeReply                  = "reply"
	EventTypeGovContractResult      = "gov_contract_result"
	EventTypeUpdateContractAdmin    = "update_contract_admin"
	EventTypeUpdateCodeAccessConfig = "update_code_access_config"
	EventTypePacketRecv             = "ibc_packet_received"
	// add new types to IsAcceptedEventOnRecvPacketErrorAck
)

// EmitAcknowledgementEvent emits an event signalling a successful or failed acknowledgement and including the error
// details if any.
func EmitAcknowledgementEvent(ctx sdk.Context, contractAddr sdk.AccAddress, ack exported.Acknowledgement, err error) {
	success := err == nil && (ack == nil || ack.Success())
	attributes := []sdk.Attribute{
		sdk.NewAttribute(sdk.AttributeKeyModule, ModuleName),
		sdk.NewAttribute(AttributeKeyContractAddr, contractAddr.String()),
		sdk.NewAttribute(AttributeKeyAckSuccess, fmt.Sprintf("%t", success)),
	}

	if err != nil {
		attributes = append(attributes, sdk.NewAttribute(AttributeKeyAckError, err.Error()))
	}

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			EventTypePacketRecv,
			attributes...,
		),
	)
}

// event attributes returned from contract execution
const (
	AttributeReservedPrefix = "_"

	AttributeKeyContractAddr        = "_contract_address"
	AttributeKeyCodeID              = "code_id"
	AttributeKeyChecksum            = "code_checksum"
	AttributeKeyResultDataHex       = "result"
	AttributeKeyRequiredCapability  = "required_capability"
	AttributeKeyNewAdmin            = "new_admin_address"
	AttributeKeyCodePermission      = "code_permission"
	AttributeKeyAuthorizedAddresses = "authorized_addresses"
	AttributeKeyAckSuccess          = "success"
	AttributeKeyAckError            = "error"
)
