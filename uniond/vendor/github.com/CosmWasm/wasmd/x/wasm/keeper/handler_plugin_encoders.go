package keeper

import (
	"encoding/json"
	"fmt"

	errorsmod "cosmossdk.io/errors"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	distributiontypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
	ibctransfertypes "github.com/cosmos/ibc-go/v7/modules/apps/transfer/types"
	ibcclienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

type (
	BankEncoder         func(sender sdk.AccAddress, msg *wasmvmtypes.BankMsg) ([]sdk.Msg, error)
	CustomEncoder       func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error)
	DistributionEncoder func(sender sdk.AccAddress, msg *wasmvmtypes.DistributionMsg) ([]sdk.Msg, error)
	StakingEncoder      func(sender sdk.AccAddress, msg *wasmvmtypes.StakingMsg) ([]sdk.Msg, error)
	StargateEncoder     func(sender sdk.AccAddress, msg *wasmvmtypes.StargateMsg) ([]sdk.Msg, error)
	WasmEncoder         func(sender sdk.AccAddress, msg *wasmvmtypes.WasmMsg) ([]sdk.Msg, error)
	IBCEncoder          func(ctx sdk.Context, sender sdk.AccAddress, contractIBCPortID string, msg *wasmvmtypes.IBCMsg) ([]sdk.Msg, error)
)

type MessageEncoders struct {
	Bank         func(sender sdk.AccAddress, msg *wasmvmtypes.BankMsg) ([]sdk.Msg, error)
	Custom       func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error)
	Distribution func(sender sdk.AccAddress, msg *wasmvmtypes.DistributionMsg) ([]sdk.Msg, error)
	IBC          func(ctx sdk.Context, sender sdk.AccAddress, contractIBCPortID string, msg *wasmvmtypes.IBCMsg) ([]sdk.Msg, error)
	Staking      func(sender sdk.AccAddress, msg *wasmvmtypes.StakingMsg) ([]sdk.Msg, error)
	Stargate     func(sender sdk.AccAddress, msg *wasmvmtypes.StargateMsg) ([]sdk.Msg, error)
	Wasm         func(sender sdk.AccAddress, msg *wasmvmtypes.WasmMsg) ([]sdk.Msg, error)
	Gov          func(sender sdk.AccAddress, msg *wasmvmtypes.GovMsg) ([]sdk.Msg, error)
}

func DefaultEncoders(unpacker codectypes.AnyUnpacker, portSource types.ICS20TransferPortSource) MessageEncoders {
	return MessageEncoders{
		Bank:         EncodeBankMsg,
		Custom:       NoCustomMsg,
		Distribution: EncodeDistributionMsg,
		IBC:          EncodeIBCMsg(portSource),
		Staking:      EncodeStakingMsg,
		Stargate:     EncodeStargateMsg(unpacker),
		Wasm:         EncodeWasmMsg,
		Gov:          EncodeGovMsg,
	}
}

func (e MessageEncoders) Merge(o *MessageEncoders) MessageEncoders {
	if o == nil {
		return e
	}
	if o.Bank != nil {
		e.Bank = o.Bank
	}
	if o.Custom != nil {
		e.Custom = o.Custom
	}
	if o.Distribution != nil {
		e.Distribution = o.Distribution
	}
	if o.IBC != nil {
		e.IBC = o.IBC
	}
	if o.Staking != nil {
		e.Staking = o.Staking
	}
	if o.Stargate != nil {
		e.Stargate = o.Stargate
	}
	if o.Wasm != nil {
		e.Wasm = o.Wasm
	}
	if o.Gov != nil {
		e.Gov = o.Gov
	}
	return e
}

func (e MessageEncoders) Encode(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) ([]sdk.Msg, error) {
	switch {
	case msg.Bank != nil:
		return e.Bank(contractAddr, msg.Bank)
	case msg.Custom != nil:
		return e.Custom(contractAddr, msg.Custom)
	case msg.Distribution != nil:
		return e.Distribution(contractAddr, msg.Distribution)
	case msg.IBC != nil:
		return e.IBC(ctx, contractAddr, contractIBCPortID, msg.IBC)
	case msg.Staking != nil:
		return e.Staking(contractAddr, msg.Staking)
	case msg.Stargate != nil:
		return e.Stargate(contractAddr, msg.Stargate)
	case msg.Wasm != nil:
		return e.Wasm(contractAddr, msg.Wasm)
	case msg.Gov != nil:
		return EncodeGovMsg(contractAddr, msg.Gov)
	}
	return nil, errorsmod.Wrap(types.ErrUnknownMsg, "unknown variant of Wasm")
}

