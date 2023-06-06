package keeper

import (
	"bytes"
	"encoding/hex"
	"fmt"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// NewWasmProposalHandler creates a new governance Handler for wasm proposals
//
// Deprecated: Do not use.
func NewWasmProposalHandler(k decoratedKeeper, enabledProposalTypes []types.ProposalType) v1beta1.Handler {
	return NewWasmProposalHandlerX(NewGovPermissionKeeper(k), enabledProposalTypes)
}

// NewWasmProposalHandlerX creates a new governance Handler for wasm proposals
//
// Deprecated: Do not use.
func NewWasmProposalHandlerX(k types.ContractOpsKeeper, enabledProposalTypes []types.ProposalType) v1beta1.Handler {
	enabledTypes := make(map[string]struct{}, len(enabledProposalTypes))
	for i := range enabledProposalTypes {
		enabledTypes[string(enabledProposalTypes[i])] = struct{}{}
	}
	return func(ctx sdk.Context, content v1beta1.Content) error {
		if content == nil {
			return errorsmod.Wrap(sdkerrors.ErrUnknownRequest, "content must not be empty")
		}
		if _, ok := enabledTypes[content.ProposalType()]; !ok {
			return errorsmod.Wrapf(sdkerrors.ErrUnknownRequest, "unsupported wasm proposal content type: %q", content.ProposalType())
		}
		switch c := content.(type) {
		case *types.StoreCodeProposal:
			return handleStoreCodeProposal(ctx, k, *c)
		case *types.InstantiateContractProposal:
			return handleInstantiateProposal(ctx, k, *c)
		case *types.InstantiateContract2Proposal:
			return handleInstantiate2Proposal(ctx, k, *c)
		case *types.MigrateContractProposal:
			return handleMigrateProposal(ctx, k, *c)
		case *types.SudoContractProposal:
			return handleSudoProposal(ctx, k, *c)
		case *types.ExecuteContractProposal:
			return handleExecuteProposal(ctx, k, *c)
		case *types.UpdateAdminProposal:
			return handleUpdateAdminProposal(ctx, k, *c)
		case *types.ClearAdminProposal:
			return handleClearAdminProposal(ctx, k, *c)
		case *types.PinCodesProposal:
			return handlePinCodesProposal(ctx, k, *c)
		case *types.UnpinCodesProposal:
			return handleUnpinCodesProposal(ctx, k, *c)
		case *types.UpdateInstantiateConfigProposal:
			return handleUpdateInstantiateConfigProposal(ctx, k, *c)
		case *types.StoreAndInstantiateContractProposal:
			return handleStoreAndInstantiateContractProposal(ctx, k, *c)
		default:
			return errorsmod.Wrapf(sdkerrors.ErrUnknownRequest, "unrecognized wasm proposal content type: %T", c)
		}
	}
}

//nolint:staticcheck
func handleStoreCodeProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.StoreCodeProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}

	runAsAddr, err := sdk.AccAddressFromBech32(p.RunAs)
	if err != nil {
		return errorsmod.Wrap(err, "run as address")
	}
	codeID, checksum, err := k.Create(ctx, runAsAddr, p.WASMByteCode, p.InstantiatePermission)
	if err != nil {
		return err
	}

	if len(p.CodeHash) != 0 && !bytes.Equal(checksum, p.CodeHash) {
		return fmt.Errorf("code-hash mismatch: %X, checksum: %X", p.CodeHash, checksum)
	}

	// if code should not be pinned return earlier
	if p.UnpinCode {
		return nil
	}
	return k.PinCode(ctx, codeID)
}

//nolint:staticcheck
func handleInstantiateProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.InstantiateContractProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}
	runAsAddr, err := sdk.AccAddressFromBech32(p.RunAs)
	if err != nil {
		return errorsmod.Wrap(err, "run as address")
	}
	var adminAddr sdk.AccAddress
	if p.Admin != "" {
		if adminAddr, err = sdk.AccAddressFromBech32(p.Admin); err != nil {
			return errorsmod.Wrap(err, "admin")
		}
	}

	_, data, err := k.Instantiate(ctx, p.CodeID, runAsAddr, adminAddr, p.Msg, p.Label, p.Funds)
	if err != nil {
		return err
	}

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeGovContractResult,
		sdk.NewAttribute(types.AttributeKeyResultDataHex, hex.EncodeToString(data)),
	))
	return nil
}

//nolint:staticcheck
func handleInstantiate2Proposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.InstantiateContract2Proposal) error {
	// Validatebasic with proposal
	if err := p.ValidateBasic(); err != nil {
		return err
	}

	// Get runAsAddr as AccAddress
	runAsAddr, err := sdk.AccAddressFromBech32(p.RunAs)
	if err != nil {
		return errorsmod.Wrap(err, "run as address")
	}

	// Get admin address
	var adminAddr sdk.AccAddress
	if p.Admin != "" {
		if adminAddr, err = sdk.AccAddressFromBech32(p.Admin); err != nil {
			return errorsmod.Wrap(err, "admin")
		}
	}

	_, data, err := k.Instantiate2(ctx, p.CodeID, runAsAddr, adminAddr, p.Msg, p.Label, p.Funds, p.Salt, p.FixMsg)
	if err != nil {
		return err
	}

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeGovContractResult,
		sdk.NewAttribute(types.AttributeKeyResultDataHex, hex.EncodeToString(data)),
	))
	return nil
}

