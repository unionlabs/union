package types

import (
	"encoding/json"
	"strings"
	"time"

	errorsmod "cosmossdk.io/errors"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ ibcexported.PacketData         = (*InterchainAccountPacketData)(nil)
	_ ibcexported.PacketDataProvider = (*InterchainAccountPacketData)(nil)
)

// MaxMemoCharLength defines the maximum length for the InterchainAccountPacketData memo field
const MaxMemoCharLength = 32768

var (
	// DefaultRelativePacketTimeoutHeight is the default packet timeout height (in blocks) relative
	// to the current block height of the counterparty chain provided by the client state. The
	// timeout is disabled when set to 0.
	DefaultRelativePacketTimeoutHeight = "0-1000"

	// DefaultRelativePacketTimeoutTimestamp is the default packet timeout timestamp (in nanoseconds)
	// relative to the current block timestamp of the counterparty chain provided by the client
	// state. The timeout is disabled when set to 0. The default is currently set to a 10 minute
	// timeout.
	DefaultRelativePacketTimeoutTimestamp = uint64((time.Duration(10) * time.Minute).Nanoseconds())
)

// ValidateBasic performs basic validation of the interchain account packet data.
// The memo may be empty.
func (iapd InterchainAccountPacketData) ValidateBasic() error {
	if iapd.Type == UNSPECIFIED {
		return errorsmod.Wrap(ErrInvalidOutgoingData, "packet data type cannot be unspecified")
	}

	if len(iapd.Data) == 0 {
		return errorsmod.Wrap(ErrInvalidOutgoingData, "packet data cannot be empty")
	}

	if len(iapd.Memo) > MaxMemoCharLength {
		return errorsmod.Wrapf(ErrInvalidOutgoingData, "packet data memo cannot be greater than %d characters", MaxMemoCharLength)
	}

	return nil
}

// GetBytes returns the JSON marshalled interchain account packet data.
func (iapd InterchainAccountPacketData) GetBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&iapd))
}

// UnmarshalJSON unmarshals raw JSON bytes into an InterchainAccountPacketData.
func (iapd *InterchainAccountPacketData) UnmarshalJSON(bz []byte) error {
	return ModuleCdc.UnmarshalJSON(bz, iapd)
}

// GetBytes returns the JSON marshalled interchain account CosmosTx.
func (ct CosmosTx) GetBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&ct))
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (ct CosmosTx) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	for _, protoAny := range ct.Messages {
		err := unpacker.UnpackAny(protoAny, new(sdk.Msg))
		if err != nil {
			return err
		}
	}

	return nil
}

// GetPacketSender returns the sender address of the interchain accounts packet data.
// It is obtained from the source port ID by cutting off the ControllerPortPrefix.
// If the source port ID does not have the ControllerPortPrefix, then an empty string is returned.
//
// NOTE:
//   - The sender address is set by the packet sender and may not have been validated a signature
//     check if the packet sender isn't the interchain accounts module.
//   - The sender address must only be used by modules on the sending chain.
func (InterchainAccountPacketData) GetPacketSender(sourcePortID string) string {
	icaOwner, found := strings.CutPrefix(sourcePortID, ControllerPortPrefix)
	if !found {
		return ""
	}
	return icaOwner
}

// GetCustomPacketData interprets the memo field of the packet data as a JSON object
// and returns the value associated with the given key.
// If the key is missing or the memo is not properly formatted, then nil is returned.
func (iapd InterchainAccountPacketData) GetCustomPacketData(key string) interface{} {
	if len(iapd.Memo) == 0 {
		return nil
	}

	jsonObject := make(map[string]interface{})
	err := json.Unmarshal([]byte(iapd.Memo), &jsonObject)
	if err != nil {
		return nil
	}

	memoData, found := jsonObject[key]
	if !found {
		return nil
	}

	return memoData
}