func EncodeBankMsg(sender sdk.AccAddress, msg *wasmvmtypes.BankMsg) ([]sdk.Msg, error) {
	if msg.Send == nil {
		return nil, errorsmod.Wrap(types.ErrUnknownMsg, "unknown variant of Bank")
	}
	if len(msg.Send.Amount) == 0 {
		return nil, nil
	}
	toSend, err := ConvertWasmCoinsToSdkCoins(msg.Send.Amount)
	if err != nil {
		return nil, err
	}
	sdkMsg := banktypes.MsgSend{
		FromAddress: sender.String(),
		ToAddress:   msg.Send.ToAddress,
		Amount:      toSend,
	}
	return []sdk.Msg{&sdkMsg}, nil
}

func NoCustomMsg(_ sdk.AccAddress, _ json.RawMessage) ([]sdk.Msg, error) {
	return nil, errorsmod.Wrap(types.ErrUnknownMsg, "custom variant not supported")
}

func EncodeDistributionMsg(sender sdk.AccAddress, msg *wasmvmtypes.DistributionMsg) ([]sdk.Msg, error) {
	switch {
	case msg.SetWithdrawAddress != nil:
		setMsg := distributiontypes.MsgSetWithdrawAddress{
			DelegatorAddress: sender.String(),
			WithdrawAddress:  msg.SetWithdrawAddress.Address,
		}
		return []sdk.Msg{&setMsg}, nil
	case msg.WithdrawDelegatorReward != nil:
		withdrawMsg := distributiontypes.MsgWithdrawDelegatorReward{
			DelegatorAddress: sender.String(),
			ValidatorAddress: msg.WithdrawDelegatorReward.Validator,
		}
		return []sdk.Msg{&withdrawMsg}, nil
	default:
		return nil, errorsmod.Wrap(types.ErrUnknownMsg, "unknown variant of Distribution")
	}
}

func EncodeStakingMsg(sender sdk.AccAddress, msg *wasmvmtypes.StakingMsg) ([]sdk.Msg, error) {
	switch {
	case msg.Delegate != nil:
		coin, err := ConvertWasmCoinToSdkCoin(msg.Delegate.Amount)
		if err != nil {
			return nil, err
		}
		sdkMsg := stakingtypes.MsgDelegate{
			DelegatorAddress: sender.String(),
			ValidatorAddress: msg.Delegate.Validator,
			Amount:           coin,
		}
		return []sdk.Msg{&sdkMsg}, nil

	case msg.Redelegate != nil:
		coin, err := ConvertWasmCoinToSdkCoin(msg.Redelegate.Amount)
		if err != nil {
			return nil, err
		}
		sdkMsg := stakingtypes.MsgBeginRedelegate{
			DelegatorAddress:    sender.String(),
			ValidatorSrcAddress: msg.Redelegate.SrcValidator,
			ValidatorDstAddress: msg.Redelegate.DstValidator,
			Amount:              coin,
		}
		return []sdk.Msg{&sdkMsg}, nil
	case msg.Undelegate != nil:
		coin, err := ConvertWasmCoinToSdkCoin(msg.Undelegate.Amount)
		if err != nil {
			return nil, err
		}
		sdkMsg := stakingtypes.MsgUndelegate{
			DelegatorAddress: sender.String(),
			ValidatorAddress: msg.Undelegate.Validator,
			Amount:           coin,
		}
		return []sdk.Msg{&sdkMsg}, nil
	default:
		return nil, errorsmod.Wrap(types.ErrUnknownMsg, "unknown variant of Staking")
	}
}

