package keeper

import (
	"context"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var _ types.MsgServer = msgServer{}

// grpc message server implementation
type msgServer struct {
	keeper *Keeper
}

// NewMsgServerImpl default constructor
func NewMsgServerImpl(k *Keeper) types.MsgServer {
	return &msgServer{keeper: k}
}

// StoreCode stores a new wasm code on chain
func (m msgServer) StoreCode(ctx context.Context, msg *types.MsgStoreCode) (*types.MsgStoreCodeResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}
	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}

	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	codeID, checksum, err := m.keeper.create(ctx, senderAddr, msg.WASMByteCode, msg.InstantiatePermission, policy)
	if err != nil {
		return nil, err
	}

	return &types.MsgStoreCodeResponse{
		CodeID:   codeID,
		Checksum: checksum,
	}, nil
}

// InstantiateContract instantiate a new contract with classic sequence based address generation
func (m msgServer) InstantiateContract(ctx context.Context, msg *types.MsgInstantiateContract) (*types.MsgInstantiateContractResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	var adminAddr sdk.AccAddress
	if msg.Admin != "" {
		if adminAddr, err = sdk.AccAddressFromBech32(msg.Admin); err != nil {
			return nil, errorsmod.Wrap(err, "admin")
		}
	}

	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	contractAddr, data, err := m.keeper.instantiate(ctx, msg.CodeID, senderAddr, adminAddr, msg.Msg, msg.Label, msg.Funds, m.keeper.ClassicAddressGenerator(), policy)
	if err != nil {
		return nil, err
	}

	return &types.MsgInstantiateContractResponse{
		Address: contractAddr.String(),
		Data:    data,
	}, nil
}

// InstantiateContract2 instantiate a new contract with predicatable address generated
func (m msgServer) InstantiateContract2(ctx context.Context, msg *types.MsgInstantiateContract2) (*types.MsgInstantiateContract2Response, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	var adminAddr sdk.AccAddress
	if msg.Admin != "" {
		if adminAddr, err = sdk.AccAddressFromBech32(msg.Admin); err != nil {
			return nil, errorsmod.Wrap(err, "admin")
		}
	}

	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	addrGenerator := PredictableAddressGenerator(senderAddr, msg.Salt, msg.Msg, msg.FixMsg)

	contractAddr, data, err := m.keeper.instantiate(ctx, msg.CodeID, senderAddr, adminAddr, msg.Msg, msg.Label, msg.Funds, addrGenerator, policy)
	if err != nil {
		return nil, err
	}

	return &types.MsgInstantiateContract2Response{
		Address: contractAddr.String(),
		Data:    data,
	}, nil
}

func (m msgServer) ExecuteContract(ctx context.Context, msg *types.MsgExecuteContract) (*types.MsgExecuteContractResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	contractAddr, err := sdk.AccAddressFromBech32(msg.Contract)
	if err != nil {
		return nil, errorsmod.Wrap(err, "contract")
	}

	data, err := m.keeper.execute(ctx, contractAddr, senderAddr, msg.Msg, msg.Funds)
	if err != nil {
		return nil, err
	}

	return &types.MsgExecuteContractResponse{
		Data: data,
	}, nil
}

func (m msgServer) MigrateContract(ctx context.Context, msg *types.MsgMigrateContract) (*types.MsgMigrateContractResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	contractAddr, err := sdk.AccAddressFromBech32(msg.Contract)
	if err != nil {
		return nil, errorsmod.Wrap(err, "contract")
	}

	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	data, err := m.keeper.migrate(ctx, contractAddr, senderAddr, msg.CodeID, msg.Msg, policy)
	if err != nil {
		return nil, err
	}

	return &types.MsgMigrateContractResponse{
		Data: data,
	}, nil
}

func (m msgServer) UpdateAdmin(ctx context.Context, msg *types.MsgUpdateAdmin) (*types.MsgUpdateAdminResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	contractAddr, err := sdk.AccAddressFromBech32(msg.Contract)
	if err != nil {
		return nil, errorsmod.Wrap(err, "contract")
	}
	newAdminAddr, err := sdk.AccAddressFromBech32(msg.NewAdmin)
	if err != nil {
		return nil, errorsmod.Wrap(err, "new admin")
	}

	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	if err := m.keeper.setContractAdmin(ctx, contractAddr, senderAddr, newAdminAddr, policy); err != nil {
		return nil, err
	}

	return &types.MsgUpdateAdminResponse{}, nil
}

func (m msgServer) ClearAdmin(ctx context.Context, msg *types.MsgClearAdmin) (*types.MsgClearAdminResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	contractAddr, err := sdk.AccAddressFromBech32(msg.Contract)
	if err != nil {
		return nil, errorsmod.Wrap(err, "contract")
	}

	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	if err := m.keeper.setContractAdmin(ctx, contractAddr, senderAddr, nil, policy); err != nil {
		return nil, err
	}

	return &types.MsgClearAdminResponse{}, nil
}

