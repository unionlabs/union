package keeper

import (
	errorsmod "cosmossdk.io/errors"
	abci "github.com/cometbft/cometbft/abci/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// ValidatorSetSource is a subset of the staking keeper
type ValidatorSetSource interface {
	ApplyAndReturnValidatorSetUpdates(sdk.Context) (updates []abci.ValidatorUpdate, err error)
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

	var maxContractID int
	for i, contract := range data.Contracts {
		contractAddr, err := sdk.AccAddressFromBech32(contract.ContractAddress)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "address in contract number %d", i)
		}
		err = keeper.importContract(ctx, contractAddr, &contract.ContractInfo, contract.ContractState, contract.ContractCodeHistory)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "contract number %d", i)
		}
		maxContractID = i + 1 // not ideal but max(contractID) is not persisted otherwise
	}

	for i, seq := range data.Sequences {
		err := keeper.importAutoIncrementID(ctx, seq.IDKey, seq.Value)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "sequence number %d", i)
		}
	}

	// sanity check seq values
	seqVal := keeper.PeekAutoIncrementID(ctx, types.KeyLastCodeID)
	if seqVal <= maxCodeID {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "seq %s with value: %d must be greater than: %d ", string(types.KeyLastCodeID), seqVal, maxCodeID)
	}
	seqVal = keeper.PeekAutoIncrementID(ctx, types.KeyLastInstanceID)
	if seqVal <= uint64(maxContractID) {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "seq %s with value: %d must be greater than: %d ", string(types.KeyLastInstanceID), seqVal, maxContractID)
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

	for _, k := range [][]byte{types.KeyLastCodeID, types.KeyLastInstanceID} {
		genState.Sequences = append(genState.Sequences, types.Sequence{
			IDKey: k,
			Value: keeper.PeekAutoIncrementID(ctx, k),
		})
	}

	return &genState
}
