package keeper

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/indexes"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// AccountKeeperI is the interface contract that x/auth's keeper implements.
type AccountKeeperI interface {
	// Return a new account with the next account number and the specified address. Does not save the new account to the store.
	NewAccountWithAddress(context.Context, sdk.AccAddress) sdk.AccountI

	// Return a new account with the next account number. Does not save the new account to the store.
	NewAccount(context.Context, sdk.AccountI) sdk.AccountI

	// Check if an account exists in the store.
	HasAccount(context.Context, sdk.AccAddress) bool

	// Retrieve an account from the store.
	GetAccount(context.Context, sdk.AccAddress) sdk.AccountI

	// Set an account in the store.
	SetAccount(context.Context, sdk.AccountI)

	// Remove an account from the store.
	RemoveAccount(context.Context, sdk.AccountI)

	// Fetch the public key of an account at a specified address
	GetPubKey(context.Context, sdk.AccAddress) (cryptotypes.PubKey, error)

	// Fetch the sequence of an account at a specified address.
	GetSequence(context.Context, sdk.AccAddress) (uint64, error)

	// Fetch the next account number, and increment the internal counter.
	//
	// Deprecated: keep this to avoid breaking api
	NextAccountNumber(context.Context) uint64

	// GetModulePermissions fetches per-module account permissions
	GetModulePermissions() map[string]types.PermissionsForAddress

	// AddressCodec returns the account address codec.
	AddressCodec() address.Codec
}

func NewAccountIndexes(sb *collections.SchemaBuilder) AccountsIndexes {
	return AccountsIndexes{
		Number: indexes.NewUnique(
			sb, types.AccountNumberStoreKeyPrefix, "account_by_number", collections.Uint64Key, sdk.AccAddressKey,
			func(_ sdk.AccAddress, v sdk.AccountI) (uint64, error) {
				return v.GetAccountNumber(), nil
			},
		),
	}
}

type AccountsIndexes struct {
	// Number is a unique index that indexes accounts by their account number.
	Number *indexes.Unique[uint64, sdk.AccAddress, sdk.AccountI]
}

func (a AccountsIndexes) IndexesList() []collections.Index[sdk.AccAddress, sdk.AccountI] {
	return []collections.Index[sdk.AccAddress, sdk.AccountI]{
		a.Number,
	}
}

// AccountKeeper encodes/decodes accounts using the go-amino (binary)
// encoding/decoding library.
type AccountKeeper struct {
	appmodule.Environment

	addressCodec      address.Codec
	AccountsModKeeper types.AccountsModKeeper

	cdc          codec.BinaryCodec
	permAddrs    map[string]types.PermissionsForAddress
	bech32Prefix string

	// The prototypical AccountI constructor.
	proto func() sdk.AccountI

	// the address capable of executing a MsgUpdateParams message. Typically, this
	// should be the x/gov module account.
	authority string

	// State
	Schema collections.Schema
	Params collections.Item[types.Params]

	// only use for upgrade handler
	//
	// Deprecated: move to accounts module accountNumber
	accountNumber collections.Sequence
	// Accounts key: AccAddr | value: AccountI | index: AccountsIndex
	Accounts *collections.IndexedMap[sdk.AccAddress, sdk.AccountI, AccountsIndexes]
}

var _ AccountKeeperI = &AccountKeeper{}

