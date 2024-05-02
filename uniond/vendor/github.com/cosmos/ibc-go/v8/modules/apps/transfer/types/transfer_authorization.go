package types

import (
	"context"
	"encoding/json"
	"math/big"
	"sort"
	"strings"

	"github.com/cosmos/gogoproto/proto"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/authz"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

var _ authz.Authorization = (*TransferAuthorization)(nil)

// maxUint256 is the maximum value for a 256 bit unsigned integer.
var maxUint256 = new(big.Int).Sub(new(big.Int).Lsh(big.NewInt(1), 256), big.NewInt(1))

// NewTransferAuthorization creates a new TransferAuthorization object.
func NewTransferAuthorization(allocations ...Allocation) *TransferAuthorization {
	return &TransferAuthorization{
		Allocations: allocations,
	}
}

// MsgTypeURL implements Authorization.MsgTypeURL.
func (TransferAuthorization) MsgTypeURL() string {
	return sdk.MsgTypeURL(&MsgTransfer{})
}

// Accept implements Authorization.Accept.
func (a TransferAuthorization) Accept(ctx context.Context, msg proto.Message) (authz.AcceptResponse, error) {
	msgTransfer, ok := msg.(*MsgTransfer)
	if !ok {
		return authz.AcceptResponse{}, errorsmod.Wrap(ibcerrors.ErrInvalidType, "type mismatch")
	}

	for index, allocation := range a.Allocations {
		if !(allocation.SourceChannel == msgTransfer.SourceChannel && allocation.SourcePort == msgTransfer.SourcePort) {
			continue
		}

		if !isAllowedAddress(sdk.UnwrapSDKContext(ctx), msgTransfer.Receiver, allocation.AllowList) {
			return authz.AcceptResponse{}, errorsmod.Wrap(ibcerrors.ErrInvalidAddress, "not allowed receiver address for transfer")
		}

		err := validateMemo(sdk.UnwrapSDKContext(ctx), msgTransfer.Memo, allocation.AllowedPacketData)
		if err != nil {
			return authz.AcceptResponse{}, err
		}

		// If the spend limit is set to the MaxUint256 sentinel value, do not subtract the amount from the spend limit.
		if allocation.SpendLimit.AmountOf(msgTransfer.Token.Denom).Equal(UnboundedSpendLimit()) {
			return authz.AcceptResponse{Accept: true, Delete: false, Updated: nil}, nil
		}

		limitLeft, isNegative := allocation.SpendLimit.SafeSub(msgTransfer.Token)
		if isNegative {
			return authz.AcceptResponse{}, errorsmod.Wrapf(ibcerrors.ErrInsufficientFunds, "requested amount is more than spend limit")
		}

		if limitLeft.IsZero() {
			a.Allocations = append(a.Allocations[:index], a.Allocations[index+1:]...)
			if len(a.Allocations) == 0 {
				return authz.AcceptResponse{Accept: true, Delete: true}, nil
			}
			return authz.AcceptResponse{Accept: true, Delete: false, Updated: &TransferAuthorization{
				Allocations: a.Allocations,
			}}, nil
		}
		a.Allocations[index] = Allocation{
			SourcePort:        allocation.SourcePort,
			SourceChannel:     allocation.SourceChannel,
			SpendLimit:        limitLeft,
			AllowList:         allocation.AllowList,
			AllowedPacketData: allocation.AllowedPacketData,
		}

		return authz.AcceptResponse{Accept: true, Delete: false, Updated: &TransferAuthorization{
			Allocations: a.Allocations,
		}}, nil
	}

	return authz.AcceptResponse{}, errorsmod.Wrapf(ibcerrors.ErrNotFound, "requested port and channel allocation does not exist")
}

// ValidateBasic implements Authorization.ValidateBasic.
func (a TransferAuthorization) ValidateBasic() error {
	if len(a.Allocations) == 0 {
		return errorsmod.Wrap(ErrInvalidAuthorization, "allocations cannot be empty")
	}

	foundChannels := make(map[string]bool, 0)

	for _, allocation := range a.Allocations {
		if _, found := foundChannels[allocation.SourceChannel]; found {
			return errorsmod.Wrapf(channeltypes.ErrInvalidChannel, "duplicate source channel ID: %s", allocation.SourceChannel)
		}

		foundChannels[allocation.SourceChannel] = true

		if allocation.SpendLimit == nil {
			return errorsmod.Wrap(ibcerrors.ErrInvalidCoins, "spend limit cannot be nil")
		}

		if err := allocation.SpendLimit.Validate(); err != nil {
			return errorsmod.Wrapf(ibcerrors.ErrInvalidCoins, err.Error())
		}

		if err := host.PortIdentifierValidator(allocation.SourcePort); err != nil {
			return errorsmod.Wrap(err, "invalid source port ID")
		}

		if err := host.ChannelIdentifierValidator(allocation.SourceChannel); err != nil {
			return errorsmod.Wrap(err, "invalid source channel ID")
		}

		found := make(map[string]bool, 0)
		for i := 0; i < len(allocation.AllowList); i++ {
			if found[allocation.AllowList[i]] {
				return errorsmod.Wrapf(ErrInvalidAuthorization, "duplicate entry in allow list %s", allocation.AllowList[i])
			}
			found[allocation.AllowList[i]] = true
		}
	}

	return nil
}

// isAllowedAddress returns a boolean indicating if the receiver address is valid for transfer.
// gasCostPerIteration gas is consumed for each iteration.
func isAllowedAddress(ctx sdk.Context, receiver string, allowedAddrs []string) bool {
	if len(allowedAddrs) == 0 {
		return true
	}

	gasCostPerIteration := ctx.KVGasConfig().IterNextCostFlat

	for _, addr := range allowedAddrs {
		ctx.GasMeter().ConsumeGas(gasCostPerIteration, "transfer authorization")
		if addr == receiver {
			return true
		}
	}
	return false
}

// validateMemo returns a nil error indicating if the memo is valid for transfer.
func validateMemo(ctx sdk.Context, memo string, allowedPacketDataList []string) error {
	// if the allow list is empty, then the memo must be an empty string
	if len(allowedPacketDataList) == 0 {
		if len(strings.TrimSpace(memo)) != 0 {
			return errorsmod.Wrapf(ErrInvalidAuthorization, "memo must be empty because allowed packet data in allocation is empty")
		}

		return nil
	}

	// if allowedPacketDataList has only 1 element and it equals AllowAllPacketDataKeys
	// then accept all the packet data keys
	if len(allowedPacketDataList) == 1 && allowedPacketDataList[0] == AllowAllPacketDataKeys {
		return nil
	}

	jsonObject := make(map[string]interface{})
	err := json.Unmarshal([]byte(memo), &jsonObject)
	if err != nil {
		return err
	}

	gasCostPerIteration := ctx.KVGasConfig().IterNextCostFlat

	for _, key := range allowedPacketDataList {
		ctx.GasMeter().ConsumeGas(gasCostPerIteration, "transfer authorization")

		_, ok := jsonObject[key]
		if ok {
			delete(jsonObject, key)
		}
	}

	var keys []string
	for k := range jsonObject {
		keys = append(keys, k)
	}
	sort.Strings(keys)

	if len(jsonObject) != 0 {
		return errorsmod.Wrapf(ErrInvalidAuthorization, "not allowed packet data keys: %s", keys)
	}

	return nil
}

// UnboundedSpendLimit returns the sentinel value that can be used
// as the amount for a denomination's spend limit for which spend limit updating
// should be disabled. Please note that using this sentinel value means that a grantee
// will be granted the privilege to do ICS20 token transfers for the total amount
// of the denomination available at the granter's account.
func UnboundedSpendLimit() sdkmath.Int {
	return sdkmath.NewIntFromBigInt(maxUint256)
}
