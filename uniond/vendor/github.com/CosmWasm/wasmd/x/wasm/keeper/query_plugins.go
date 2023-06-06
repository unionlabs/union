package keeper

import (
	"encoding/json"
	"errors"
	"fmt"

	errorsmod "cosmossdk.io/errors"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	distributiontypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

type QueryHandler struct {
	Ctx         sdk.Context
	Plugins     WasmVMQueryHandler
	Caller      sdk.AccAddress
	gasRegister GasRegister
}

func NewQueryHandler(ctx sdk.Context, vmQueryHandler WasmVMQueryHandler, caller sdk.AccAddress, gasRegister GasRegister) QueryHandler {
	return QueryHandler{
		Ctx:         ctx,
		Plugins:     vmQueryHandler,
		Caller:      caller,
		gasRegister: gasRegister,
	}
}

type GRPCQueryRouter interface {
	Route(path string) baseapp.GRPCQueryHandler
}

// -- end baseapp interfaces --

var _ wasmvmtypes.Querier = QueryHandler{}

func (q QueryHandler) Query(request wasmvmtypes.QueryRequest, gasLimit uint64) ([]byte, error) {
	// set a limit for a subCtx
	sdkGas := q.gasRegister.FromWasmVMGas(gasLimit)
	// discard all changes/ events in subCtx by not committing the cached context
	subCtx, _ := q.Ctx.WithGasMeter(sdk.NewGasMeter(sdkGas)).CacheContext()

	// make sure we charge the higher level context even on panic
	defer func() {
		q.Ctx.GasMeter().ConsumeGas(subCtx.GasMeter().GasConsumed(), "contract sub-query")
	}()

	res, err := q.Plugins.HandleQuery(subCtx, q.Caller, request)
	if err == nil {
		// short-circuit, the rest is dealing with handling existing errors
		return res, nil
	}

	// special mappings to wasmvm system error (which are not redacted)
	var wasmvmErr types.WasmVMErrorable
	if ok := errors.As(err, &wasmvmErr); ok {
		err = wasmvmErr.ToWasmVMError()
	}

	// Issue #759 - we don't return error string for worries of non-determinism
	return nil, redactError(err)
}

func (q QueryHandler) GasConsumed() uint64 {
	return q.Ctx.GasMeter().GasConsumed()
}

type CustomQuerier func(ctx sdk.Context, request json.RawMessage) ([]byte, error)

type QueryPlugins struct {
	Bank     func(ctx sdk.Context, request *wasmvmtypes.BankQuery) ([]byte, error)
	Custom   CustomQuerier
	IBC      func(ctx sdk.Context, caller sdk.AccAddress, request *wasmvmtypes.IBCQuery) ([]byte, error)
	Staking  func(ctx sdk.Context, request *wasmvmtypes.StakingQuery) ([]byte, error)
	Stargate func(ctx sdk.Context, request *wasmvmtypes.StargateQuery) ([]byte, error)
	Wasm     func(ctx sdk.Context, request *wasmvmtypes.WasmQuery) ([]byte, error)
}

type contractMetaDataSource interface {
	GetContractInfo(ctx sdk.Context, contractAddress sdk.AccAddress) *types.ContractInfo
}

type wasmQueryKeeper interface {
	contractMetaDataSource
	GetCodeInfo(ctx sdk.Context, codeID uint64) *types.CodeInfo
	QueryRaw(ctx sdk.Context, contractAddress sdk.AccAddress, key []byte) []byte
	QuerySmart(ctx sdk.Context, contractAddr sdk.AccAddress, req []byte) ([]byte, error)
	IsPinnedCode(ctx sdk.Context, codeID uint64) bool
}

func DefaultQueryPlugins(
	bank types.BankViewKeeper,
	staking types.StakingKeeper,
	distKeeper types.DistributionKeeper,
	channelKeeper types.ChannelKeeper,
	wasm wasmQueryKeeper,
) QueryPlugins {
	return QueryPlugins{
		Bank:     BankQuerier(bank),
		Custom:   NoCustomQuerier,
		IBC:      IBCQuerier(wasm, channelKeeper),
		Staking:  StakingQuerier(staking, distKeeper),
		Stargate: RejectStargateQuerier(),
		Wasm:     WasmQuerier(wasm),
	}
}

func (e QueryPlugins) Merge(o *QueryPlugins) QueryPlugins {
	// only update if this is non-nil and then only set values
	if o == nil {
		return e
	}
	if o.Bank != nil {
		e.Bank = o.Bank
	}
	if o.Custom != nil {
		e.Custom = o.Custom
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
	return e
}

// HandleQuery executes the requested query
func (e QueryPlugins) HandleQuery(ctx sdk.Context, caller sdk.AccAddress, request wasmvmtypes.QueryRequest) ([]byte, error) {
	// do the query
	if request.Bank != nil {
		return e.Bank(ctx, request.Bank)
	}
	if request.Custom != nil {
		return e.Custom(ctx, request.Custom)
	}
	if request.IBC != nil {
		return e.IBC(ctx, caller, request.IBC)
	}
	if request.Staking != nil {
		return e.Staking(ctx, request.Staking)
	}
	if request.Stargate != nil {
		return e.Stargate(ctx, request.Stargate)
	}
	if request.Wasm != nil {
		return e.Wasm(ctx, request.Wasm)
	}
	return nil, wasmvmtypes.Unknown{}
}

func BankQuerier(bankKeeper types.BankViewKeeper) func(ctx sdk.Context, request *wasmvmtypes.BankQuery) ([]byte, error) {
	return func(ctx sdk.Context, request *wasmvmtypes.BankQuery) ([]byte, error) {
		if request.AllBalances != nil {
			addr, err := sdk.AccAddressFromBech32(request.AllBalances.Address)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, request.AllBalances.Address)
			}
			coins := bankKeeper.GetAllBalances(ctx, addr)
			res := wasmvmtypes.AllBalancesResponse{
				Amount: ConvertSdkCoinsToWasmCoins(coins),
			}
			return json.Marshal(res)
		}
		if request.Balance != nil {
			addr, err := sdk.AccAddressFromBech32(request.Balance.Address)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, request.Balance.Address)
			}
			coin := bankKeeper.GetBalance(ctx, addr, request.Balance.Denom)
			res := wasmvmtypes.BalanceResponse{
				Amount: wasmvmtypes.Coin{
					Denom:  coin.Denom,
					Amount: coin.Amount.String(),
				},
			}
			return json.Marshal(res)
		}
		if request.Supply != nil {
			coin := bankKeeper.GetSupply(ctx, request.Supply.Denom)
			res := wasmvmtypes.SupplyResponse{
				Amount: wasmvmtypes.Coin{
					Denom:  coin.Denom,
					Amount: coin.Amount.String(),
				},
			}
			return json.Marshal(res)
		}
		return nil, wasmvmtypes.UnsupportedRequest{Kind: "unknown BankQuery variant"}
	}
}