func (m msgServer) UpdateInstantiateConfig(ctx context.Context, msg *types.MsgUpdateInstantiateConfig) (*types.MsgUpdateInstantiateConfigResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	if err := m.keeper.setAccessConfig(ctx, msg.CodeID, senderAddr, *msg.NewInstantiatePermission, policy); err != nil {
		return nil, err
	}

	return &types.MsgUpdateInstantiateConfigResponse{}, nil
}

// UpdateParams updates the module parameters
func (m msgServer) UpdateParams(ctx context.Context, req *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if err := req.ValidateBasic(); err != nil {
		return nil, err
	}
	authority := m.keeper.GetAuthority()
	if authority != req.Authority {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "invalid authority; expected %s, got %s", authority, req.Authority)
	}

	if err := m.keeper.SetParams(ctx, req.Params); err != nil {
		return nil, err
	}

	return &types.MsgUpdateParamsResponse{}, nil
}

// PinCodes pins a set of code ids in the wasmvm cache.
func (m msgServer) PinCodes(ctx context.Context, req *types.MsgPinCodes) (*types.MsgPinCodesResponse, error) {
	if err := req.ValidateBasic(); err != nil {
		return nil, err
	}

	authority := m.keeper.GetAuthority()
	if authority != req.Authority {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "invalid authority; expected %s, got %s", authority, req.Authority)
	}

	for _, codeID := range req.CodeIDs {
		if err := m.keeper.pinCode(ctx, codeID); err != nil {
			return nil, err
		}
	}

	return &types.MsgPinCodesResponse{}, nil
}

// UnpinCodes unpins a set of code ids in the wasmvm cache.
func (m msgServer) UnpinCodes(ctx context.Context, req *types.MsgUnpinCodes) (*types.MsgUnpinCodesResponse, error) {
	if err := req.ValidateBasic(); err != nil {
		return nil, err
	}

	authority := m.keeper.GetAuthority()
	if authority != req.Authority {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "invalid authority; expected %s, got %s", authority, req.Authority)
	}

	for _, codeID := range req.CodeIDs {
		if err := m.keeper.unpinCode(ctx, codeID); err != nil {
			return nil, err
		}
	}

	return &types.MsgUnpinCodesResponse{}, nil
}

// SudoContract calls sudo on a contract.
func (m msgServer) SudoContract(ctx context.Context, req *types.MsgSudoContract) (*types.MsgSudoContractResponse, error) {
	if err := req.ValidateBasic(); err != nil {
		return nil, err
	}
	authority := m.keeper.GetAuthority()
	if authority != req.Authority {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "invalid authority; expected %s, got %s", authority, req.Authority)
	}

	contractAddr, err := sdk.AccAddressFromBech32(req.Contract)
	if err != nil {
		return nil, errorsmod.Wrap(err, "contract")
	}

	data, err := m.keeper.Sudo(ctx, contractAddr, req.Msg)
	if err != nil {
		return nil, err
	}

	return &types.MsgSudoContractResponse{Data: data}, nil
}

