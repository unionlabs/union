package simulation_test

import (
	"fmt"
	"math/rand"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil/configurator"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	_ "github.com/cosmos/cosmos-sdk/x/auth"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	_ "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
	_ "github.com/cosmos/cosmos-sdk/x/bank"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	"github.com/cosmos/cosmos-sdk/x/bank/testutil"
	_ "github.com/cosmos/cosmos-sdk/x/consensus"
	govcodec "github.com/cosmos/cosmos-sdk/x/gov/codec"
	"github.com/cosmos/cosmos-sdk/x/gov/keeper"
	"github.com/cosmos/cosmos-sdk/x/gov/simulation"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	_ "github.com/cosmos/cosmos-sdk/x/params"
	_ "github.com/cosmos/cosmos-sdk/x/staking"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
)

var (
	_ simtypes.WeightedProposalMsg     = MockWeightedProposals{}
	_ simtypes.WeightedProposalContent = MockWeightedProposals{} //nolint:staticcheck
)

type MockWeightedProposals struct {
	n int
}

func (m MockWeightedProposals) AppParamsKey() string {
	return fmt.Sprintf("AppParamsKey-%d", m.n)
}

func (m MockWeightedProposals) DefaultWeight() int {
	return m.n
}

func (m MockWeightedProposals) MsgSimulatorFn() simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, _ sdk.Context, _ []simtypes.Account) sdk.Msg {
		return nil
	}
}

func (m MockWeightedProposals) ContentSimulatorFn() simtypes.ContentSimulatorFn { //nolint:staticcheck
	return func(r *rand.Rand, _ sdk.Context, _ []simtypes.Account) simtypes.Content { //nolint:staticcheck
		return v1beta1.NewTextProposal(
			fmt.Sprintf("title-%d: %s", m.n, simtypes.RandStringOfLength(r, 100)),
			fmt.Sprintf("description-%d: %s", m.n, simtypes.RandStringOfLength(r, 4000)),
		)
	}
}

func mockWeightedProposalMsg(n int) []simtypes.WeightedProposalMsg {
	wpc := make([]simtypes.WeightedProposalMsg, n)
	for i := 0; i < n; i++ {
		wpc[i] = MockWeightedProposals{i}
	}
	return wpc
}

func mockWeightedLegacyProposalContent(n int) []simtypes.WeightedProposalContent { //nolint:staticcheck
	wpc := make([]simtypes.WeightedProposalContent, n) //nolint:staticcheck
	for i := 0; i < n; i++ {
		wpc[i] = MockWeightedProposals{i}
	}
	return wpc
}

// TestWeightedOperations tests the weights of the operations.
func TestWeightedOperations(t *testing.T) {
	suite, ctx := createTestSuite(t, false)
	app := suite.App
	ctx.WithChainID("test-chain")
	appParams := make(simtypes.AppParams)

	weightesOps := simulation.WeightedOperations(appParams, govcodec.ModuleCdc, suite.AccountKeeper,
		suite.BankKeeper, suite.GovKeeper, mockWeightedProposalMsg(3), mockWeightedLegacyProposalContent(1),
	)

	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accs := getTestingAccounts(t, r, suite.AccountKeeper, suite.BankKeeper, suite.StakingKeeper, ctx, 3)

	expected := []struct {
		weight     int
		opMsgRoute string
		opMsgName  string
	}{
		{simulation.DefaultWeightMsgDeposit, types.ModuleName, simulation.TypeMsgDeposit},
		{simulation.DefaultWeightMsgVote, types.ModuleName, simulation.TypeMsgVote},
		{simulation.DefaultWeightMsgVoteWeighted, types.ModuleName, simulation.TypeMsgVoteWeighted},
		{0, types.ModuleName, simulation.TypeMsgSubmitProposal},
		{1, types.ModuleName, simulation.TypeMsgSubmitProposal},
		{2, types.ModuleName, simulation.TypeMsgSubmitProposal},
		{0, types.ModuleName, simulation.TypeMsgSubmitProposal},
	}

	require.Equal(t, len(weightesOps), len(expected), "number of operations should be the same")
	for i, w := range weightesOps {
		operationMsg, _, err := w.Op()(r, app.BaseApp, ctx, accs, ctx.ChainID())
		require.NoError(t, err)

		// the following checks are very much dependent from the ordering of the output given
		// by WeightedOperations. if the ordering in WeightedOperations changes some tests
		// will fail
		require.Equal(t, expected[i].weight, w.Weight(), "weight should be the same")
		require.Equal(t, expected[i].opMsgRoute, operationMsg.Route, "route should be the same")
		require.Equal(t, expected[i].opMsgName, operationMsg.Name, "operation Msg name should be the same")
	}
}

