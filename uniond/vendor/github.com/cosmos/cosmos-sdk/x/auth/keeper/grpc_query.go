package keeper

import (
	"context"
	"errors"
	"sort"
	"strings"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ types.QueryServer = queryServer{}

func NewQueryServer(k AccountKeeper) types.QueryServer {
	return queryServer{k: k}
}

type queryServer struct{ k AccountKeeper }

func (s queryServer) AccountAddressByID(ctx context.Context, req *types.QueryAccountAddressByIDRequest) (*types.QueryAccountAddressByIDResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Id != 0 { // ignoring `0` case since it is default value.
		return nil, status.Error(codes.InvalidArgument, "requesting with id isn't supported, try to request using account-id")
	}

	accID := req.AccountId

	address, err := s.k.Accounts.Indexes.Number.MatchExact(ctx, accID)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "account address not found with account number %d", accID)
	}

	addr, err := s.k.addressCodec.BytesToString(address)
	if err != nil {
		return nil, err
	}

	return &types.QueryAccountAddressByIDResponse{AccountAddress: addr}, nil
}

func (s queryServer) Accounts(ctx context.Context, req *types.QueryAccountsRequest) (*types.QueryAccountsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	accounts, pageRes, err := query.CollectionPaginate(
		ctx,
		s.k.Accounts,
		req.Pagination,
		func(_ sdk.AccAddress, value sdk.AccountI) (*codectypes.Any, error) {
			return codectypes.NewAnyWithValue(value)
		},
	)

	return &types.QueryAccountsResponse{Accounts: accounts, Pagination: pageRes}, err
}

// Account returns account details based on address
func (s queryServer) Account(ctx context.Context, req *types.QueryAccountRequest) (*types.QueryAccountResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Address == "" {
		return nil, status.Error(codes.InvalidArgument, "Address cannot be empty")
	}

	addr, err := s.k.addressCodec.StringToBytes(req.Address)
	if err != nil {
		return nil, err
	}
	account := s.k.GetAccount(ctx, addr)
	if account == nil {
		xAccount, err := s.getFromXAccounts(ctx, addr)
		if err != nil {
			return nil, status.Errorf(codes.NotFound, "account %s not found", req.Address)
		}
		return &types.QueryAccountResponse{Account: xAccount.Account}, nil
	}

	any, err := codectypes.NewAnyWithValue(account)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryAccountResponse{Account: any}, nil
}

// Params returns parameters of auth module
func (s queryServer) Params(ctx context.Context, req *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}
	params := s.k.GetParams(ctx)

	return &types.QueryParamsResponse{Params: params}, nil
}

// ModuleAccounts returns all the existing Module Accounts
func (s queryServer) ModuleAccounts(ctx context.Context, req *types.QueryModuleAccountsRequest) (*types.QueryModuleAccountsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	// For deterministic output, sort the permAddrs by module name.
	sortedPermAddrs := make([]string, 0, len(s.k.permAddrs))
	for moduleName := range s.k.permAddrs {
		sortedPermAddrs = append(sortedPermAddrs, moduleName)
	}
	sort.Strings(sortedPermAddrs)

	modAccounts := make([]*codectypes.Any, 0, len(s.k.permAddrs))

	for _, moduleName := range sortedPermAddrs {
		account := s.k.GetModuleAccount(ctx, moduleName)
		if account == nil {
			return nil, status.Errorf(codes.NotFound, "account %s not found", moduleName)
		}
		any, err := codectypes.NewAnyWithValue(account)
		if err != nil {
			return nil, status.Error(codes.Internal, err.Error())
		}
		modAccounts = append(modAccounts, any)
	}

	return &types.QueryModuleAccountsResponse{Accounts: modAccounts}, nil
}

// ModuleAccountByName returns module account by module name
func (s queryServer) ModuleAccountByName(ctx context.Context, req *types.QueryModuleAccountByNameRequest) (*types.QueryModuleAccountByNameResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if len(req.Name) == 0 {
		return nil, status.Error(codes.InvalidArgument, "module name is empty")
	}

	moduleName := req.Name

	account := s.k.GetModuleAccount(ctx, moduleName)
	if account == nil {
		return nil, status.Errorf(codes.NotFound, "account %s not found", moduleName)
	}
	any, err := codectypes.NewAnyWithValue(account)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryModuleAccountByNameResponse{Account: any}, nil
}