// StoreAndInstantiateContract stores and instantiates the contract.
func (m msgServer) StoreAndInstantiateContract(goCtx context.Context, req *types.MsgStoreAndInstantiateContract) (*types.MsgStoreAndInstantiateContractResponse, error) {
	if err := req.ValidateBasic(); err != nil {
		return nil, err
	}

	authorityAddr, err := sdk.AccAddressFromBech32(req.Authority)
	if err != nil {
		return nil, errorsmod.Wrap(err, "authority")
	}

	var adminAddr sdk.AccAddress
	if req.Admin != "" {
		if adminAddr, err = sdk.AccAddressFromBech32(req.Admin); err != nil {
			return nil, errorsmod.Wrap(err, "admin")
		}
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	policy := m.selectAuthorizationPolicy(ctx, req.Authority)

	codeID, _, err := m.keeper.create(ctx, authorityAddr, req.WASMByteCode, req.InstantiatePermission, policy)
	if err != nil {
		return nil, err
	}

	contractAddr, data, err := m.keeper.instantiate(ctx, codeID, authorityAddr, adminAddr, req.Msg, req.Label, req.Funds, m.keeper.ClassicAddressGenerator(), policy)
	if err != nil {
		return nil, err
	}

	return &types.MsgStoreAndInstantiateContractResponse{
		Address: contractAddr.String(),
		Data:    data,
	}, nil
}

// AddCodeUploadParamsAddresses adds addresses to code upload params
func (m msgServer) AddCodeUploadParamsAddresses(goCtx context.Context, req *types.MsgAddCodeUploadParamsAddresses) (*types.MsgAddCodeUploadParamsAddressesResponse, error) {
	if err := req.ValidateBasic(); err != nil {
		return nil, err
	}
	authority := m.keeper.GetAuthority()
	if authority != req.Authority {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "invalid authority; expected %s, got %s", authority, req.Authority)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	params := m.keeper.GetParams(ctx)
	if params.CodeUploadAccess.Permission != types.AccessTypeAnyOfAddresses {
		return nil, errorsmod.Wrap(types.ErrInvalid, "permission")
	}

	addresses := params.CodeUploadAccess.Addresses
	for _, newAddr := range req.Addresses {
		if !contains(addresses, newAddr) {
			addresses = append(addresses, newAddr)
		}
	}

	params.CodeUploadAccess.Addresses = addresses
	if err := m.keeper.SetParams(ctx, params); err != nil {
		return nil, err
	}

	return &types.MsgAddCodeUploadParamsAddressesResponse{}, nil
}

// RemoveCodeUploadParamsAddresses removes addresses to code upload params
func (m msgServer) RemoveCodeUploadParamsAddresses(goCtx context.Context, req *types.MsgRemoveCodeUploadParamsAddresses) (*types.MsgRemoveCodeUploadParamsAddressesResponse, error) {
	if err := req.ValidateBasic(); err != nil {
		return nil, err
	}
	authority := m.keeper.GetAuthority()
	if authority != req.Authority {
		return nil, errorsmod.Wrapf(types.ErrInvalid, "invalid authority; expected %s, got %s", authority, req.Authority)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	params := m.keeper.GetParams(ctx)
	if params.CodeUploadAccess.Permission != types.AccessTypeAnyOfAddresses {
		return nil, errorsmod.Wrap(types.ErrInvalid, "permission")
	}
	addresses := params.CodeUploadAccess.Addresses
	newAddresses := make([]string, 0)
	for _, addr := range addresses {
		if contains(req.Addresses, addr) {
			continue
		}
		newAddresses = append(newAddresses, addr)
	}

	params.CodeUploadAccess.Addresses = newAddresses

	if err := m.keeper.SetParams(ctx, params); err != nil {
		return nil, err
	}

	return &types.MsgRemoveCodeUploadParamsAddressesResponse{}, nil
}

func contains[T comparable](src []T, o T) bool {
	for _, v := range src {
		if v == o {
			return true
		}
	}
	return false
}

func (m msgServer) selectAuthorizationPolicy(ctx context.Context, actor string) types.AuthorizationPolicy {
	if actor == m.keeper.GetAuthority() {
		return newGovAuthorizationPolicy(m.keeper.propagateGovAuthorization)
	}
	if policy, ok := types.SubMsgAuthzPolicy(ctx); ok {
		return policy
	}
	return DefaultAuthorizationPolicy{}
}

// StoreAndMigrateContract stores and migrates the contract.
func (m msgServer) StoreAndMigrateContract(goCtx context.Context, req *types.MsgStoreAndMigrateContract) (*types.MsgStoreAndMigrateContractResponse, error) {
	authorityAddr, err := sdk.AccAddressFromBech32(req.Authority)
	if err != nil {
		return nil, errorsmod.Wrap(err, "authority")
	}

	if err = req.ValidateBasic(); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	policy := m.selectAuthorizationPolicy(ctx, req.Authority)

	codeID, checksum, err := m.keeper.create(ctx, authorityAddr, req.WASMByteCode, req.InstantiatePermission, policy)
	if err != nil {
		return nil, err
	}

	contractAddr, err := sdk.AccAddressFromBech32(req.Contract)
	if err != nil {
		return nil, errorsmod.Wrap(err, "contract")
	}

	data, err := m.keeper.migrate(ctx, contractAddr, authorityAddr, codeID, req.Msg, policy)
	if err != nil {
		return nil, err
	}

	return &types.MsgStoreAndMigrateContractResponse{
		CodeID:   codeID,
		Checksum: checksum,
		Data:     data,
	}, nil
}

func (m msgServer) UpdateContractLabel(ctx context.Context, msg *types.MsgUpdateContractLabel) (*types.MsgUpdateContractLabelResponse, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}

	senderAddr, err := sdk.AccAddressFromBech32(msg.Sender)
	if err != nil {
		return nil, errorsmod.Wrap(err, "sender")
	}
	contractAddr, err := sdk.AccAddressFromBech32(msg.Contract)
	if err != nil {
		return nil, errorsmod.Wrap(err, "contract")
	}

	policy := m.selectAuthorizationPolicy(ctx, msg.Sender)

	if err := m.keeper.setContractLabel(ctx, contractAddr, senderAddr, msg.NewLabel, policy); err != nil {
		return nil, err
	}

	return &types.MsgUpdateContractLabelResponse{}, nil
}