// TestSimulateMsgSubmitProposal tests the normal scenario of a valid message of type TypeMsgSubmitProposal.
// Abnormal scenarios, where errors occur, are not tested here.
func TestSimulateMsgSubmitProposal(t *testing.T) {
	suite, ctx := createTestSuite(t, false)
	app := suite.App

	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := getTestingAccounts(t, r, suite.AccountKeeper, suite.BankKeeper, suite.StakingKeeper, ctx, 3)

	// begin a new block
	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: app.LastBlockHeight() + 1, AppHash: app.LastCommitID().Hash}})

	// execute operation
	op := simulation.SimulateMsgSubmitProposal(suite.AccountKeeper, suite.BankKeeper, suite.GovKeeper, MockWeightedProposals{3}.MsgSimulatorFn())
	operationMsg, _, err := op(r, app.BaseApp, ctx, accounts, "")
	require.NoError(t, err)

	var msg v1.MsgSubmitProposal
	err = govcodec.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	require.NoError(t, err)

	require.True(t, operationMsg.OK)
	require.Equal(t, "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", msg.Proposer)
	require.NotEqual(t, len(msg.InitialDeposit), 0)
	require.Equal(t, "560969stake", msg.InitialDeposit[0].String())
	require.Equal(t, simulation.TypeMsgSubmitProposal, sdk.MsgTypeURL(&msg))
}

