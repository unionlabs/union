package types_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	corecontext "cosmossdk.io/core/context"
	coregas "cosmossdk.io/core/gas"
	coreheader "cosmossdk.io/core/header"
	storetypes "cosmossdk.io/store/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	coin100 = sdk.NewInt64Coin("steak", 100)
	coin150 = sdk.NewInt64Coin("steak", 150)
	coin50  = sdk.NewInt64Coin("steak", 50)
	delAddr = sdk.AccAddress("_____delegator _____")
	val1    = sdk.ValAddress("_____validator1_____")
	val2    = sdk.ValAddress("_____validator2_____")
	val3    = sdk.ValAddress("_____validator3_____")
)

func valAddressToString(t *testing.T, addr sdk.ValAddress) string {
	t.Helper()
	r, err := codectestutil.CodecOptions{}.GetValidatorCodec().BytesToString(addr)
	assert.NoError(t, err)
	return r
}

func accAddressToString(t *testing.T, addr sdk.AccAddress) string {
	t.Helper()
	r, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(addr)
	assert.NoError(t, err)
	return r
}

type headerService struct{}

func (h headerService) HeaderInfo(ctx context.Context) coreheader.Info {
	return sdk.UnwrapSDKContext(ctx).HeaderInfo()
}

type mockGasService struct {
	coregas.Service
}

func (m mockGasService) GasMeter(ctx context.Context) coregas.Meter {
	return mockGasMeter{}
}

type mockGasMeter struct {
	coregas.Meter
}

func (m mockGasMeter) Consume(amount coregas.Gas, descriptor string) error {
	return nil
}

