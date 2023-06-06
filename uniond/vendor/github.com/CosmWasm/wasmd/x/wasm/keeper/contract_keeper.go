package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var _ types.ContractOpsKeeper = PermissionedKeeper{}

// decoratedKeeper contains a subset of the wasm keeper that are already or can be guarded by an authorization policy in the future
type decoratedKeeper interface {
	create(ctx sdk.Context, creator sdk.AccAddress, wasmCode []byte, instantiateAccess *types.AccessConfig, authZ AuthorizationPolicy) (codeID uint64, checksum []byte, err error)

	instantiate(
		ctx sdk.Context,
		codeID uint64,
		creator, admin sdk.AccAddress,
		initMsg []byte,
		label string,
		deposit sdk.Coins,
		addressGenerator AddressGenerator,
		authZ AuthorizationPolicy,
	) (sdk.AccAddress, []byte, error)

	migrate(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, newCodeID uint64, msg []byte, authZ AuthorizationPolicy) ([]byte, error)
	setContractAdmin(ctx sdk.Context, contractAddress, caller, newAdmin sdk.AccAddress, authZ AuthorizationPolicy) error
	pinCode(ctx sdk.Context, codeID uint64) error
	unpinCode(ctx sdk.Context, codeID uint64) error
	execute(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, msg []byte, coins sdk.Coins) ([]byte, error)
	Sudo(ctx sdk.Context, contractAddress sdk.AccAddress, msg []byte) ([]byte, error)
	setContractInfoExtension(ctx sdk.Context, contract sdk.AccAddress, extra types.ContractInfoExtension) error
	setAccessConfig(ctx sdk.Context, codeID uint64, caller sdk.AccAddress, newConfig types.AccessConfig, autz AuthorizationPolicy) error
	ClassicAddressGenerator() AddressGenerator
}

type PermissionedKeeper struct {
	authZPolicy AuthorizationPolicy
	nested      decoratedKeeper
}

func NewPermissionedKeeper(nested decoratedKeeper, authZPolicy AuthorizationPolicy) *PermissionedKeeper {
	return &PermissionedKeeper{authZPolicy: authZPolicy, nested: nested}
}

func NewGovPermissionKeeper(nested decoratedKeeper) *PermissionedKeeper {
	return NewPermissionedKeeper(nested, GovAuthorizationPolicy{})
}

func NewDefaultPermissionKeeper(nested decoratedKeeper) *PermissionedKeeper {
	return NewPermissionedKeeper(nested, DefaultAuthorizationPolicy{})
}

func (p PermissionedKeeper) Create(ctx sdk.Context, creator sdk.AccAddress, wasmCode []byte, instantiateAccess *types.AccessConfig) (codeID uint64, checksum []byte, err error) {
	return p.nested.create(ctx, creator, wasmCode, instantiateAccess, p.authZPolicy)
}

// Instantiate creates an instance of a WASM contract using the classic sequence based address generator
func (p PermissionedKeeper) Instantiate(
	ctx sdk.Context,
	codeID uint64,
	creator, admin sdk.AccAddress,
	initMsg []byte,
	label string,
	deposit sdk.Coins,
) (sdk.AccAddress, []byte, error) {
	return p.nested.instantiate(ctx, codeID, creator, admin, initMsg, label, deposit, p.nested.ClassicAddressGenerator(), p.authZPolicy)
}

// Instantiate2 creates an instance of a WASM contract using the predictable address generator
func (p PermissionedKeeper) Instantiate2(
	ctx sdk.Context,
	codeID uint64,
	creator, admin sdk.AccAddress,
	initMsg []byte,
	label string,
	deposit sdk.Coins,
	salt []byte,
	fixMsg bool,
) (sdk.AccAddress, []byte, error) {
	return p.nested.instantiate(
		ctx,
		codeID,
		creator,
		admin,
		initMsg,
		label,
		deposit,
		PredicableAddressGenerator(creator, salt, initMsg, fixMsg),
		p.authZPolicy,
	)
}

func (p PermissionedKeeper) Execute(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, msg []byte, coins sdk.Coins) ([]byte, error) {
	return p.nested.execute(ctx, contractAddress, caller, msg, coins)
}

func (p PermissionedKeeper) Migrate(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, newCodeID uint64, msg []byte) ([]byte, error) {
	return p.nested.migrate(ctx, contractAddress, caller, newCodeID, msg, p.authZPolicy)
}

func (p PermissionedKeeper) Sudo(ctx sdk.Context, contractAddress sdk.AccAddress, msg []byte) ([]byte, error) {
	return p.nested.Sudo(ctx, contractAddress, msg)
}

func (p PermissionedKeeper) UpdateContractAdmin(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, newAdmin sdk.AccAddress) error {
	return p.nested.setContractAdmin(ctx, contractAddress, caller, newAdmin, p.authZPolicy)
}

func (p PermissionedKeeper) ClearContractAdmin(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress) error {
	return p.nested.setContractAdmin(ctx, contractAddress, caller, nil, p.authZPolicy)
}

func (p PermissionedKeeper) PinCode(ctx sdk.Context, codeID uint64) error {
	return p.nested.pinCode(ctx, codeID)
}

func (p PermissionedKeeper) UnpinCode(ctx sdk.Context, codeID uint64) error {
	return p.nested.unpinCode(ctx, codeID)
}

// SetContractInfoExtension updates the extra attributes that can be stored with the contract info
func (p PermissionedKeeper) SetContractInfoExtension(ctx sdk.Context, contract sdk.AccAddress, extra types.ContractInfoExtension) error {
	return p.nested.setContractInfoExtension(ctx, contract, extra)
}

// SetAccessConfig updates the access config of a code id.
func (p PermissionedKeeper) SetAccessConfig(ctx sdk.Context, codeID uint64, caller sdk.AccAddress, newConfig types.AccessConfig) error {
	return p.nested.setAccessConfig(ctx, codeID, caller, newConfig, p.authZPolicy)
}
