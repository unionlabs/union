package types

import (
	"encoding/json"
	"strings"
	"time"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ ibcexported.PacketData         = (*FungibleTokenPacketData)(nil)
	_ ibcexported.PacketDataProvider = (*FungibleTokenPacketData)(nil)
)

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

// NewFungibleTokenPacketData contructs a new FungibleTokenPacketData instance
func NewFungibleTokenPacketData(
	denom string, amount string,
	sender, receiver string,
	memo string,
) FungibleTokenPacketData {
	return FungibleTokenPacketData{
		Denom:    denom,
		Amount:   amount,
		Sender:   sender,
		Receiver: receiver,
		Memo:     memo,
	}
}

// ValidateBasic is used for validating the token transfer.
// NOTE: The addresses formats are not validated as the sender and recipient can have different
// formats defined by their corresponding chains that are not known to IBC.
func (ftpd FungibleTokenPacketData) ValidateBasic() error {
	amount, ok := sdkmath.NewIntFromString(ftpd.Amount)
	if !ok {
		return errorsmod.Wrapf(ErrInvalidAmount, "unable to parse transfer amount (%s) into math.Int", ftpd.Amount)
	}
	if !amount.IsPositive() {
		return errorsmod.Wrapf(ErrInvalidAmount, "amount must be strictly positive: got %d", amount)
	}
	if strings.TrimSpace(ftpd.Sender) == "" {
		return errorsmod.Wrap(ibcerrors.ErrInvalidAddress, "sender address cannot be blank")
	}
	if strings.TrimSpace(ftpd.Receiver) == "" {
		return errorsmod.Wrap(ibcerrors.ErrInvalidAddress, "receiver address cannot be blank")
	}
	return ValidatePrefixedDenom(ftpd.Denom)
}

// GetBytes is a helper for serialising
func (ftpd FungibleTokenPacketData) GetBytes() []byte {
	return sdk.MustSortJSON(mustProtoMarshalJSON(&ftpd))
}

// GetPacketSender returns the sender address embedded in the packet data.
//
// NOTE:
//   - The sender address is set by the module which requested the packet to be sent,
//     and this module may not have validated the sender address by a signature check.
//   - The sender address must only be used by modules on the sending chain.
//   - sourcePortID is not used in this implementation.
func (ftpd FungibleTokenPacketData) GetPacketSender(sourcePortID string) string {
	return ftpd.Sender
}

// GetCustomPacketData interprets the memo field of the packet data as a JSON object
// and returns the value associated with the given key.
// If the key is missing or the memo is not properly formatted, then nil is returned.
func (ftpd FungibleTokenPacketData) GetCustomPacketData(key string) interface{} {
	if len(ftpd.Memo) == 0 {
		return nil
	}

	jsonObject := make(map[string]interface{})
	err := json.Unmarshal([]byte(ftpd.Memo), &jsonObject)
	if err != nil {
		return nil
	}

	memoData, found := jsonObject[key]
	if !found {
		return nil
	}

	return memoData
}