// NewAccountKeeper returns a new AccountKeeperI that uses go-amino to
// (binary) encode and decode concrete sdk.Accounts.
// `maccPerms` is a map that takes accounts' addresses as keys, and their respective permissions as values. This map is used to construct
// types.PermissionsForAddress and is used in keeper.ValidatePermissions. Permissions are plain strings,
// and don't have to fit into any predefined structure. This auth module does not use account permissions internally, though other modules
// may use auth.Keeper to access the accounts permissions map.
func NewAccountKeeper(
	env appmodule.Environment, cdc codec.BinaryCodec, proto func() sdk.AccountI, accountsModKeeper types.AccountsModKeeper,
	maccPerms map[string][]string, ac address.Codec, bech32Prefix, authority string,
) AccountKeeper {
	permAddrs := make(map[string]types.PermissionsForAddress)
	for name, perms := range maccPerms {
		permAddrs[name] = types.NewPermissionsForAddress(name, perms)
	}

	sb := collections.NewSchemaBuilder(env.KVStoreService)

	ak := AccountKeeper{
		Environment:       env,
		addressCodec:      ac,
		bech32Prefix:      bech32Prefix,
		proto:             proto,
		cdc:               cdc,
		AccountsModKeeper: accountsModKeeper,
		permAddrs:         permAddrs,
		authority:         authority,
		Params:            collections.NewItem(sb, types.ParamsKey, "params", codec.CollValue[types.Params](cdc)),
		accountNumber:     collections.NewSequence(sb, types.GlobalAccountNumberKey, "account_number"),
		Accounts:          collections.NewIndexedMap(sb, types.AddressStoreKeyPrefix, "accounts", sdk.AccAddressKey, codec.CollInterfaceValue[sdk.AccountI](cdc), NewAccountIndexes(sb)),
	}
	schema, err := sb.Build()
	if err != nil {
		panic(err)
	}
	ak.Schema = schema
	return ak
}

// removeLegacyAccountNumberUnsafe is used for migration purpose only. It deletes the sequence in the DB
// and returns the last value used on success.
// Deprecated
func (ak AccountKeeper) removeLegacyAccountNumberUnsafe(ctx context.Context) (uint64, error) {
	accNum, err := ak.accountNumber.Peek(ctx)
	if err != nil {
		return 0, err
	}

	// Delete DB entry for legacy account number
	store := ak.KVStoreService.OpenKVStore(ctx)
	err = store.Delete(types.GlobalAccountNumberKey.Bytes())

	return accNum, err
}

// GetAuthority returns the x/auth module's authority.
func (ak AccountKeeper) GetAuthority() string {
	return ak.authority
}

func (ak AccountKeeper) GetEnvironment() appmodule.Environment {
	return ak.Environment
}

// AddressCodec returns the x/auth account address codec.
// x/auth is tied to bech32 encoded user accounts
func (ak AccountKeeper) AddressCodec() address.Codec {
	return ak.addressCodec
}

// GetPubKey Returns the PubKey of the account at address
func (ak AccountKeeper) GetPubKey(ctx context.Context, addr sdk.AccAddress) (cryptotypes.PubKey, error) {
	acc := ak.GetAccount(ctx, addr)
	if acc == nil {
		return nil, errorsmod.Wrapf(sdkerrors.ErrUnknownAddress, "account %s does not exist", addr)
	}

	return acc.GetPubKey(), nil
}

// GetSequence Returns the Sequence of the account at address
func (ak AccountKeeper) GetSequence(ctx context.Context, addr sdk.AccAddress) (uint64, error) {
	acc := ak.GetAccount(ctx, addr)
	if acc == nil {
		return 0, errorsmod.Wrapf(sdkerrors.ErrUnknownAddress, "account %s does not exist", addr)
	}

	return acc.GetSequence(), nil
}

// NextAccountNumber returns and increments the global account number counter.
// If the global account number is not set, it initializes it with value 0.
//
// Deprecated: NextAccountNumber is deprecated
func (ak AccountKeeper) NextAccountNumber(ctx context.Context) uint64 {
	n, err := ak.AccountsModKeeper.NextAccountNumber(ctx)
	if err != nil {
		panic(err)
	}
	return n
}

// GetModulePermissions fetches per-module account permissions.
func (ak AccountKeeper) GetModulePermissions() map[string]types.PermissionsForAddress {
	return ak.permAddrs
}

// ValidatePermissions validates that the module account has been granted
// permissions within its set of allowed permissions.
func (ak AccountKeeper) ValidatePermissions(macc sdk.ModuleAccountI) error {
	permAddr := ak.permAddrs[macc.GetName()]
	for _, perm := range macc.GetPermissions() {
		if !permAddr.HasPermission(perm) {
			return fmt.Errorf("invalid module permission %s", perm)
		}
	}

	return nil
}

// GetModuleAddress returns an address based on the module name
func (ak AccountKeeper) GetModuleAddress(moduleName string) sdk.AccAddress {
	permAddr, ok := ak.permAddrs[moduleName]
	if !ok {
		return nil
	}

	return permAddr.GetAddress()
}