// TestSimulateMsgSubmitProposal tests the normal scenario of a valid message of type TypeMsgSubmitProposal.
// Abnormal scenarios, where errors occur, are not tested here.
func TestSimulateMsgSubmitLegacyProposal(t *testing.T) {
	suite, ctx := createTestSuite(t, false)
	app := suite.App

	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := getTestingAccounts(t, r, suite.AccountKeeper, suite.BankKeeper, suite.StakingKeeper, ctx, 3)

	// begin a new block
	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: app.LastBlockHeight() + 1, AppHash: app.LastCommitID().Hash}})

	// execute operation
	op := simulation.SimulateMsgSubmitLegacyProposal(suite.AccountKeeper, suite.BankKeeper, suite.GovKeeper, MockWeightedProposals{3}.ContentSimulatorFn())
	operationMsg, _, err := op(r, app.BaseApp, ctx, accounts, "")
	require.NoError(t, err)

	var msg v1.MsgSubmitProposal
	err = govcodec.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	require.NoError(t, err)

	require.True(t, operationMsg.OK)
	require.Equal(t, "cosmos1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7u4x9a0", msg.Proposer)
	require.NotEqual(t, len(msg.InitialDeposit), 0)
	require.Equal(t, "2686011stake", msg.InitialDeposit[0].String())
	require.Equal(t, "title-3: ZBSpYuLyYggwexjxusrBqDOTtGTOWeLrQKjLxzIivHSlcxgdXhhuTSkuxKGLwQvuyNhYFmBZHeAerqyNEUzXPFGkqEGqiQWIXnku", msg.Messages[0].GetCachedValue().(*v1.MsgExecLegacyContent).Content.GetCachedValue().(v1beta1.Content).GetTitle())
	require.Equal(t, "description-3: NJWzHdBNpAXKJPHWQdrGYcAHSctgVlqwqHoLfHsXUdStwfefwzqLuKEhmMyYLdbZrcPgYqjNHxPexsruwEGStAneKbWkQDDIlCWBLSiAASNhZqNFlPtfqPJoxKsgMdzjWqLWdqKQuJqWPMvwPQWZUtVMOTMYKJbfdlZsjdsomuScvDmbDkgRualsxDvRJuCAmPOXitIbcyWsKGSdrEunFAOdmXnsuyFVgJqEjbklvmwrUlsxjRSfKZxGcpayDdgoFcnVSutxjRgOSFzPwidAjubMncNweqpbxhXGchpZUxuFDOtpnhNUycJICRYqsPhPSCjPTWZFLkstHWJxvdPEAyEIxXgLwbNOjrgzmaujiBABBIXvcXpLrbcEWNNQsbjvgJFgJkflpRohHUutvnaUqoopuKjTDaemDeSdqbnOzcfJpcTuAQtZoiLZOoAIlboFDAeGmSNwkvObPRvRWQgWkGkxwtPauYgdkmypLjbqhlHJIQTntgWjXwZdOyYEdQRRLfMSdnxqppqUofqLbLQDUjwKVKfZJUJQPsWIPwIVaSTrmKskoAhvmZyJgeRpkaTfGgrJzAigcxtfshmiDCFkuiluqtMOkidknnTBtumyJYlIsWLnCQclqdVmikUoMOPdPWwYbJxXyqUVicNxFxyqJTenNblyyKSdlCbiXxUiYUiMwXZASYfvMDPFgxniSjWaZTjHkqlJvtBsXqwPpyVxnJVGFWhfSxgOcduoxkiopJvFjMmFabrGYeVtTXLhxVUEiGwYUvndjFGzDVntUvibiyZhfMQdMhgsiuysLMiePBNXifRLMsSmXPkwlPloUbJveCvUlaalhZHuvdkCnkSHbMbmOnrfEGPwQiACiPlnihiaOdbjPqPiTXaHDoJXjSlZmltGqNHHNrcKdlFSCdmVOuvDcBLdSklyGJmcLTbSFtALdGlPkqqecJrpLCXNPWefoTJNgEJlyMEPneVaxxduAAEqQpHWZodWyRkDAxzyMnFMcjSVqeRXLqsNyNtQBbuRvunZflWSbbvXXdkyLikYqutQhLPONXbvhcQZJPSWnOulqQaXmbfFxAkqfYeseSHOQidHwbcsOaMnSrrmGjjRmEMQNuknupMxJiIeVjmgZvbmjPIQTEhQFULQLBMPrxcFPvBinaOPYWGvYGRKxLZdwamfRQQFngcdSlvwjfaPbURasIsGJVHtcEAxnIIrhSriiXLOlbEBLXFElXJFGxHJczRBIxAuPKtBisjKBwfzZFagdNmjdwIRvwzLkFKWRTDPxJCmpzHUcrPiiXXHnOIlqNVoGSXZewdnCRhuxeYGPVTfrNTQNOxZmxInOazUYNTNDgzsxlgiVEHPKMfbesvPHUqpNkUqbzeuzfdrsuLDpKHMUbBMKczKKWOdYoIXoPYtEjfOnlQLoGnbQUCuERdEFaptwnsHzTJDsuZkKtzMpFaZobynZdzNydEeJJHDYaQcwUxcqvwfWwNUsCiLvkZQiSfzAHftYgAmVsXgtmcYgTqJIawstRYJrZdSxlfRiqTufgEQVambeZZmaAyRQbcmdjVUZZCgqDrSeltJGXPMgZnGDZqISrGDOClxXCxMjmKqEPwKHoOfOeyGmqWqihqjINXLqnyTesZePQRqaWDQNqpLgNrAUKulklmckTijUltQKuWQDwpLmDyxLppPVMwsmBIpOwQttYFMjgJQZLYFPmxWFLIeZihkRNnkzoypBICIxgEuYsVWGIGRbbxqVasYnstWomJnHwmtOhAFSpttRYYzBmyEtZXiCthvKvWszTXDbiJbGXMcrYpKAgvUVFtdKUfvdMfhAryctklUCEdjetjuGNfJjajZtvzdYaqInKtFPPLYmRaXPdQzxdSQfmZDEVHlHGEGNSPRFJuIfKLLfUmnHxHnRjmzQPNlqrXgifUdzAGKVabYqvcDeYoTYgPsBUqehrBhmQUgTvDnsdpuhUoxskDdppTsYMcnDIPSwKIqhXDCIxOuXrywahvVavvHkPuaenjLmEbMgrkrQLHEAwrhHkPRNvonNQKqprqOFVZKAtpRSpvQUxMoXCMZLSSbnLEFsjVfANdQNQVwTmGxqVjVqRuxREAhuaDrFgEZpYKhwWPEKBevBfsOIcaZKyykQafzmGPLRAKDtTcJxJVgiiuUkmyMYuDUNEUhBEdoBLJnamtLmMJQgmLiUELIhLpiEvpOXOvXCPUeldLFqkKOwfacqIaRcnnZvERKRMCKUkMABbDHytQqQblrvoxOZkwzosQfDKGtIdfcXRJNqlBNwOCWoQBcEWyqrMlYZIAXYJmLfnjoJepgSFvrgajaBAIksoyeHqgqbGvpAstMIGmIhRYGGNPRIfOQKsGoKgxtsidhTaAePRCBFqZgPDWCIkqOJezGVkjfYUCZTlInbxBXwUAVRsxHTQtJFnnpmMvXDYCVlEmnZBKhmmxQOIQzxFWpJQkQoSAYzTEiDWEOsVLNrbfzeHFRyeYATakQQWmFDLPbVMCJcWjFGJjfqCoVzlbNNEsqxdSmNPjTjHYOkuEMFLkXYGaoJlraLqayMeCsTjWNRDPBywBJLAPVkGQqTwApVVwYAetlwSbzsdHWsTwSIcctkyKDuRWYDQikRqsKTMJchrliONJeaZIzwPQrNbTwxsGdwuduvibtYndRwpdsvyCktRHFalvUuEKMqXbItfGcNGWsGzubdPMYayOUOINjpcFBeESdwpdlTYmrPsLsVDhpTzoMegKrytNVZkfJRPuDCUXxSlSthOohmsuxmIZUedzxKmowKOdXTMcEtdpHaPWgIsIjrViKrQOCONlSuazmLuCUjLltOGXeNgJKedTVrrVCpWYWHyVrdXpKgNaMJVjbXxnVMSChdWKuZdqpisvrkBJPoURDYxWOtpjzZoOpWzyUuYNhCzRoHsMjmmWDcXzQiHIyjwdhPNwiPqFxeUfMVFQGImhykFgMIlQEoZCaRoqSBXTSWAeDumdbsOGtATwEdZlLfoBKiTvodQBGOEcuATWXfiinSjPmJKcWgQrTVYVrwlyMWhxqNbCMpIQNoSMGTiWfPTCezUjYcdWppnsYJihLQCqbNLRGgqrwHuIvsazapTpoPZIyZyeeSueJuTIhpHMEJfJpScshJubJGfkusuVBgfTWQoywSSliQQSfbvaHKiLnyjdSbpMkdBgXepoSsHnCQaYuHQqZsoEOmJCiuQUpJkmfyfbIShzlZpHFmLCsbknEAkKXKfRTRnuwdBeuOGgFbJLbDksHVapaRayWzwoYBEpmrlAxrUxYMUekKbpjPNfjUCjhbdMAnJmYQVZBQZkFVweHDAlaqJjRqoQPoOMLhyvYCzqEuQsAFoxWrzRnTVjStPadhsESlERnKhpEPsfDxNvxqcOyIulaCkmPdambLHvGhTZzysvqFauEgkFRItPfvisehFmoBhQqmkfbHVsgfHXDPJVyhwPllQpuYLRYvGodxKjkarnSNgsXoKEMlaSKxKdcVgvOkuLcfLFfdtXGTclqfPOfeoVLbqcjcXCUEBgAGplrkgsmIEhWRZLlGPGCwKWRaCKMkBHTAcypUrYjWwCLtOPVygMwMANGoQwFnCqFrUGMCRZUGJKTZIGPyldsifauoMnJPLTcDHmilcmahlqOELaAUYDBuzsVywnDQfwRLGIWozYaOAilMBcObErwgTDNGWnwQMUgFFSKtPDMEoEQCTKVREqrXZSGLqwTMcxHfWotDllNkIJPMbXzjDVjPOOjCFuIvTyhXKLyhUScOXvYthRXpPfKwMhptXaxIxgqBoUqzrWbaoLTVpQoottZyPFfNOoMioXHRuFwMRYUiKvcWPkrayyTLOCFJlAyslDameIuqVAuxErqFPEWIScKpBORIuZqoXlZuTvAjEdlEWDODFRregDTqGNoFBIHxvimmIZwLfFyKUfEWAnNBdtdzDmTPXtpHRGdIbuucfTjOygZsTxPjfweXhSUkMhPjMaxKlMIJMOXcnQfyzeOcbWwNbeH", msg.Messages[0].GetCachedValue().(*v1.MsgExecLegacyContent).Content.GetCachedValue().(v1beta1.Content).GetDescription())
	require.Equal(t, "gov", msg.Route())
	require.Equal(t, simulation.TypeMsgSubmitProposal, msg.Type())
}