func NoCustomQuerier(sdk.Context, json.RawMessage) ([]byte, error) {
	return nil, wasmvmtypes.UnsupportedRequest{Kind: "custom"}
}

func IBCQuerier(wasm contractMetaDataSource, channelKeeper types.ChannelKeeper) func(ctx sdk.Context, caller sdk.AccAddress, request *wasmvmtypes.IBCQuery) ([]byte, error) {
	return func(ctx sdk.Context, caller sdk.AccAddress, request *wasmvmtypes.IBCQuery) ([]byte, error) {
		if request.PortID != nil {
			contractInfo := wasm.GetContractInfo(ctx, caller)
			res := wasmvmtypes.PortIDResponse{
				PortID: contractInfo.IBCPortID,
			}
			return json.Marshal(res)
		}
		if request.ListChannels != nil {
			portID := request.ListChannels.PortID
			channels := make(wasmvmtypes.IBCChannels, 0)
			channelKeeper.IterateChannels(ctx, func(ch channeltypes.IdentifiedChannel) bool {
				// it must match the port and be in open state
				if (portID == "" || portID == ch.PortId) && ch.State == channeltypes.OPEN {
					newChan := wasmvmtypes.IBCChannel{
						Endpoint: wasmvmtypes.IBCEndpoint{
							PortID:    ch.PortId,
							ChannelID: ch.ChannelId,
						},
						CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{
							PortID:    ch.Counterparty.PortId,
							ChannelID: ch.Counterparty.ChannelId,
						},
						Order:        ch.Ordering.String(),
						Version:      ch.Version,
						ConnectionID: ch.ConnectionHops[0],
					}
					channels = append(channels, newChan)
				}
				return false
			})
			res := wasmvmtypes.ListChannelsResponse{
				Channels: channels,
			}
			return json.Marshal(res)
		}
		if request.Channel != nil {
			channelID := request.Channel.ChannelID
			portID := request.Channel.PortID
			if portID == "" {
				contractInfo := wasm.GetContractInfo(ctx, caller)
				portID = contractInfo.IBCPortID
			}
			got, found := channelKeeper.GetChannel(ctx, portID, channelID)
			var channel *wasmvmtypes.IBCChannel
			// it must be in open state
			if found && got.State == channeltypes.OPEN {
				channel = &wasmvmtypes.IBCChannel{
					Endpoint: wasmvmtypes.IBCEndpoint{
						PortID:    portID,
						ChannelID: channelID,
					},
					CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{
						PortID:    got.Counterparty.PortId,
						ChannelID: got.Counterparty.ChannelId,
					},
					Order:        got.Ordering.String(),
					Version:      got.Version,
					ConnectionID: got.ConnectionHops[0],
				}
			}
			res := wasmvmtypes.ChannelResponse{
				Channel: channel,
			}
			return json.Marshal(res)
		}
		return nil, wasmvmtypes.UnsupportedRequest{Kind: "unknown IBCQuery variant"}
	}
}