func TestAuthzAuthorizations(t *testing.T) {
	key := storetypes.NewKVStoreKey(stakingtypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	sdkCtx := testCtx.Ctx.WithHeaderInfo(coreheader.Info{})
	ctx := context.WithValue(sdkCtx.Context(), corecontext.EnvironmentContextKey, appmodulev2.Environment{
		HeaderService: headerService{},
		GasService:    mockGasService{},
	})

	valAddressCodec := codectestutil.CodecOptions{}.GetValidatorCodec()
	// verify ValidateBasic returns error for the AUTHORIZATION_TYPE_UNSPECIFIED authorization type
	delAuth, err := stakingtypes.NewStakeAuthorization([]sdk.ValAddress{val1, val2}, []sdk.ValAddress{}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNSPECIFIED, &coin100, valAddressCodec)
	require.NoError(t, err)
	require.Error(t, delAuth.ValidateBasic())

	// verify MethodName
	delAuth, err = stakingtypes.NewStakeAuthorization([]sdk.ValAddress{val1, val2}, []sdk.ValAddress{}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, &coin100, valAddressCodec)
	require.NoError(t, err)
	require.Equal(t, delAuth.MsgTypeURL(), sdk.MsgTypeURL(&stakingtypes.MsgDelegate{}))

	// error both allow & deny list
	_, err = stakingtypes.NewStakeAuthorization([]sdk.ValAddress{val1, val2}, []sdk.ValAddress{val1}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, &coin100, valAddressCodec)
	require.Error(t, err)

	// error duplicate allow list
	_, err = stakingtypes.NewStakeAuthorization([]sdk.ValAddress{val1, val1}, []sdk.ValAddress{}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, &coin100, valAddressCodec)
	require.ErrorContains(t, err, "duplicate allowed validator address")

	// error duplicate denied list
	_, err = stakingtypes.NewStakeAuthorization([]sdk.ValAddress{}, []sdk.ValAddress{val1, val1}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE, &coin100, valAddressCodec)
	require.ErrorContains(t, err, "duplicate denied validator address")

	// verify MethodName
	undelAuth, _ := stakingtypes.NewStakeAuthorization([]sdk.ValAddress{val1, val2}, []sdk.ValAddress{}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE, &coin100, valAddressCodec)
	require.Equal(t, undelAuth.MsgTypeURL(), sdk.MsgTypeURL(&stakingtypes.MsgUndelegate{}))

	// verify MethodName
	beginRedelAuth, _ := stakingtypes.NewStakeAuthorization([]sdk.ValAddress{val1, val2}, []sdk.ValAddress{}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE, &coin100, valAddressCodec)
	require.Equal(t, beginRedelAuth.MsgTypeURL(), sdk.MsgTypeURL(&stakingtypes.MsgBeginRedelegate{}))

	// verify MethodName for CancelUnbondingDelegation
	cancelUnbondAuth, _ := stakingtypes.NewStakeAuthorization([]sdk.ValAddress{val1, val2}, []sdk.ValAddress{}, stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION, &coin100, valAddressCodec)
	require.Equal(t, cancelUnbondAuth.MsgTypeURL(), sdk.MsgTypeURL(&stakingtypes.MsgCancelUnbondingDelegation{}))

	validators1_2 := []string{valAddressToString(t, val1), valAddressToString(t, val2)}

	testCases := []struct {
		msg                  string
		allowed              []sdk.ValAddress
		denied               []sdk.ValAddress
		msgType              stakingtypes.AuthorizationType
		limit                *sdk.Coin
		srvMsg               sdk.Msg
		expectErr            bool
		isDelete             bool
		updatedAuthorization *stakingtypes.StakeAuthorization
	}{
		{
			"delegate: expect 0 remaining coins",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			&coin100,
			stakingtypes.NewMsgDelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin100),
			false,
			true,
			nil,
		},
		{
			"delegate: coins more than allowed",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			&coin100,
			stakingtypes.NewMsgDelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin150),
			true,
			false,
			nil,
		},
		{
			"delegate: verify remaining coins",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			&coin100,
			stakingtypes.NewMsgDelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin50),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: validators1_2},
				}, MaxTokens: &coin50, AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			},
		},
		{
			"delegate: testing with invalid validator",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			&coin100,
			stakingtypes.NewMsgDelegate(accAddressToString(t, delAddr), valAddressToString(t, val3), coin100),
			true,
			false,
			nil,
		},
		{
			"delegate: testing delegate without spent limit",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			nil,
			stakingtypes.NewMsgDelegate(accAddressToString(t, delAddr), valAddressToString(t, val2), coin100),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: validators1_2},
				}, MaxTokens: nil, AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			},
		},
		{
			"delegate: fail validator denied",
			[]sdk.ValAddress{},
			[]sdk.ValAddress{val1},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			nil,
			stakingtypes.NewMsgDelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin100),
			true,
			false,
			nil,
		},
		{
			"delegate: testing with a validator out of denylist",
			[]sdk.ValAddress{},
			[]sdk.ValAddress{val1},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			nil,
			stakingtypes.NewMsgDelegate(accAddressToString(t, delAddr), valAddressToString(t, val2), coin100),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_DenyList{
					DenyList: &stakingtypes.StakeAuthorization_Validators{Address: []string{valAddressToString(t, val1)}},
				}, MaxTokens: nil, AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_DELEGATE,
			},
		},
		{
			"undelegate: expect 0 remaining coins",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE,
			&coin100,
			stakingtypes.NewMsgUndelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin100),
			false,
			true,
			nil,
		},
		{
			"undelegate: verify remaining coins",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE,
			&coin100,
			stakingtypes.NewMsgUndelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin50),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: validators1_2},
				}, MaxTokens: &coin50, AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE,
			},
		},
		{
			"undelegate: testing with invalid validator",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE,
			&coin100,
			stakingtypes.NewMsgUndelegate(accAddressToString(t, delAddr), valAddressToString(t, val3), coin100),
			true,
			false,
			nil,
		},
		{
			"undelegate: testing delegate without spent limit",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE,
			nil,
			stakingtypes.NewMsgUndelegate(accAddressToString(t, delAddr), valAddressToString(t, val2), coin100),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: validators1_2},
				}, MaxTokens: nil, AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE,
			},
		},
		{
			"undelegate: fail cannot undelegate, permission denied",
			[]sdk.ValAddress{},
			[]sdk.ValAddress{val1},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_UNDELEGATE,
			&coin100,
			stakingtypes.NewMsgUndelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin100),
			true,
			false,
			nil,
		},

		{
			"redelegate: expect 0 remaining coins",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE,
			&coin100,
			stakingtypes.NewMsgUndelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), coin100),
			false,
			true,
			nil,
		},
		{
			"redelegate: verify remaining coins",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE,
			&coin100,
			stakingtypes.NewMsgBeginRedelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), valAddressToString(t, val1), coin50),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: validators1_2},
				}, MaxTokens: &coin50, AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE,
			},
		},
		{
			"redelegate: testing with invalid validator",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE,
			&coin100,
			stakingtypes.NewMsgBeginRedelegate(accAddressToString(t, delAddr), valAddressToString(t, val3), valAddressToString(t, val3), coin100),
			true,
			false,
			nil,
		},
		{
			"redelegate: testing delegate without spent limit",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE,
			nil,
			stakingtypes.NewMsgBeginRedelegate(accAddressToString(t, delAddr), valAddressToString(t, val2), valAddressToString(t, val2), coin100),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: validators1_2},
				}, MaxTokens: nil, AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE,
			},
		},
		{
			"redelegate: fail cannot undelegate, permission denied",
			[]sdk.ValAddress{},
			[]sdk.ValAddress{val1},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_REDELEGATE,
			&coin100,
			stakingtypes.NewMsgBeginRedelegate(accAddressToString(t, delAddr), valAddressToString(t, val1), valAddressToString(t, val1), coin100),
			true,
			false,
			nil,
		},
		{
			"cancel unbonding delegation: expect 0 remaining coins",
			[]sdk.ValAddress{val1},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION,
			&coin100,
			stakingtypes.NewMsgCancelUnbondingDelegation(accAddressToString(t, delAddr), valAddressToString(t, val1), sdkCtx.HeaderInfo().Height, coin100),
			false,
			true,
			nil,
		},
		{
			"cancel unbonding delegation: verify remaining coins",
			[]sdk.ValAddress{val1},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION,
			&coin100,
			stakingtypes.NewMsgCancelUnbondingDelegation(accAddressToString(t, delAddr), valAddressToString(t, val1), sdkCtx.HeaderInfo().Height, coin50),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: []string{valAddressToString(t, val1)}},
				},
				MaxTokens:         &coin50,
				AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION,
			},
		},
		{
			"cancel unbonding delegation: testing with invalid validator",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION,
			&coin100,
			stakingtypes.NewMsgCancelUnbondingDelegation(accAddressToString(t, delAddr), valAddressToString(t, val3), sdkCtx.HeaderInfo().Height, coin50),
			true,
			false,
			nil,
		},
		{
			"cancel unbonding delegation: testing delegate without spent limit",
			[]sdk.ValAddress{val1, val2},
			[]sdk.ValAddress{},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION,
			nil,
			stakingtypes.NewMsgCancelUnbondingDelegation(accAddressToString(t, delAddr), valAddressToString(t, val2), sdkCtx.HeaderInfo().Height, coin100),
			false,
			false,
			&stakingtypes.StakeAuthorization{
				Validators: &stakingtypes.StakeAuthorization_AllowList{
					AllowList: &stakingtypes.StakeAuthorization_Validators{Address: validators1_2},
				},
				MaxTokens:         nil,
				AuthorizationType: stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION,
			},
		},
		{
			"cancel unbonding delegation: fail cannot undelegate, permission denied",
			[]sdk.ValAddress{},
			[]sdk.ValAddress{val1},
			stakingtypes.AuthorizationType_AUTHORIZATION_TYPE_CANCEL_UNBONDING_DELEGATION,
			&coin100,
			stakingtypes.NewMsgCancelUnbondingDelegation(accAddressToString(t, delAddr), valAddressToString(t, val1), sdkCtx.HeaderInfo().Height, coin100),
			true,
			false,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.msg, func(t *testing.T) {
			delAuth, err := stakingtypes.NewStakeAuthorization(tc.allowed, tc.denied, tc.msgType, tc.limit, valAddressCodec)
			require.NoError(t, err)
			resp, err := delAuth.Accept(ctx, tc.srvMsg)
			require.Equal(t, tc.isDelete, resp.Delete)
			if tc.expectErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				if tc.updatedAuthorization != nil {
					require.Equal(t, tc.updatedAuthorization.String(), resp.Updated.String())
				}
			}
		})
	}
}