func EncodeStargateMsg(unpacker codectypes.AnyUnpacker) StargateEncoder {
	return func(sender sdk.AccAddress, msg *wasmvmtypes.StargateMsg) ([]sdk.Msg, error) {
		codecAny := codectypes.Any{
			TypeUrl: msg.TypeURL,
			Value:   msg.Value,
		}
		var sdkMsg sdk.Msg
		if err := unpacker.UnpackAny(&codecAny, &sdkMsg); err != nil {
			return nil, errorsmod.Wrap(types.ErrInvalidMsg, fmt.Sprintf("Cannot unpack proto message with type URL: %s", msg.TypeURL))
		}
		if err := codectypes.UnpackInterfaces(sdkMsg, unpacker); err != nil {
			return nil, errorsmod.Wrap(types.ErrInvalidMsg, fmt.Sprintf("UnpackInterfaces inside msg: %s", err))
		}
		return []sdk.Msg{sdkMsg}, nil
	}
}

func EncodeWasmMsg(sender sdk.AccAddress, msg *wasmvmtypes.WasmMsg) ([]sdk.Msg, error) {
	switch {
	case msg.Execute != nil:
		coins, err := ConvertWasmCoinsToSdkCoins(msg.Execute.Funds)
		if err != nil {
			return nil, err
		}

		sdkMsg := types.MsgExecuteContract{
			Sender:   sender.String(),
			Contract: msg.Execute.ContractAddr,
			Msg:      msg.Execute.Msg,
			Funds:    coins,
		}
		return []sdk.Msg{&sdkMsg}, nil
	case msg.Instantiate != nil:
		coins, err := ConvertWasmCoinsToSdkCoins(msg.Instantiate.Funds)
		if err != nil {
			return nil, err
		}

		sdkMsg := types.MsgInstantiateContract{
			Sender: sender.String(),
			CodeID: msg.Instantiate.CodeID,
			Label:  msg.Instantiate.Label,
			Msg:    msg.Instantiate.Msg,
			Admin:  msg.Instantiate.Admin,
			Funds:  coins,
		}
		return []sdk.Msg{&sdkMsg}, nil
	case msg.Instantiate2 != nil:
		coins, err := ConvertWasmCoinsToSdkCoins(msg.Instantiate2.Funds)
		if err != nil {
			return nil, err
		}

		sdkMsg := types.MsgInstantiateContract2{
			Sender: sender.String(),
			Admin:  msg.Instantiate2.Admin,
			CodeID: msg.Instantiate2.CodeID,
			Label:  msg.Instantiate2.Label,
			Msg:    msg.Instantiate2.Msg,
			Funds:  coins,
			Salt:   msg.Instantiate2.Salt,
			// FixMsg is discouraged, see: https://medium.com/cosmwasm/dev-note-3-limitations-of-instantiate2-and-how-to-deal-with-them-a3f946874230
			FixMsg: false,
		}
		return []sdk.Msg{&sdkMsg}, nil
	case msg.Migrate != nil:
		sdkMsg := types.MsgMigrateContract{
			Sender:   sender.String(),
			Contract: msg.Migrate.ContractAddr,
			CodeID:   msg.Migrate.NewCodeID,
			Msg:      msg.Migrate.Msg,
		}
		return []sdk.Msg{&sdkMsg}, nil
	case msg.ClearAdmin != nil:
		sdkMsg := types.MsgClearAdmin{
			Sender:   sender.String(),
			Contract: msg.ClearAdmin.ContractAddr,
		}
		return []sdk.Msg{&sdkMsg}, nil
	case msg.UpdateAdmin != nil:
		sdkMsg := types.MsgUpdateAdmin{
			Sender:   sender.String(),
			Contract: msg.UpdateAdmin.ContractAddr,
			NewAdmin: msg.UpdateAdmin.Admin,
		}
		return []sdk.Msg{&sdkMsg}, nil
	default:
		return nil, errorsmod.Wrap(types.ErrUnknownMsg, "unknown variant of Wasm")
	}
}

