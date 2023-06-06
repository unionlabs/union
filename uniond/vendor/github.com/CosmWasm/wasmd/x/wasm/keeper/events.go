package keeper

import (
	"fmt"
	"strings"

	errorsmod "cosmossdk.io/errors"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// newWasmModuleEvent creates with wasm module event for interacting with the given contract. Adds custom attributes
// to this event.
func newWasmModuleEvent(customAttributes []wasmvmtypes.EventAttribute, contractAddr sdk.AccAddress) (sdk.Events, error) {
	attrs, err := contractSDKEventAttributes(customAttributes, contractAddr)
	if err != nil {
		return nil, err
	}

	// each wasm invocation always returns one sdk.Event
	return sdk.Events{sdk.NewEvent(types.WasmModuleEventType, attrs...)}, nil
}

const eventTypeMinLength = 2

// newCustomEvents converts wasmvm events from a contract response to sdk type events
func newCustomEvents(evts wasmvmtypes.Events, contractAddr sdk.AccAddress) (sdk.Events, error) {
	events := make(sdk.Events, 0, len(evts))
	for _, e := range evts {
		typ := strings.TrimSpace(e.Type)
		if len(typ) <= eventTypeMinLength {
			return nil, errorsmod.Wrap(types.ErrInvalidEvent, fmt.Sprintf("Event type too short: '%s'", typ))
		}
		attributes, err := contractSDKEventAttributes(e.Attributes, contractAddr)
		if err != nil {
			return nil, err
		}
		events = append(events, sdk.NewEvent(fmt.Sprintf("%s%s", types.CustomContractEventPrefix, typ), attributes...))
	}
	return events, nil
}

// convert and add contract address issuing this event
func contractSDKEventAttributes(customAttributes []wasmvmtypes.EventAttribute, contractAddr sdk.AccAddress) ([]sdk.Attribute, error) {
	attrs := []sdk.Attribute{sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddr.String())}
	// append attributes from wasm to the sdk.Event
	for _, l := range customAttributes {
		// ensure key and value are non-empty (and trim what is there)
		key := strings.TrimSpace(l.Key)
		if len(key) == 0 {
			return nil, errorsmod.Wrap(types.ErrInvalidEvent, fmt.Sprintf("Empty attribute key. Value: %s", l.Value))
		}
		value := strings.TrimSpace(l.Value)
		// TODO: check if this is legal in the SDK - if it is, we can remove this check
		if len(value) == 0 {
			return nil, errorsmod.Wrap(types.ErrInvalidEvent, fmt.Sprintf("Empty attribute value. Key: %s", key))
		}
		// and reserve all _* keys for our use (not contract)
		if strings.HasPrefix(key, types.AttributeReservedPrefix) {
			return nil, errorsmod.Wrap(types.ErrInvalidEvent, fmt.Sprintf("Attribute key starts with reserved prefix %s: '%s'", types.AttributeReservedPrefix, key))
		}
		attrs = append(attrs, sdk.NewAttribute(key, value))
	}
	return attrs, nil
}
