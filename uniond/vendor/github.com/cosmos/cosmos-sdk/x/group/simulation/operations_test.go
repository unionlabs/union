package simulation_test

import (
	"math/rand"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	"github.com/cosmos/cosmos-sdk/x/bank/testutil"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/group"
	groupcdc "github.com/cosmos/cosmos-sdk/x/group/codec"
	groupkeeper "github.com/cosmos/cosmos-sdk/x/group/keeper"
	"github.com/cosmos/cosmos-sdk/x/group/simulation"
	grouptestutil "github.com/cosmos/cosmos-sdk/x/group/testutil"
)

type SimTestSuite struct {
	suite.Suite

	ctx               sdk.Context
	app               *runtime.App
	codec             codec.Codec
	interfaceRegistry codectypes.InterfaceRegistry
	accountKeeper     authkeeper.AccountKeeper
	bankKeeper        bankkeeper.Keeper
	groupKeeper       groupkeeper.Keeper
}

func (suite *SimTestSuite) SetupTest() {
	app, err := simtestutil.Setup(
		grouptestutil.AppConfig,
		&suite.codec,
		&suite.interfaceRegistry,
		&suite.accountKeeper,
		&suite.bankKeeper,
		&suite.groupKeeper,
	)
	suite.Require().NoError(err)

	suite.app = app
	suite.ctx = app.BaseApp.NewContext(false, tmproto.Header{})
}

func (suite *SimTestSuite) TestWeightedOperations() {
	cdc := suite.codec
	appParams := make(simtypes.AppParams)

	weightedOps := simulation.WeightedOperations(suite.interfaceRegistry, appParams, cdc, suite.accountKeeper,
		suite.bankKeeper, suite.groupKeeper, cdc,
	)

	s := rand.NewSource(2)
	r := rand.New(s)
	accs := suite.getTestingAccounts(r, 3)

	expected := []struct {
		weight     int
		opMsgRoute string
		opMsgName  string
	}{
		{simulation.WeightMsgCreateGroup, group.MsgCreateGroup{}.Route(), simulation.TypeMsgCreateGroup},
		{simulation.WeightMsgCreateGroupPolicy, group.MsgCreateGroupPolicy{}.Route(), simulation.TypeMsgCreateGroupPolicy},
		{simulation.WeightMsgCreateGroupWithPolicy, group.MsgCreateGroupWithPolicy{}.Route(), simulation.TypeMsgCreateGroupWithPolicy},
		{simulation.WeightMsgSubmitProposal, group.MsgSubmitProposal{}.Route(), simulation.TypeMsgSubmitProposal},
		{simulation.WeightMsgSubmitProposal, group.MsgSubmitProposal{}.Route(), simulation.TypeMsgSubmitProposal},
		{simulation.WeightMsgWithdrawProposal, group.MsgWithdrawProposal{}.Route(), simulation.TypeMsgWithdrawProposal},
		{simulation.WeightMsgVote, group.MsgVote{}.Route(), simulation.TypeMsgVote},
		{simulation.WeightMsgExec, group.MsgExec{}.Route(), simulation.TypeMsgExec},
		{simulation.WeightMsgUpdateGroupMetadata, group.MsgUpdateGroupMetadata{}.Route(), simulation.TypeMsgUpdateGroupMetadata},
		{simulation.WeightMsgUpdateGroupAdmin, group.MsgUpdateGroupAdmin{}.Route(), simulation.TypeMsgUpdateGroupAdmin},
		{simulation.WeightMsgUpdateGroupMembers, group.MsgUpdateGroupMembers{}.Route(), simulation.TypeMsgUpdateGroupMembers},
		{simulation.WeightMsgUpdateGroupPolicyAdmin, group.MsgUpdateGroupPolicyAdmin{}.Route(), simulation.TypeMsgUpdateGroupPolicyAdmin},
		{simulation.WeightMsgUpdateGroupPolicyDecisionPolicy, group.MsgUpdateGroupPolicyDecisionPolicy{}.Route(), simulation.TypeMsgUpdateGroupPolicyDecisionPolicy},
		{simulation.WeightMsgUpdateGroupPolicyMetadata, group.MsgUpdateGroupPolicyMetadata{}.Route(), simulation.TypeMsgUpdateGroupPolicyMetadata},
		{simulation.WeightMsgLeaveGroup, group.MsgLeaveGroup{}.Route(), simulation.TypeMsgLeaveGroup},
	}

	for i, w := range weightedOps {
		operationMsg, _, err := w.Op()(r, suite.app.BaseApp, suite.ctx, accs, "")
		suite.Require().NoError(err)

		// the following checks are very much dependent from the ordering of the output given
		// by WeightedOperations. if the ordering in WeightedOperations changes some tests
		// will fail
		suite.Require().Equal(expected[i].weight, w.Weight(), "weight should be the same")
		suite.Require().Equal(expected[i].opMsgRoute, operationMsg.Route, "route should be the same")
		suite.Require().Equal(expected[i].opMsgName, operationMsg.Name, "operation Msg name should be the same")
	}
}

