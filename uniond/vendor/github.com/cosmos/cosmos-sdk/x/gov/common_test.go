package gov_test

import (
	"bytes"
	"log"
	"sort"
	"testing"

	"cosmossdk.io/math"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil/configurator"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	_ "github.com/cosmos/cosmos-sdk/x/auth"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	_ "github.com/cosmos/cosmos-sdk/x/bank"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	_ "github.com/cosmos/cosmos-sdk/x/consensus"
	"github.com/cosmos/cosmos-sdk/x/gov/keeper"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	_ "github.com/cosmos/cosmos-sdk/x/params"
	_ "github.com/cosmos/cosmos-sdk/x/staking"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
	"github.com/stretchr/testify/require"
)

var (
	valTokens           = sdk.TokensFromConsensusPower(42, sdk.DefaultPowerReduction)
	TestProposal        = v1beta1.NewTextProposal("Test", "description")
	TestDescription     = stakingtypes.NewDescription("T", "E", "S", "T", "Z")
	TestCommissionRates = stakingtypes.NewCommissionRates(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyZeroDec())
)

// mkTestLegacyContent creates a MsgExecLegacyContent for testing purposes.
func mkTestLegacyContent(t *testing.T) *v1.MsgExecLegacyContent {
	msgContent, err := v1.NewLegacyContent(TestProposal, authtypes.NewModuleAddress(types.ModuleName).String())
	require.NoError(t, err)

	return msgContent
}

// SortAddresses - Sorts Addresses
func SortAddresses(addrs []sdk.AccAddress) {
	byteAddrs := make([][]byte, len(addrs))

	for i, addr := range addrs {
		byteAddrs[i] = addr.Bytes()
	}

	SortByteArrays(byteAddrs)

	for i, byteAddr := range byteAddrs {
		addrs[i] = byteAddr
	}
}

// implement `Interface` in sort package.
type sortByteArrays [][]byte

func (b sortByteArrays) Len() int {
	return len(b)
}

func (b sortByteArrays) Less(i, j int) bool {
	// bytes package already implements Comparable for []byte.
	switch bytes.Compare(b[i], b[j]) {
	case -1:
		return true
	case 0, 1:
		return false
	default:
		log.Panic("not fail-able with `bytes.Comparable` bounded [-1, 1].")
		return false
	}
}

func (b sortByteArrays) Swap(i, j int) {
	b[j], b[i] = b[i], b[j]
}

// SortByteArrays - sorts the provided byte array
func SortByteArrays(src [][]byte) [][]byte {
	sorted := sortByteArrays(src)
	sort.Sort(sorted)
	return sorted
}

var pubkeys = []cryptotypes.PubKey{
	ed25519.GenPrivKey().PubKey(),
	ed25519.GenPrivKey().PubKey(),
	ed25519.GenPrivKey().PubKey(),
}

type suite struct {
	AccountKeeper authkeeper.AccountKeeper
	BankKeeper    bankkeeper.Keeper
	GovKeeper     *keeper.Keeper
	StakingKeeper *stakingkeeper.Keeper
	App           *runtime.App
}

func createTestSuite(t *testing.T) suite {
	res := suite{}

	app, err := simtestutil.SetupWithConfiguration(
		configurator.NewAppConfig(
			configurator.ParamsModule(),
			configurator.AuthModule(),
			configurator.StakingModule(),
			configurator.BankModule(),
			configurator.GovModule(),
			configurator.ConsensusModule(),
		),
		simtestutil.DefaultStartUpConfig(),
		&res.AccountKeeper, &res.BankKeeper, &res.GovKeeper, &res.StakingKeeper,
	)
	require.NoError(t, err)

	res.App = app
	return res
}
