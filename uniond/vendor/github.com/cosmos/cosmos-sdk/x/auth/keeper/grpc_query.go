package keeper

import (
	"context"
	"errors"
	"sort"
	"strings"

	"github.com/cosmos/cosmos-sdk/store/prefix"
	"github.com/cosmos/cosmos-sdk/types/query"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ types.QueryServer = AccountKeeper{}

func (ak AccountKeeper) AccountAddressByID(c context.Context, req *types.QueryAccountAddressByIDRequest) (*types.QueryAccountAddressByIDResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Id != 0 { // ignoring `0` case since it is default value.
		return nil, status.Error(codes.InvalidArgument, "requesting with id isn't supported, try to request using account-id")
	}

	accId := req.AccountId

	ctx := sdk.UnwrapSDKContext(c)
	address := ak.GetAccountAddressByID(ctx, accId)
	if len(address) == 0 {
		return nil, status.Errorf(codes.NotFound, "account address not found with account number %d", req.Id)
	}

	return &types.QueryAccountAddressByIDResponse{AccountAddress: address}, nil
}

func (ak AccountKeeper) Accounts(c context.Context, req *types.QueryAccountsRequest) (*types.QueryAccountsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)
	store := ctx.KVStore(ak.storeKey)
	accountsStore := prefix.NewStore(store, types.AddressStoreKeyPrefix)

	var accounts []*codectypes.Any
	pageRes, err := query.Paginate(accountsStore, req.Pagination, func(key, value []byte) error {
		account := ak.decodeAccount(value)
		any, err := codectypes.NewAnyWithValue(account)
		if err != nil {
			return err
		}

		accounts = append(accounts, any)
		return nil
	})
	if err != nil {
		return nil, status.Errorf(codes.Internal, "paginate: %v", err)
	}

	return &types.QueryAccountsResponse{Accounts: accounts, Pagination: pageRes}, err
}

// Account returns account details based on address
func (ak AccountKeeper) Account(c context.Context, req *types.QueryAccountRequest) (*types.QueryAccountResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Address == "" {
		return nil, status.Error(codes.InvalidArgument, "Address cannot be empty")
	}

	ctx := sdk.UnwrapSDKContext(c)
	addr, err := sdk.AccAddressFromBech32(req.Address)
	if err != nil {
		return nil, err
	}
	account := ak.GetAccount(ctx, addr)
	if account == nil {
		return nil, status.Errorf(codes.NotFound, "account %s not found", req.Address)
	}

	any, err := codectypes.NewAnyWithValue(account)
	if err != nil {
		return nil, status.Errorf(codes.Internal, err.Error())
	}

	return &types.QueryAccountResponse{Account: any}, nil
}

// Params returns parameters of auth module
func (ak AccountKeeper) Params(c context.Context, req *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}
	ctx := sdk.UnwrapSDKContext(c)
	params := ak.GetParams(ctx)

	return &types.QueryParamsResponse{Params: params}, nil
}

// ModuleAccounts returns all the existing Module Accounts
func (ak AccountKeeper) ModuleAccounts(c context.Context, req *types.QueryModuleAccountsRequest) (*types.QueryModuleAccountsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	// For deterministic output, sort the permAddrs by module name.
	sortedPermAddrs := make([]string, 0, len(ak.permAddrs))
	for moduleName := range ak.permAddrs {
		sortedPermAddrs = append(sortedPermAddrs, moduleName)
	}
	sort.Strings(sortedPermAddrs)

	modAccounts := make([]*codectypes.Any, 0, len(ak.permAddrs))

	for _, moduleName := range sortedPermAddrs {
		account := ak.GetModuleAccount(ctx, moduleName)
		if account == nil {
			return nil, status.Errorf(codes.NotFound, "account %s not found", moduleName)
		}
		any, err := codectypes.NewAnyWithValue(account)
		if err != nil {
			return nil, status.Errorf(codes.Internal, err.Error())
		}
		modAccounts = append(modAccounts, any)
	}

	return &types.QueryModuleAccountsResponse{Accounts: modAccounts}, nil
}

// ModuleAccountByName returns module account by module name
func (ak AccountKeeper) ModuleAccountByName(c context.Context, req *types.QueryModuleAccountByNameRequest) (*types.QueryModuleAccountByNameResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if len(req.Name) == 0 {
		return nil, status.Error(codes.InvalidArgument, "module name is empty")
	}

	ctx := sdk.UnwrapSDKContext(c)
	moduleName := req.Name

	account := ak.GetModuleAccount(ctx, moduleName)
	if account == nil {
		return nil, status.Errorf(codes.NotFound, "account %s not found", moduleName)
	}
	any, err := codectypes.NewAnyWithValue(account)
	if err != nil {
		return nil, status.Errorf(codes.Internal, err.Error())
	}

	return &types.QueryModuleAccountByNameResponse{Account: any}, nil
}

// Bech32Prefix returns the keeper internally stored bech32 prefix.
func (ak AccountKeeper) Bech32Prefix(ctx context.Context, req *types.Bech32PrefixRequest) (*types.Bech32PrefixResponse, error) {
	bech32Prefix, err := ak.getBech32Prefix()
	if err != nil {
		return nil, err
	}

	return &types.Bech32PrefixResponse{Bech32Prefix: bech32Prefix}, nil
}

// AddressBytesToString converts an address from bytes to string, using the
// keeper's bech32 prefix.
func (ak AccountKeeper) AddressBytesToString(ctx context.Context, req *types.AddressBytesToStringRequest) (*types.AddressBytesToStringResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if len(req.AddressBytes) == 0 {
		return nil, errors.New("empty address bytes is not allowed")
	}

	text, err := ak.addressCdc.BytesToString(req.AddressBytes)
	if err != nil {
		return nil, err
	}

	return &types.AddressBytesToStringResponse{AddressString: text}, nil
}

// AddressStringToBytes converts an address from string to bytes, using the
// keeper's bech32 prefix.
func (ak AccountKeeper) AddressStringToBytes(ctx context.Context, req *types.AddressStringToBytesRequest) (*types.AddressStringToBytesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if len(strings.TrimSpace(req.AddressString)) == 0 {
		return nil, errors.New("empty address string is not allowed")
	}

	bz, err := ak.addressCdc.StringToBytes(req.AddressString)
	if err != nil {
		return nil, err
	}

	return &types.AddressStringToBytesResponse{AddressBytes: bz}, nil
}

// AccountInfo implements the AccountInfo query.
func (ak AccountKeeper) AccountInfo(goCtx context.Context, req *types.QueryAccountInfoRequest) (*types.QueryAccountInfoResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Address == "" {
		return nil, status.Error(codes.InvalidArgument, "address cannot be empty")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	addr, err := sdk.AccAddressFromBech32(req.Address)
	if err != nil {
		return nil, err
	}

	account := ak.GetAccount(ctx, addr)
	if account == nil {
		return nil, status.Errorf(codes.NotFound, "account %s not found", req.Address)
	}

	// if there is no public key, avoid serializing the nil value
	pubKey := account.GetPubKey()
	var pkAny *codectypes.Any
	if pubKey != nil {
		pkAny, err = codectypes.NewAnyWithValue(account.GetPubKey())
		if err != nil {
			return nil, status.Errorf(codes.Internal, err.Error())
		}
	}

	return &types.QueryAccountInfoResponse{
		Info: &types.BaseAccount{
			Address:       addr.String(),
			PubKey:        pkAny,
			AccountNumber: account.GetAccountNumber(),
			Sequence:      account.GetSequence(),
		},
	}, nil
}