// RejectStargateQuerier rejects all stargate queries
func RejectStargateQuerier() func(ctx sdk.Context, request *wasmvmtypes.StargateQuery) ([]byte, error) {
	return func(ctx sdk.Context, request *wasmvmtypes.StargateQuery) ([]byte, error) {
		return nil, wasmvmtypes.UnsupportedRequest{Kind: "Stargate queries are disabled"}
	}
}

// AcceptedStargateQueries define accepted Stargate queries as a map with path as key and response type as value.
// For example:
// acceptList["/cosmos.auth.v1beta1.Query/Account"]= &authtypes.QueryAccountResponse{}
type AcceptedStargateQueries map[string]codec.ProtoMarshaler

// AcceptListStargateQuerier supports a preconfigured set of stargate queries only.
// All arguments must be non nil.
//
// Warning: Chains need to test and maintain their accept list carefully.
// There were critical consensus breaking issues in the past with non-deterministic behaviour in the SDK.
//
// This queries can be set via WithQueryPlugins option in the wasm keeper constructor:
// WithQueryPlugins(&QueryPlugins{Stargate: AcceptListStargateQuerier(acceptList, queryRouter, codec)})
func AcceptListStargateQuerier(acceptList AcceptedStargateQueries, queryRouter GRPCQueryRouter, codec codec.Codec) func(ctx sdk.Context, request *wasmvmtypes.StargateQuery) ([]byte, error) {
	return func(ctx sdk.Context, request *wasmvmtypes.StargateQuery) ([]byte, error) {
		protoResponse, accepted := acceptList[request.Path]
		if !accepted {
			return nil, wasmvmtypes.UnsupportedRequest{Kind: fmt.Sprintf("'%s' path is not allowed from the contract", request.Path)}
		}

		route := queryRouter.Route(request.Path)
		if route == nil {
			return nil, wasmvmtypes.UnsupportedRequest{Kind: fmt.Sprintf("No route to query '%s'", request.Path)}
		}

		res, err := route(ctx, abci.RequestQuery{
			Data: request.Data,
			Path: request.Path,
		})
		if err != nil {
			return nil, err
		}

		return ConvertProtoToJSONMarshal(codec, protoResponse, res.Value)
	}
}