//nolint:staticcheck
func handleStoreAndInstantiateContractProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.StoreAndInstantiateContractProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}
	runAsAddr, err := sdk.AccAddressFromBech32(p.RunAs)
	if err != nil {
		return errorsmod.Wrap(err, "run as address")
	}
	var adminAddr sdk.AccAddress
	if p.Admin != "" {
		if adminAddr, err = sdk.AccAddressFromBech32(p.Admin); err != nil {
			return errorsmod.Wrap(err, "admin")
		}
	}

	codeID, checksum, err := k.Create(ctx, runAsAddr, p.WASMByteCode, p.InstantiatePermission)
	if err != nil {
		return err
	}

	if p.CodeHash != nil && !bytes.Equal(checksum, p.CodeHash) {
		return errorsmod.Wrap(fmt.Errorf("code-hash mismatch: %X, checksum: %X", p.CodeHash, checksum), "code-hash mismatch")
	}

	if !p.UnpinCode {
		if err := k.PinCode(ctx, codeID); err != nil {
			return err
		}
	}

	_, data, err := k.Instantiate(ctx, codeID, runAsAddr, adminAddr, p.Msg, p.Label, p.Funds)
	if err != nil {
		return err
	}

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeGovContractResult,
		sdk.NewAttribute(types.AttributeKeyResultDataHex, hex.EncodeToString(data)),
	))
	return nil
}

//nolint:staticcheck
func handleMigrateProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.MigrateContractProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}

	contractAddr, err := sdk.AccAddressFromBech32(p.Contract)
	if err != nil {
		return errorsmod.Wrap(err, "contract")
	}

	// runAs is not used if this is permissioned, so just put any valid address there (second contractAddr)
	data, err := k.Migrate(ctx, contractAddr, contractAddr, p.CodeID, p.Msg)
	if err != nil {
		return err
	}

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeGovContractResult,
		sdk.NewAttribute(types.AttributeKeyResultDataHex, hex.EncodeToString(data)),
	))
	return nil
}

//nolint:staticcheck
func handleSudoProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.SudoContractProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}

	contractAddr, err := sdk.AccAddressFromBech32(p.Contract)
	if err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	data, err := k.Sudo(ctx, contractAddr, p.Msg)
	if err != nil {
		return err
	}

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeGovContractResult,
		sdk.NewAttribute(types.AttributeKeyResultDataHex, hex.EncodeToString(data)),
	))
	return nil
}

//nolint:staticcheck
func handleExecuteProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.ExecuteContractProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}

	contractAddr, err := sdk.AccAddressFromBech32(p.Contract)
	if err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	runAsAddr, err := sdk.AccAddressFromBech32(p.RunAs)
	if err != nil {
		return errorsmod.Wrap(err, "run as address")
	}
	data, err := k.Execute(ctx, contractAddr, runAsAddr, p.Msg, p.Funds)
	if err != nil {
		return err
	}

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeGovContractResult,
		sdk.NewAttribute(types.AttributeKeyResultDataHex, hex.EncodeToString(data)),
	))
	return nil
}

//nolint:staticcheck
func handleUpdateAdminProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.UpdateAdminProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}
	contractAddr, err := sdk.AccAddressFromBech32(p.Contract)
	if err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	newAdminAddr, err := sdk.AccAddressFromBech32(p.NewAdmin)
	if err != nil {
		return errorsmod.Wrap(err, "run as address")
	}

	return k.UpdateContractAdmin(ctx, contractAddr, nil, newAdminAddr)
}

//nolint:staticcheck
func handleClearAdminProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.ClearAdminProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}

	contractAddr, err := sdk.AccAddressFromBech32(p.Contract)
	if err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	err = k.ClearContractAdmin(ctx, contractAddr, nil)
	return err
}

//nolint:staticcheck
func handlePinCodesProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.PinCodesProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}
	for _, v := range p.CodeIDs {
		if err := k.PinCode(ctx, v); err != nil {
			return errorsmod.Wrapf(err, "code id: %d", v)
		}
	}
	return nil
}

//nolint:staticcheck
func handleUnpinCodesProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.UnpinCodesProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}
	for _, v := range p.CodeIDs {
		if err := k.UnpinCode(ctx, v); err != nil {
			return errorsmod.Wrapf(err, "code id: %d", v)
		}
	}
	return nil
}

//nolint:staticcheck
func handleUpdateInstantiateConfigProposal(ctx sdk.Context, k types.ContractOpsKeeper, p types.UpdateInstantiateConfigProposal) error {
	if err := p.ValidateBasic(); err != nil {
		return err
	}

	var emptyCaller sdk.AccAddress
	for _, accessConfigUpdate := range p.AccessConfigUpdates {
		if err := k.SetAccessConfig(ctx, accessConfigUpdate.CodeID, emptyCaller, accessConfigUpdate.InstantiatePermission); err != nil {
			return errorsmod.Wrapf(err, "code id: %d", accessConfigUpdate.CodeID)
		}
	}
	return nil
}