func EncodeIBCMsg(portSource types.ICS20TransferPortSource) func(ctx sdk.Context, sender sdk.AccAddress, contractIBCPortID string, msg *wasmvmtypes.IBCMsg) ([]sdk.Msg, error) {
	return func(ctx sdk.Context, sender sdk.AccAddress, contractIBCPortID string, msg *wasmvmtypes.IBCMsg) ([]sdk.Msg, error) {
		switch {
		case msg.CloseChannel != nil:
			return []sdk.Msg{&channeltypes.MsgChannelCloseInit{
				PortId:    PortIDForContract(sender),
				ChannelId: msg.CloseChannel.ChannelID,
				Signer:    sender.String(),
			}}, nil
		case msg.Transfer != nil:
			amount, err := ConvertWasmCoinToSdkCoin(msg.Transfer.Amount)
			if err != nil {
				return nil, errorsmod.Wrap(err, "amount")
			}
			msg := &ibctransfertypes.MsgTransfer{
				SourcePort:       portSource.GetPort(ctx),
				SourceChannel:    msg.Transfer.ChannelID,
				Token:            amount,
				Sender:           sender.String(),
				Receiver:         msg.Transfer.ToAddress,
				TimeoutHeight:    ConvertWasmIBCTimeoutHeightToCosmosHeight(msg.Transfer.Timeout.Block),
				TimeoutTimestamp: msg.Transfer.Timeout.Timestamp,
			}
			return []sdk.Msg{msg}, nil
		default:
			return nil, errorsmod.Wrap(types.ErrUnknownMsg, "unknown variant of IBC")
		}
	}
}

func EncodeGovMsg(sender sdk.AccAddress, msg *wasmvmtypes.GovMsg) ([]sdk.Msg, error) {
	switch {
	case msg.Vote != nil:
		voteOption, err := convertVoteOption(msg.Vote.Vote)
		if err != nil {
			return nil, errorsmod.Wrap(err, "vote option")
		}
		m := v1.NewMsgVote(sender, msg.Vote.ProposalId, voteOption, "")
		return []sdk.Msg{m}, nil
	case msg.VoteWeighted != nil:
		opts := make([]*v1.WeightedVoteOption, len(msg.VoteWeighted.Options))
		for i, v := range msg.VoteWeighted.Options {
			weight, err := sdk.NewDecFromStr(v.Weight)
			if err != nil {
				return nil, errorsmod.Wrapf(err, "weight for vote %d", i+1)
			}
			voteOption, err := convertVoteOption(v.Option)
			if err != nil {
				return nil, errorsmod.Wrap(err, "vote option")
			}
			opts[i] = &v1.WeightedVoteOption{Option: voteOption, Weight: weight.String()}
		}
		m := v1.NewMsgVoteWeighted(sender, msg.VoteWeighted.ProposalId, opts, "")
		return []sdk.Msg{m}, nil

	default:
		return nil, types.ErrUnknownMsg.Wrap("unknown variant of gov")
	}
}

func convertVoteOption(s interface{}) (v1.VoteOption, error) {
	var option v1.VoteOption
	switch s {
	case wasmvmtypes.Yes:
		option = v1.OptionYes
	case wasmvmtypes.No:
		option = v1.OptionNo
	case wasmvmtypes.NoWithVeto:
		option = v1.OptionNoWithVeto
	case wasmvmtypes.Abstain:
		option = v1.OptionAbstain
	default:
		return v1.OptionEmpty, types.ErrInvalid
	}
	return option, nil
}

// ConvertWasmIBCTimeoutHeightToCosmosHeight converts a wasmvm type ibc timeout height to ibc module type height
func ConvertWasmIBCTimeoutHeightToCosmosHeight(ibcTimeoutBlock *wasmvmtypes.IBCTimeoutBlock) ibcclienttypes.Height {
	if ibcTimeoutBlock == nil {
		return ibcclienttypes.NewHeight(0, 0)
	}
	return ibcclienttypes.NewHeight(ibcTimeoutBlock.Revision, ibcTimeoutBlock.Height)
}

// ConvertWasmCoinsToSdkCoins converts the wasm vm type coins to sdk type coins
func ConvertWasmCoinsToSdkCoins(coins []wasmvmtypes.Coin) (sdk.Coins, error) {
	var toSend sdk.Coins
	for _, coin := range coins {
		c, err := ConvertWasmCoinToSdkCoin(coin)
		if err != nil {
			return nil, err
		}
		toSend = toSend.Add(c)
	}
	return toSend.Sort(), nil
}

// ConvertWasmCoinToSdkCoin converts a wasm vm type coin to sdk type coin
func ConvertWasmCoinToSdkCoin(coin wasmvmtypes.Coin) (sdk.Coin, error) {
	amount, ok := sdk.NewIntFromString(coin.Amount)
	if !ok {
		return sdk.Coin{}, errorsmod.Wrap(sdkerrors.ErrInvalidCoins, coin.Amount+coin.Denom)
	}
	r := sdk.Coin{
		Denom:  coin.Denom,
		Amount: amount,
	}
	return r, r.Validate()
}