func StakingQuerier(keeper types.StakingKeeper, distKeeper types.DistributionKeeper) func(ctx sdk.Context, request *wasmvmtypes.StakingQuery) ([]byte, error) {
	return func(ctx sdk.Context, request *wasmvmtypes.StakingQuery) ([]byte, error) {
		if request.BondedDenom != nil {
			denom := keeper.BondDenom(ctx)
			res := wasmvmtypes.BondedDenomResponse{
				Denom: denom,
			}
			return json.Marshal(res)
		}
		if request.AllValidators != nil {
			validators := keeper.GetBondedValidatorsByPower(ctx)
			// validators := keeper.GetAllValidators(ctx)
			wasmVals := make([]wasmvmtypes.Validator, len(validators))
			for i, v := range validators {
				wasmVals[i] = wasmvmtypes.Validator{
					Address:       v.OperatorAddress,
					Commission:    v.Commission.Rate.String(),
					MaxCommission: v.Commission.MaxRate.String(),
					MaxChangeRate: v.Commission.MaxChangeRate.String(),
				}
			}
			res := wasmvmtypes.AllValidatorsResponse{
				Validators: wasmVals,
			}
			return json.Marshal(res)
		}
		if request.Validator != nil {
			valAddr, err := sdk.ValAddressFromBech32(request.Validator.Address)
			if err != nil {
				return nil, err
			}
			v, found := keeper.GetValidator(ctx, valAddr)
			res := wasmvmtypes.ValidatorResponse{}
			if found {
				res.Validator = &wasmvmtypes.Validator{
					Address:       v.OperatorAddress,
					Commission:    v.Commission.Rate.String(),
					MaxCommission: v.Commission.MaxRate.String(),
					MaxChangeRate: v.Commission.MaxChangeRate.String(),
				}
			}
			return json.Marshal(res)
		}
		if request.AllDelegations != nil {
			delegator, err := sdk.AccAddressFromBech32(request.AllDelegations.Delegator)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, request.AllDelegations.Delegator)
			}
			sdkDels := keeper.GetAllDelegatorDelegations(ctx, delegator)
			delegations, err := sdkToDelegations(ctx, keeper, sdkDels)
			if err != nil {
				return nil, err
			}
			res := wasmvmtypes.AllDelegationsResponse{
				Delegations: delegations,
			}
			return json.Marshal(res)
		}
		if request.Delegation != nil {
			delegator, err := sdk.AccAddressFromBech32(request.Delegation.Delegator)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, request.Delegation.Delegator)
			}
			validator, err := sdk.ValAddressFromBech32(request.Delegation.Validator)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, request.Delegation.Validator)
			}

			var res wasmvmtypes.DelegationResponse
			d, found := keeper.GetDelegation(ctx, delegator, validator)
			if found {
				res.Delegation, err = sdkToFullDelegation(ctx, keeper, distKeeper, d)
				if err != nil {
					return nil, err
				}
			}
			return json.Marshal(res)
		}
		return nil, wasmvmtypes.UnsupportedRequest{Kind: "unknown Staking variant"}
	}
}

func sdkToDelegations(ctx sdk.Context, keeper types.StakingKeeper, delegations []stakingtypes.Delegation) (wasmvmtypes.Delegations, error) {
	result := make([]wasmvmtypes.Delegation, len(delegations))
	bondDenom := keeper.BondDenom(ctx)

	for i, d := range delegations {
		delAddr, err := sdk.AccAddressFromBech32(d.DelegatorAddress)
		if err != nil {
			return nil, errorsmod.Wrap(err, "delegator address")
		}
		valAddr, err := sdk.ValAddressFromBech32(d.ValidatorAddress)
		if err != nil {
			return nil, errorsmod.Wrap(err, "validator address")
		}

		// shares to amount logic comes from here:
		// https://github.com/cosmos/cosmos-sdk/blob/v0.38.3/x/staking/keeper/querier.go#L404
		val, found := keeper.GetValidator(ctx, valAddr)
		if !found {
			return nil, errorsmod.Wrap(stakingtypes.ErrNoValidatorFound, "can't load validator for delegation")
		}
		amount := sdk.NewCoin(bondDenom, val.TokensFromShares(d.Shares).TruncateInt())

		result[i] = wasmvmtypes.Delegation{
			Delegator: delAddr.String(),
			Validator: valAddr.String(),
			Amount:    ConvertSdkCoinToWasmCoin(amount),
		}
	}
	return result, nil
}