// TestSimulateMsgDeposit tests the normal scenario of a valid message of type TypeMsgDeposit.
// Abnormal scenarios, where errors occur, are not tested here.
func TestSimulateMsgDeposit(t *testing.T) {
	suite, ctx := createTestSuite(t, false)
	app := suite.App
	blockTime := time.Now().UTC()
	ctx = ctx.WithBlockTime(blockTime)

	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := getTestingAccounts(t, r, suite.AccountKeeper, suite.BankKeeper, suite.StakingKeeper, ctx, 3)

	// setup a proposal
	content := v1beta1.NewTextProposal("Test", "description")
	contentMsg, err := v1.NewLegacyContent(content, suite.GovKeeper.GetGovernanceAccount(ctx).GetAddress().String())
	require.NoError(t, err)

	submitTime := ctx.BlockHeader().Time
	depositPeriod := suite.GovKeeper.GetParams(ctx).MaxDepositPeriod

	proposal, err := v1.NewProposal([]sdk.Msg{contentMsg}, 1, submitTime, submitTime.Add(*depositPeriod), "", "text proposal", "description", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
	require.NoError(t, err)

	suite.GovKeeper.SetProposal(ctx, proposal)

	// begin a new block
	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: app.LastBlockHeight() + 1, AppHash: app.LastCommitID().Hash, Time: blockTime}})

	// execute operation
	op := simulation.SimulateMsgDeposit(suite.AccountKeeper, suite.BankKeeper, suite.GovKeeper)
	operationMsg, _, err := op(r, app.BaseApp, ctx, accounts, "")
	require.NoError(t, err)

	var msg v1.MsgDeposit
	err = govcodec.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	require.NoError(t, err)

	require.True(t, operationMsg.OK)
	require.Equal(t, uint64(1), msg.ProposalId)
	require.Equal(t, "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", msg.Depositor)
	require.NotEqual(t, len(msg.Amount), 0)
	require.Equal(t, "560969stake", msg.Amount[0].String())
	require.Equal(t, "gov", msg.Route())
	require.Equal(t, simulation.TypeMsgDeposit, msg.Type())
}