func (suite *SimTestSuite) getTestingAccounts(r *rand.Rand, n int) []simtypes.Account {
	accounts := simtypes.RandomAccounts(r, n)

	initAmt := sdk.TokensFromConsensusPower(200, sdk.DefaultPowerReduction)
	initCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, initAmt))

	// add coins to the accounts
	for _, account := range accounts {
		acc := suite.accountKeeper.NewAccountWithAddress(suite.ctx, account.Address)
		suite.accountKeeper.SetAccount(suite.ctx, acc)
		suite.Require().NoError(testutil.FundAccount(suite.bankKeeper, suite.ctx, account.Address, initCoins))
	}

	return accounts
}

func (suite *SimTestSuite) TestSimulateCreateGroup() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 1)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	acc := accounts[0]

	// execute operation
	op := simulation.SimulateMsgCreateGroup(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgCreateGroup
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(acc.Address.String(), msg.Admin)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateCreateGroupWithPolicy() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 1)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	acc := accounts[0]

	// execute operation
	op := simulation.SimulateMsgCreateGroupWithPolicy(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgCreateGroupWithPolicy
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(acc.Address.String(), msg.Admin)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateCreateGroupPolicy() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 1)
	acc := accounts[0]

	// setup a group
	_, err := suite.groupKeeper.CreateGroup(sdk.WrapSDKContext(suite.ctx),
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgCreateGroupPolicy(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgCreateGroupPolicy
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(acc.Address.String(), msg.Admin)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateSubmitProposal() {
	// setup 1 account
	s := rand.NewSource(2)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 1)
	acc := accounts[0]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:   acc.Address.String(),
		GroupId: groupRes.GroupId,
	}
	err = accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("1", time.Hour, 0))
	suite.Require().NoError(err)
	groupPolicyRes, err := suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgSubmitProposal(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgSubmitProposal
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(groupPolicyRes.Address, msg.GroupPolicyAddress)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestWithdrawProposal() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 3)
	acc := accounts[0]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	addr := acc.Address.String()
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: addr,
			Members: []group.MemberRequest{
				{
					Address: addr,
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:   addr,
		GroupId: groupRes.GroupId,
	}
	err = accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("1", time.Hour, 0))
	suite.Require().NoError(err)
	groupPolicyRes, err := suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	suite.Require().NoError(err)

	// setup a proposal
	proposalReq, err := group.NewMsgSubmitProposal(groupPolicyRes.Address, []string{addr}, []sdk.Msg{
		&banktypes.MsgSend{
			FromAddress: groupPolicyRes.Address,
			ToAddress:   addr,
			Amount:      sdk.Coins{sdk.NewInt64Coin("token", 100)},
		},
	}, "", 0, "MsgSend", "this is a test proposal")
	suite.Require().NoError(err)
	_, err = suite.groupKeeper.SubmitProposal(ctx, proposalReq)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgWithdrawProposal(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgWithdrawProposal
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(addr, msg.Address)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateVote() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 1)
	acc := accounts[0]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	addr := acc.Address.String()
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: addr,
			Members: []group.MemberRequest{
				{
					Address: addr,
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:    addr,
		GroupId:  groupRes.GroupId,
		Metadata: "",
	}
	err = accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("1", time.Hour, 0))
	suite.Require().NoError(err)
	groupPolicyRes, err := suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	suite.Require().NoError(err)

	// setup a proposal
	proposalReq, err := group.NewMsgSubmitProposal(groupPolicyRes.Address, []string{addr}, []sdk.Msg{
		&banktypes.MsgSend{
			FromAddress: groupPolicyRes.Address,
			ToAddress:   addr,
			Amount:      sdk.Coins{sdk.NewInt64Coin("token", 100)},
		},
	}, "", 0, "MsgSend", "this is a test proposal")
	suite.Require().NoError(err)
	_, err = suite.groupKeeper.SubmitProposal(ctx, proposalReq)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgVote(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgVote
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(addr, msg.Voter)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateExec() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 1)
	acc := accounts[0]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	addr := acc.Address.String()
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: addr,
			Members: []group.MemberRequest{
				{
					Address: addr,
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:   addr,
		GroupId: groupRes.GroupId,
	}
	err = accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("1", time.Hour, 0))
	suite.Require().NoError(err)
	groupPolicyRes, err := suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	suite.Require().NoError(err)

	// setup a proposal
	proposalReq, err := group.NewMsgSubmitProposal(groupPolicyRes.Address, []string{addr}, []sdk.Msg{
		&banktypes.MsgSend{
			FromAddress: groupPolicyRes.Address,
			ToAddress:   addr,
			Amount:      sdk.Coins{sdk.NewInt64Coin("token", 100)},
		},
	}, "", 0, "MsgSend", "this is a test proposal")
	suite.Require().NoError(err)
	proposalRes, err := suite.groupKeeper.SubmitProposal(ctx, proposalReq)
	suite.Require().NoError(err)

	// vote
	_, err = suite.groupKeeper.Vote(ctx, &group.MsgVote{
		ProposalId: proposalRes.ProposalId,
		Voter:      addr,
		Option:     group.VOTE_OPTION_YES,
		Exec:       1,
	})
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgExec(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgExec
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(addr, msg.Executor)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateUpdateGroupAdmin() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 2)
	acc := accounts[0]

	// setup a group
	_, err := suite.groupKeeper.CreateGroup(sdk.WrapSDKContext(suite.ctx),
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgUpdateGroupAdmin(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgUpdateGroupAdmin
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(acc.Address.String(), msg.Admin)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateUpdateGroupMetadata() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 2)
	acc := accounts[0]

	// setup a group
	_, err := suite.groupKeeper.CreateGroup(sdk.WrapSDKContext(suite.ctx),
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgUpdateGroupMetadata(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgUpdateGroupMetadata
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(acc.Address.String(), msg.Admin)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateUpdateGroupMembers() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 2)
	acc := accounts[0]

	// setup a group
	_, err := suite.groupKeeper.CreateGroup(sdk.WrapSDKContext(suite.ctx),
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgUpdateGroupMembers(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgUpdateGroupMembers
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(acc.Address.String(), msg.Admin)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateUpdateGroupPolicyAdmin() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 2)
	acc := accounts[0]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:   acc.Address.String(),
		GroupId: groupRes.GroupId,
	}
	err = accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("1", time.Hour, 0))
	suite.Require().NoError(err)
	groupPolicyRes, err := suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgUpdateGroupPolicyAdmin(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgUpdateGroupPolicyAdmin
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(groupPolicyRes.Address, msg.GroupPolicyAddress)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateUpdateGroupPolicyDecisionPolicy() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 2)
	acc := accounts[0]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:   acc.Address.String(),
		GroupId: groupRes.GroupId,
	}
	err = accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("1", time.Hour, 0))
	suite.Require().NoError(err)
	groupPolicyRes, err := suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgUpdateGroupPolicyDecisionPolicy(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgUpdateGroupPolicyDecisionPolicy
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(groupPolicyRes.Address, msg.GroupPolicyAddress)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateUpdateGroupPolicyMetadata() {
	// setup 1 account
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 2)
	acc := accounts[0]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: acc.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: acc.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	suite.Require().NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:   acc.Address.String(),
		GroupId: groupRes.GroupId,
	}
	err = accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("1", time.Hour, 0))
	suite.Require().NoError(err)
	groupPolicyRes, err := suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	suite.Require().NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgUpdateGroupPolicyMetadata(codec.NewProtoCodec(suite.interfaceRegistry), suite.accountKeeper, suite.bankKeeper, suite.groupKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgUpdateGroupPolicyMetadata
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(groupPolicyRes.Address, msg.GroupPolicyAddress)
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateLeaveGroup() {
	s := rand.NewSource(1)
	r := rand.New(s)
	require := suite.Require()

	// setup 4 account
	accounts := suite.getTestingAccounts(r, 4)
	admin := accounts[0]
	member1 := accounts[1]
	member2 := accounts[2]
	member3 := accounts[3]

	// setup a group
	ctx := sdk.WrapSDKContext(suite.ctx)
	groupRes, err := suite.groupKeeper.CreateGroup(ctx,
		&group.MsgCreateGroup{
			Admin: admin.Address.String(),
			Members: []group.MemberRequest{
				{
					Address: member1.Address.String(),
					Weight:  "1",
				},
				{
					Address: member2.Address.String(),
					Weight:  "2",
				},
				{
					Address: member3.Address.String(),
					Weight:  "1",
				},
			},
		},
	)
	require.NoError(err)

	// setup a group account
	accountReq := &group.MsgCreateGroupPolicy{
		Admin:    admin.Address.String(),
		GroupId:  groupRes.GroupId,
		Metadata: "",
	}
	require.NoError(accountReq.SetDecisionPolicy(group.NewThresholdDecisionPolicy("3", time.Hour, time.Hour)))
	_, err = suite.groupKeeper.CreateGroupPolicy(ctx, accountReq)
	require.NoError(err)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height:  suite.app.LastBlockHeight() + 1,
			AppHash: suite.app.LastCommitID().Hash,
		},
	})

	// execute operation
	op := simulation.SimulateMsgLeaveGroup(codec.NewProtoCodec(suite.interfaceRegistry), suite.groupKeeper, suite.accountKeeper, suite.bankKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg group.MsgLeaveGroup
	err = groupcdc.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)
	suite.Require().NoError(err)
	suite.Require().True(operationMsg.OK)
	suite.Require().Equal(groupRes.GroupId, msg.GroupId)
	suite.Require().Len(futureOperations, 0)
}

func TestSimTestSuite(t *testing.T) {
	suite.Run(t, new(SimTestSuite))
}