func sdkToFullDelegation(ctx sdk.Context, keeper types.StakingKeeper, distKeeper types.DistributionKeeper, delegation stakingtypes.Delegation) (*wasmvmtypes.FullDelegation, error) {
	delAddr, err := sdk.AccAddressFromBech32(delegation.DelegatorAddress)
	if err != nil {
		return nil, errorsmod.Wrap(err, "delegator address")
	}
	valAddr, err := sdk.ValAddressFromBech32(delegation.ValidatorAddress)
	if err != nil {
		return nil, errorsmod.Wrap(err, "validator address")
	}
	val, found := keeper.GetValidator(ctx, valAddr)
	if !found {
		return nil, errorsmod.Wrap(stakingtypes.ErrNoValidatorFound, "can't load validator for delegation")
	}
	bondDenom := keeper.BondDenom(ctx)
	amount := sdk.NewCoin(bondDenom, val.TokensFromShares(delegation.Shares).TruncateInt())

	delegationCoins := ConvertSdkCoinToWasmCoin(amount)

	// FIXME: this is very rough but better than nothing...
	// https://github.com/CosmWasm/wasmd/issues/282
	// if this (val, delegate) pair is receiving a redelegation, it cannot redelegate more
	// otherwise, it can redelegate the full amount
	// (there are cases of partial funds redelegated, but this is a start)
	redelegateCoins := wasmvmtypes.NewCoin(0, bondDenom)
	if !keeper.HasReceivingRedelegation(ctx, delAddr, valAddr) {
		redelegateCoins = delegationCoins
	}

	// FIXME: make a cleaner way to do this (modify the sdk)
	// we need the info from `distKeeper.calculateDelegationRewards()`, but it is not public
	// neither is `queryDelegationRewards(ctx sdk.Context, _ []string, req abci.RequestQuery, k Keeper)`
	// so we go through the front door of the querier....
	accRewards, err := getAccumulatedRewards(ctx, distKeeper, delegation)
	if err != nil {
		return nil, err
	}

	return &wasmvmtypes.FullDelegation{
		Delegator:          delAddr.String(),
		Validator:          valAddr.String(),
		Amount:             delegationCoins,
		AccumulatedRewards: accRewards,
		CanRedelegate:      redelegateCoins,
	}, nil
}

// FIXME: simplify this enormously when
// https://github.com/cosmos/cosmos-sdk/issues/7466 is merged
func getAccumulatedRewards(ctx sdk.Context, distKeeper types.DistributionKeeper, delegation stakingtypes.Delegation) ([]wasmvmtypes.Coin, error) {
	// Try to get *delegator* reward info!
	params := distributiontypes.QueryDelegationRewardsRequest{
		DelegatorAddress: delegation.DelegatorAddress,
		ValidatorAddress: delegation.ValidatorAddress,
	}
	cache, _ := ctx.CacheContext()
	qres, err := distKeeper.DelegationRewards(sdk.WrapSDKContext(cache), &params)
	if err != nil {
		return nil, err
	}

	// now we have it, convert it into wasmvm types
	rewards := make([]wasmvmtypes.Coin, len(qres.Rewards))
	for i, r := range qres.Rewards {
		rewards[i] = wasmvmtypes.Coin{
			Denom:  r.Denom,
			Amount: r.Amount.TruncateInt().String(),
		}
	}
	return rewards, nil
}