// GetModuleAddressAndPermissions returns an address and permissions based on the module name
func (ak AccountKeeper) GetModuleAddressAndPermissions(moduleName string) (addr sdk.AccAddress, permissions []string) {
	permAddr, ok := ak.permAddrs[moduleName]
	if !ok {
		return addr, permissions
	}

	return permAddr.GetAddress(), permAddr.GetPermissions()
}

// GetModuleAccountAndPermissions gets the module account from the auth account store and its
// registered permissions
func (ak AccountKeeper) GetModuleAccountAndPermissions(ctx context.Context, moduleName string) (sdk.ModuleAccountI, []string) {
	addr, perms := ak.GetModuleAddressAndPermissions(moduleName)
	if addr == nil {
		return nil, []string{}
	}

	acc := ak.GetAccount(ctx, addr)
	if acc != nil {
		macc, ok := acc.(sdk.ModuleAccountI)
		if !ok {
			panic("account is not a module account")
		}
		return macc, perms
	}

	// create a new module account
	macc := types.NewEmptyModuleAccount(moduleName, perms...)
	maccI := (ak.NewAccount(ctx, macc)).(sdk.ModuleAccountI) // set the account number
	ak.SetModuleAccount(ctx, maccI)

	return maccI, perms
}

// GetModuleAccount gets the module account from the auth account store, if the account does not
// exist in the AccountKeeper, then it is created.
func (ak AccountKeeper) GetModuleAccount(ctx context.Context, moduleName string) sdk.ModuleAccountI {
	acc, _ := ak.GetModuleAccountAndPermissions(ctx, moduleName)
	return acc
}

// SetModuleAccount sets the module account to the auth account store
func (ak AccountKeeper) SetModuleAccount(ctx context.Context, macc sdk.ModuleAccountI) {
	ak.SetAccount(ctx, macc)
}

// add getter for bech32Prefix
func (ak AccountKeeper) getBech32Prefix() (string, error) {
	return ak.bech32Prefix, nil
}

// GetParams gets the auth module's parameters.
func (ak AccountKeeper) GetParams(ctx context.Context) (params types.Params) {
	params, err := ak.Params.Get(ctx)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		panic(err)
	}
	return params
}

func (ak AccountKeeper) NonAtomicMsgsExec(ctx context.Context, signer sdk.AccAddress, msgs []sdk.Msg) ([]*types.NonAtomicExecResult, error) {
	msgResponses := make([]*types.NonAtomicExecResult, 0, len(msgs))

	for _, msg := range msgs {
		if m, ok := msg.(sdk.HasValidateBasic); ok {
			if err := m.ValidateBasic(); err != nil {
				value := &types.NonAtomicExecResult{Error: err.Error()}
				msgResponses = append(msgResponses, value)
				continue
			}
		}

		if err := ak.BranchService.Execute(ctx, func(ctx context.Context) error {
			result, err := ak.AccountsModKeeper.SendModuleMessage(ctx, signer, msg)
			if err != nil {
				// If an error occurs during message execution, append error response
				response := &types.NonAtomicExecResult{Resp: nil, Error: err.Error()}
				msgResponses = append(msgResponses, response)
			} else {
				resp, err := codectypes.NewAnyWithValue(result)
				if err != nil {
					response := &types.NonAtomicExecResult{Resp: nil, Error: err.Error()}
					msgResponses = append(msgResponses, response)
				}
				response := &types.NonAtomicExecResult{Resp: resp, Error: ""}
				msgResponses = append(msgResponses, response)
			}

			return nil
		}); err != nil {
			return nil, err
		}
	}

	return msgResponses, nil
}

// MigrateAccountNumberUnsafe migrates auth's account number to accounts's account number
// and delete store entry for auth legacy GlobalAccountNumberKey.
//
// Should only use in an upgrade handler for migrating account number.
func MigrateAccountNumberUnsafe(ctx context.Context, ak *AccountKeeper) error {
	currentAccNum, err := ak.removeLegacyAccountNumberUnsafe(ctx)
	if err != nil {
		return fmt.Errorf("failed to migrate account number: %w", err)
	}

	err = ak.AccountsModKeeper.InitAccountNumberSeqUnsafe(ctx, currentAccNum)

	return err
}