// TestSimulateMsgVote tests the normal scenario of a valid message of type TypeMsgVote.
// Abnormal scenarios, where errors occur, are not tested here.
func TestSimulateMsgVote(t *testing.T) {
	suite, ctx := createTestSuite(t, false)
	app := suite.App
	blockTime := time.Now().UTC()
	ctx = ctx.WithBlockTime(blockTime)

	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := getTestingAccounts(t, r, suite.AccountKeeper, suite.BankKeeper, suite.StakingKeeper, ctx, 3)

	// setup a proposal
	govAcc := suite.GovKeeper.GetGovernanceAccount(ctx).GetAddress().String()
	contentMsg, err := v1.NewLegacyContent(v1beta1.NewTextProposal("Test", "description"), govAcc)
	require.NoError(t, err)

	submitTime := ctx.BlockHeader().Time
	depositPeriod := suite.GovKeeper.GetParams(ctx).MaxDepositPeriod

	proposal, err := v1.NewProposal([]sdk.Msg{contentMsg}, 1, submitTime, submitTime.Add(*depositPeriod), "", "text proposal", "description", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
	require.NoError(t, err)

	suite.GovKeeper.ActivateVotingPeriod(ctx, proposal)

	// begin a new block
	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: app.LastBlockHeight() + 1, AppHash: app.LastCommitID().Hash, Time: blockTime}})

	// execute operation
	op := simulation.SimulateMsgVote(suite.AccountKeeper, suite.BankKeeper, suite.GovKeeper)
	operationMsg, _, err := op(r, app.BaseApp, ctx, accounts, "")
	require.NoError(t, err)

	var msg v1.MsgVote
	govcodec.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(t, operationMsg.OK)
	require.Equal(t, uint64(1), msg.ProposalId)
	require.Equal(t, "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", msg.Voter)
	require.Equal(t, v1.OptionYes, msg.Option)
	require.Equal(t, "gov", msg.Route())
	require.Equal(t, simulation.TypeMsgVote, msg.Type())
}

