package keeper

import (
	"context"

	abci "github.com/cometbft/cometbft/abci/types"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// ValidatorSetSource is a subset of the staking keeper
type ValidatorSetSource interface {
	ApplyAndReturnValidatorSetUpdates(context.Context) (updates []abci.ValidatorUpdate, err error)
}

// InitGenesis sets supply information for genesis.
//
// CONTRACT: all types of accounts must have been already initialized/created
func InitGenesis(ctx sdk.Context, keeper *Keeper, data types.GenesisState) ([]abci.ValidatorUpdate, error) {
	contractKeeper := NewGovPermissionKeeper(keeper)
	err := keeper.SetParams(ctx, data.Params)
	if err != nil {
		return nil, errorsmod.Wrapf(err, "set params")
	}

	var maxCodeID uint64
	for i, code := range data.Codes {
		err := keeper.importCode(ctx, code.CodeID, code.CodeInfo, code.CodeBytes)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "code %d with id: %d", i, code.CodeID)
		}
		if code.CodeID > maxCodeID {
			maxCodeID = code.CodeID
		}
		if code.Pinned {
			if err := contractKeeper.PinCode(ctx, code.CodeID); err != nil {
				return nil, errorsmod.Wrapf(err, "contract number %d", i)
			}
		}
	}

	for i, contract := range data.Contracts {
		contractAddr, err := sdk.AccAddressFromBech32(contract.ContractAddress)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "address in contract number %d", i)
		}
		err = keeper.importContract(ctx, contractAddr, &contract.ContractInfo, contract.ContractState, contract.ContractCodeHistory) //nolint:gosec
		if err != nil {
			return nil, errorsmod.Wrapf(err, "contract number %d", i)
		}
	}

	for i, seq := range data.Sequences {
		err := keeper.importAutoIncrementID(ctx, seq.IDKey, seq.Value)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "sequence number %d", i)
		}
	}

	// sanity check seq values
	seqVal, err := keeper.PeekAutoIncrementID(ctx, types.KeySequenceCodeID)
	if err != nil {
		return nil, err
	}
	if seqVal <= maxCodeID {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "seq %s with value: %d must be greater than: %d ", string(types.KeySequenceCodeID), seqVal, maxCodeID)
	}
	// ensure next classic address is unused so that we know the sequence is good
	rCtx, _ := ctx.CacheContext()
	seqVal, err = keeper.PeekAutoIncrementID(rCtx, types.KeySequenceInstanceID)
	if err != nil {
		return nil, err
	}
	addr := keeper.ClassicAddressGenerator()(rCtx, seqVal, nil)
	if keeper.HasContractInfo(ctx, addr) {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "value: %d for seq %s was used already", seqVal, string(types.KeySequenceInstanceID))
	}
	return nil, nil
}

// ExportGenesis returns a GenesisState for a given context and keeper.
func ExportGenesis(ctx sdk.Context, keeper *Keeper) *types.GenesisState {
	var genState types.GenesisState

	genState.Params = keeper.GetParams(ctx)

	keeper.IterateCodeInfos(ctx, func(codeID uint64, info types.CodeInfo) bool {
		bytecode, err := keeper.GetByteCode(ctx, codeID)
		if err != nil {
			panic(err)
		}
		genState.Codes = append(genState.Codes, types.Code{
			CodeID:    codeID,
			CodeInfo:  info,
			CodeBytes: bytecode,
			Pinned:    keeper.IsPinnedCode(ctx, codeID),
		})
		return false
	})

	keeper.IterateContractInfo(ctx, func(addr sdk.AccAddress, contract types.ContractInfo) bool {
		var state []types.Model
		keeper.IterateContractState(ctx, addr, func(key, value []byte) bool {
			state = append(state, types.Model{Key: key, Value: value})
			return false
		})

		contractCodeHistory := keeper.GetContractHistory(ctx, addr)

		genState.Contracts = append(genState.Contracts, types.Contract{
			ContractAddress:     addr.String(),
			ContractInfo:        contract,
			ContractState:       state,
			ContractCodeHistory: contractCodeHistory,
		})
		return false
	})

	for _, k := range [][]byte{types.KeySequenceCodeID, types.KeySequenceInstanceID} {
		id, err := keeper.PeekAutoIncrementID(ctx, k)
		if err != nil {
			panic(err)
		}
		genState.Sequences = append(genState.Sequences, types.Sequence{
			IDKey: k,
			Value: id,
		})
	}

	return &genState
}