func WasmQuerier(k wasmQueryKeeper) func(ctx sdk.Context, request *wasmvmtypes.WasmQuery) ([]byte, error) {
	return func(ctx sdk.Context, request *wasmvmtypes.WasmQuery) ([]byte, error) {
		switch {
		case request.Smart != nil:
			addr, err := sdk.AccAddressFromBech32(request.Smart.ContractAddr)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, request.Smart.ContractAddr)
			}
			msg := types.RawContractMessage(request.Smart.Msg)
			if err := msg.ValidateBasic(); err != nil {
				return nil, errorsmod.Wrap(err, "json msg")
			}
			return k.QuerySmart(ctx, addr, msg)
		case request.Raw != nil:
			addr, err := sdk.AccAddressFromBech32(request.Raw.ContractAddr)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, request.Raw.ContractAddr)
			}
			return k.QueryRaw(ctx, addr, request.Raw.Key), nil
		case request.ContractInfo != nil:
			contractAddr := request.ContractInfo.ContractAddr
			addr, err := sdk.AccAddressFromBech32(contractAddr)
			if err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, contractAddr)
			}
			info := k.GetContractInfo(ctx, addr)
			if info == nil {
				return nil, types.ErrNoSuchContractFn(contractAddr).
					Wrapf("address %s", contractAddr)
			}
			res := wasmvmtypes.ContractInfoResponse{
				CodeID:  info.CodeID,
				Creator: info.Creator,
				Admin:   info.Admin,
				Pinned:  k.IsPinnedCode(ctx, info.CodeID),
				IBCPort: info.IBCPortID,
			}
			return json.Marshal(res)
		case request.CodeInfo != nil:
			if request.CodeInfo.CodeID == 0 {
				return nil, types.ErrEmpty.Wrap("code id")
			}
			info := k.GetCodeInfo(ctx, request.CodeInfo.CodeID)
			if info == nil {
				return nil, types.ErrNoSuchCodeFn(request.CodeInfo.CodeID).
					Wrapf("code id %d", request.CodeInfo.CodeID)
			}

			res := wasmvmtypes.CodeInfoResponse{
				CodeID:   request.CodeInfo.CodeID,
				Creator:  info.Creator,
				Checksum: info.CodeHash,
			}
			return json.Marshal(res)
		}
		return nil, wasmvmtypes.UnsupportedRequest{Kind: "unknown WasmQuery variant"}
	}
}

// ConvertSdkCoinsToWasmCoins covert sdk type to wasmvm coins type
func ConvertSdkCoinsToWasmCoins(coins []sdk.Coin) wasmvmtypes.Coins {
	converted := make(wasmvmtypes.Coins, len(coins))
	for i, c := range coins {
		converted[i] = ConvertSdkCoinToWasmCoin(c)
	}
	return converted
}

// ConvertSdkCoinToWasmCoin covert sdk type to wasmvm coin type
func ConvertSdkCoinToWasmCoin(coin sdk.Coin) wasmvmtypes.Coin {
	return wasmvmtypes.Coin{
		Denom:  coin.Denom,
		Amount: coin.Amount.String(),
	}
}

// ConvertProtoToJSONMarshal  unmarshals the given bytes into a proto message and then marshals it to json.
// This is done so that clients calling stargate queries do not need to define their own proto unmarshalers,
// being able to use response directly by json marshalling, which is supported in cosmwasm.
func ConvertProtoToJSONMarshal(cdc codec.Codec, protoResponse codec.ProtoMarshaler, bz []byte) ([]byte, error) {
	// unmarshal binary into stargate response data structure
	err := cdc.Unmarshal(bz, protoResponse)
	if err != nil {
		return nil, errorsmod.Wrap(err, "to proto")
	}

	bz, err = cdc.MarshalJSON(protoResponse)
	if err != nil {
		return nil, errorsmod.Wrap(err, "to json")
	}

	protoResponse.Reset()
	return bz, nil
}

var _ WasmVMQueryHandler = WasmVMQueryHandlerFn(nil)

// WasmVMQueryHandlerFn is a helper to construct a function based query handler.
type WasmVMQueryHandlerFn func(ctx sdk.Context, caller sdk.AccAddress, request wasmvmtypes.QueryRequest) ([]byte, error)

// HandleQuery delegates call into wrapped WasmVMQueryHandlerFn
func (w WasmVMQueryHandlerFn) HandleQuery(ctx sdk.Context, caller sdk.AccAddress, request wasmvmtypes.QueryRequest) ([]byte, error) {
	return w(ctx, caller, request)
}