// Bech32Prefix returns the keeper internally stored bech32 prefix.
func (s queryServer) Bech32Prefix(ctx context.Context, req *types.Bech32PrefixRequest) (*types.Bech32PrefixResponse, error) {
	bech32Prefix, err := s.k.getBech32Prefix()
	if err != nil {
		return nil, err
	}

	if bech32Prefix == "" {
		return &types.Bech32PrefixResponse{Bech32Prefix: "bech32 is not used on this chain"}, nil
	}

	return &types.Bech32PrefixResponse{Bech32Prefix: bech32Prefix}, nil
}

// AddressBytesToString converts an address from bytes to string, using the
// keeper's bech32 prefix.
func (s queryServer) AddressBytesToString(ctx context.Context, req *types.AddressBytesToStringRequest) (*types.AddressBytesToStringResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if len(req.AddressBytes) == 0 {
		return nil, errors.New("empty address bytes is not allowed")
	}

	text, err := s.k.addressCodec.BytesToString(req.AddressBytes)
	if err != nil {
		return nil, err
	}

	return &types.AddressBytesToStringResponse{AddressString: text}, nil
}

// AddressStringToBytes converts an address from string to bytes, using the
// keeper's bech32 prefix.
func (s queryServer) AddressStringToBytes(ctx context.Context, req *types.AddressStringToBytesRequest) (*types.AddressStringToBytesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if len(strings.TrimSpace(req.AddressString)) == 0 {
		return nil, errors.New("empty address string is not allowed")
	}

	bz, err := s.k.addressCodec.StringToBytes(req.AddressString)
	if err != nil {
		return nil, err
	}

	return &types.AddressStringToBytesResponse{AddressBytes: bz}, nil
}

// AccountInfo implements the AccountInfo query.
func (s queryServer) AccountInfo(ctx context.Context, req *types.QueryAccountInfoRequest) (*types.QueryAccountInfoResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Address == "" {
		return nil, status.Error(codes.InvalidArgument, "address cannot be empty")
	}

	addr, err := s.k.addressCodec.StringToBytes(req.Address)
	if err != nil {
		return nil, err
	}

	account := s.k.GetAccount(ctx, addr)
	if account == nil {
		xAccount, err := s.getFromXAccounts(ctx, addr)
		// account info is nil it means that the account can be encapsulated into a
		// legacy account representation but not a base account one.
		if err != nil || xAccount.Base == nil {
			return nil, status.Errorf(codes.NotFound, "account %s not found", req.Address)
		}
		return &types.QueryAccountInfoResponse{Info: xAccount.Base}, nil
	}

	// if there is no public key, avoid serializing the nil value
	pubKey := account.GetPubKey()
	var pkAny *codectypes.Any
	if pubKey != nil {
		pkAny, err = codectypes.NewAnyWithValue(account.GetPubKey())
		if err != nil {
			return nil, status.Error(codes.Internal, err.Error())
		}
	}

	return &types.QueryAccountInfoResponse{
		Info: &types.BaseAccount{
			Address:       req.Address,
			PubKey:        pkAny,
			AccountNumber: account.GetAccountNumber(),
			Sequence:      account.GetSequence(),
		},
	}, nil
}

var (
	errNotXAccount              = errors.New("not an x/account")
	errInvalidLegacyAccountImpl = errors.New("invalid legacy account implementation")
)

func (s queryServer) getFromXAccounts(ctx context.Context, address []byte) (*types.QueryLegacyAccountResponse, error) {
	if !s.k.AccountsModKeeper.IsAccountsModuleAccount(ctx, address) {
		return nil, errNotXAccount
	}

	// attempt to check if it can be queried for a legacy account representation.
	resp, err := s.k.AccountsModKeeper.Query(ctx, address, &types.QueryLegacyAccount{})
	if err != nil {
		return nil, err
	}

	typedResp, ok := resp.(*types.QueryLegacyAccountResponse)
	if !ok {
		return nil, errInvalidLegacyAccountImpl
	}
	return typedResp, nil
}