// TestSimulateMsgVoteWeighted tests the normal scenario of a valid message of type TypeMsgVoteWeighted.
// Abnormal scenarios, where errors occur, are not tested here.
func TestSimulateMsgVoteWeighted(t *testing.T) {
	suite, ctx := createTestSuite(t, false)
	app := suite.App
	blockTime := time.Now().UTC()
	ctx = ctx.WithBlockTime(blockTime)

	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := getTestingAccounts(t, r, suite.AccountKeeper, suite.BankKeeper, suite.StakingKeeper, ctx, 3)

	// setup a proposal
	govAcc := suite.GovKeeper.GetGovernanceAccount(ctx).GetAddress().String()
	contentMsg, err := v1.NewLegacyContent(v1beta1.NewTextProposal("Test", "description"), govAcc)
	require.NoError(t, err)
	submitTime := ctx.BlockHeader().Time
	depositPeriod := suite.GovKeeper.GetParams(ctx).MaxDepositPeriod

	proposal, err := v1.NewProposal([]sdk.Msg{contentMsg}, 1, submitTime, submitTime.Add(*depositPeriod), "", "text proposal", "test", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
	require.NoError(t, err)

	suite.GovKeeper.ActivateVotingPeriod(ctx, proposal)

	// begin a new block
	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: app.LastBlockHeight() + 1, AppHash: app.LastCommitID().Hash, Time: blockTime}})

	// execute operation
	op := simulation.SimulateMsgVoteWeighted(suite.AccountKeeper, suite.BankKeeper, suite.GovKeeper)
	operationMsg, _, err := op(r, app.BaseApp, ctx, accounts, "")
	require.NoError(t, err)

	var msg v1.MsgVoteWeighted
	govcodec.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(t, operationMsg.OK)
	require.Equal(t, uint64(1), msg.ProposalId)
	require.Equal(t, "cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", msg.Voter)
	require.True(t, len(msg.Options) >= 1)
	require.Equal(t, "gov", msg.Route())
	require.Equal(t, simulation.TypeMsgVoteWeighted, msg.Type())
}

type suite struct {
	cdc           codec.Codec
	AccountKeeper authkeeper.AccountKeeper
	BankKeeper    bankkeeper.Keeper
	GovKeeper     *keeper.Keeper
	StakingKeeper *stakingkeeper.Keeper
	App           *runtime.App
}

// returns context and an app with updated mint keeper
func createTestSuite(t *testing.T, isCheckTx bool) (suite, sdk.Context) {
	res := suite{}

	app, err := simtestutil.Setup(configurator.NewAppConfig(
		configurator.AuthModule(),
		configurator.TxModule(),
		configurator.ParamsModule(),
		configurator.BankModule(),
		configurator.StakingModule(),
		configurator.ConsensusModule(),
		configurator.GovModule(),
	), &res.AccountKeeper, &res.BankKeeper, &res.GovKeeper, &res.StakingKeeper, &res.cdc)
	require.NoError(t, err)

	ctx := app.BaseApp.NewContext(isCheckTx, tmproto.Header{})

	res.App = app
	return res, ctx
}

func getTestingAccounts(
	t *testing.T, r *rand.Rand,
	accountKeeper authkeeper.AccountKeeper, bankKeeper bankkeeper.Keeper, stakingKeeper *stakingkeeper.Keeper,
	ctx sdk.Context, n int,
) []simtypes.Account {
	accounts := simtypes.RandomAccounts(r, n)

	initAmt := stakingKeeper.TokensFromConsensusPower(ctx, 200)
	initCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, initAmt))

	// add coins to the accounts
	for _, account := range accounts {
		acc := accountKeeper.NewAccountWithAddress(ctx, account.Address)
		accountKeeper.SetAccount(ctx, acc)
		require.NoError(t, testutil.FundAccount(bankKeeper, ctx, account.Address, initCoins))
	}

	return accounts
}
