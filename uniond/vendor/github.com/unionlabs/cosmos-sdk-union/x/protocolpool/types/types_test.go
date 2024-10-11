package types

import (
	"testing"
	time "time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var hour = time.Hour

func TestRegisterInterfaces(t *testing.T) {
	interfaceRegistry := codectestutil.NewCodecOptionsWithPrefixes("cosmos", "cosmosval").NewInterfaceRegistry()
	RegisterInterfaces(interfaceRegistry)
	require.NoError(t, interfaceRegistry.EnsureRegistered(&MsgFundCommunityPool{}))
	require.NoError(t, interfaceRegistry.EnsureRegistered(&MsgCommunityPoolSpend{}))
	require.NoError(t, interfaceRegistry.EnsureRegistered(&MsgSubmitBudgetProposal{}))
	require.NoError(t, interfaceRegistry.EnsureRegistered(&MsgClaimBudget{}))
	require.NoError(t, interfaceRegistry.EnsureRegistered(&MsgCreateContinuousFund{}))
	require.NoError(t, interfaceRegistry.EnsureRegistered(&MsgCancelContinuousFund{}))
	require.NoError(t, interfaceRegistry.EnsureRegistered(&MsgWithdrawContinuousFund{}))
}

func TestNewMsgFundCommunityPool(t *testing.T) {
	amount := sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(100)))
	depositor := "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4"
	msg := NewMsgFundCommunityPool(amount, depositor)
	require.Equal(t, amount, msg.Amount)
	require.Equal(t, depositor, msg.Depositor)
}

func TestNewMsgCommunityPoolSpend(t *testing.T) {
	amount := sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(100)))
	authority := "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4"
	recipient := "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z5"
	msg := NewMsgCommunityPoolSpend(amount, authority, recipient)
	require.Equal(t, amount, msg.Amount)
	require.Equal(t, authority, msg.Authority)
	require.Equal(t, recipient, msg.Recipient)
}

func TestValidateGenesis(t *testing.T) {
	defaultGenesis := DefaultGenesisState()
	err := ValidateGenesis(defaultGenesis)
	require.NoError(t, err)

	gs := NewGenesisState(
		[]*ContinuousFund{
			{
				Recipient:  "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				Percentage: math.LegacyMustNewDecFromStr("0.1"),
				Expiry:     nil,
			},
		},
		[]*Budget{
			{
				RecipientAddress: "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				ClaimedAmount:    &sdk.Coin{},
				LastClaimedAt:    &time.Time{},
				TranchesLeft:     10,
				BudgetPerTranche: &sdk.Coin{Denom: "stake", Amount: math.NewInt(100)},
				Period:           &hour,
			},
		},
	)

	err = ValidateGenesis(gs)
	require.NoError(t, err)

	gs.Budget[0].RecipientAddress = ""
	err = ValidateGenesis(gs)
	require.EqualError(t, err, "recipient cannot be empty")

	gs.ContinuousFund[0].Recipient = ""
	err = ValidateGenesis(gs)
	require.EqualError(t, err, "recipient cannot be empty")
}

func TestValidateBudget(t *testing.T) {
	testCases := []struct {
		name      string
		budget    Budget
		expErrMsg string
	}{
		{
			"valid budget",
			Budget{
				RecipientAddress: "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				ClaimedAmount:    &sdk.Coin{},
				LastClaimedAt:    &time.Time{},
				TranchesLeft:     10,
				BudgetPerTranche: &sdk.Coin{Denom: "stake", Amount: math.NewInt(100)},
				Period:           &hour,
			},
			"",
		},
		{
			"empty recipient",
			Budget{
				RecipientAddress: "",
			},
			"recipient cannot be empty",
		},
		{
			"zero budget per tranche",
			Budget{
				RecipientAddress: "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				BudgetPerTranche: &sdk.Coin{Denom: "stake", Amount: math.NewInt(0)},
			},
			"budget per tranche cannot be zero",
		},
		{
			"nil budget per tranche",
			Budget{
				RecipientAddress: "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				BudgetPerTranche: nil,
			},
			"budget per tranche cannot be zero",
		},
		{
			"negative budget per tranche",
			Budget{
				RecipientAddress: "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				BudgetPerTranche: &sdk.Coin{Denom: "stake", Amount: math.NewInt(-100)},
			},
			"-100stake: invalid coins",
		},
		{
			"zero tranches left",
			Budget{
				RecipientAddress: "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				TranchesLeft:     0,
				BudgetPerTranche: &sdk.Coin{Denom: "stake", Amount: math.NewInt(100)},
			},
			"invalid budget proposal: tranches must be greater than zero",
		},
		{
			"zero period",
			Budget{
				RecipientAddress: "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				ClaimedAmount:    &sdk.Coin{},
				LastClaimedAt:    &time.Time{},
				TranchesLeft:     10,
				BudgetPerTranche: &sdk.Coin{Denom: "stake", Amount: math.NewInt(100)},
				Period:           nil,
			},
			"invalid budget proposal: period length should be greater than zero",
		},
	}

	for _, tc := range testCases {
		err := validateBudget(tc.budget)
		if tc.expErrMsg == "" {
			require.NoError(t, err)
		} else {
			require.EqualError(t, err, tc.expErrMsg)
		}
	}
}

func TestValidateContinuousFund(t *testing.T) {
	testCases := []struct {
		name      string
		cf        ContinuousFund
		expErrMsg string
	}{
		{
			"valid continuous fund",
			ContinuousFund{
				Recipient:  "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				Percentage: math.LegacyMustNewDecFromStr("0.1"),
				Expiry:     nil,
			},
			"",
		},
		{
			"empty recipient",
			ContinuousFund{
				Recipient: "",
			},
			"recipient cannot be empty",
		},
		{
			"zero percentage",
			ContinuousFund{
				Recipient:  "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				Percentage: math.LegacyZeroDec(),
			},
			"percentage cannot be zero or empty",
		},
		{
			"nil percentage",
			ContinuousFund{
				Recipient:  "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				Percentage: math.LegacyDec{},
			},
			"percentage cannot be zero or empty",
		},
		{
			"negative percentage",
			ContinuousFund{
				Recipient:  "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				Percentage: math.LegacyMustNewDecFromStr("-0.1"),
			},
			"percentage cannot be negative",
		},
		{
			"percentage exceeds 100%",
			ContinuousFund{
				Recipient:  "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2z2l8z4",
				Percentage: math.LegacyMustNewDecFromStr("1.1"),
			},
			"percentage cannot be greater than one",
		},
	}

	for _, tc := range testCases {
		err := validateContinuousFund(tc.cf)
		if tc.expErrMsg == "" {
			require.NoError(t, err)
		} else {
			require.EqualError(t, err, tc.expErrMsg)
		}
	}
}
