package keeper_test

import (
	"strings"
	"time"

	sdkmath "cosmossdk.io/math"
	banktypes "cosmossdk.io/x/bank/types"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/codec/types"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	abc = "abc"
	o1  = "-0.1"
)

var longAddressError = "address max length is 255"

func (suite *KeeperTestSuite) TestMsgSubmitProposal() {
	suite.reset()
	addrs := suite.addrs
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)

	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress())
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	initialDeposit := coins
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	cases := map[string]struct {
		preRun    func() (*v1.MsgSubmitProposal, error)
		expErr    bool
		expErrMsg string
	}{
		"invalid addr": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					"",
					strings.Repeat("1", 100),
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "invalid proposer address",
		},
		"empty msgs and metadata": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					nil,
					initialDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "no messages proposed",
		},
		"empty title": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					"",
					"",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "proposal title cannot be empty",
		},
		"empty description": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					"",
					"Proposal",
					"",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "proposal summary cannot be empty",
		},
		"title != metadata.title": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					"{\"title\":\"Proposal\", \"description\":\"description of proposal\"}",
					"Proposal2",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "metadata title 'Proposal' must equal proposal title 'Proposal2",
		},
		"summary != metadata.summary": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					"{\"title\":\"Proposal\", \"description\":\"description of proposal\"}",
					"Proposal",
					"description",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "metadata summary '' must equal proposal summary 'description'",
		},
		"title too long": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					"Metadata",
					strings.Repeat("1", 256),
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "title too long",
		},
		"metadata too long": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					strings.Repeat("1", 257),
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "metadata too long",
		},
		"summary too long": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					"",
					"Proposal",
					strings.Repeat("1", 10201),
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "summary too long",
		},
		"many signers": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{testdata.NewTestMsg(govAcct, addrs[0])},
					initialDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "expected authority account as only signer for proposal message",
		},
		"signer isn't gov account": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{testdata.NewTestMsg(addrs[0])},
					initialDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "expected authority account as only signer for proposal message",
		},
		"invalid msg handler": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{testdata.NewTestMsg(govAcct)},
					initialDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "proposal message not recognized by router",
		},
		"invalid deposited coin": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					[]sdk.Coin{sdk.NewCoin("invalid", sdkmath.NewInt(100))},
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "deposited 100invalid, but gov accepts only the following denom(s): [stake]: invalid deposit denom",
		},
		"invalid deposited coin (multiple)": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit.Add(sdk.NewCoin("invalid", sdkmath.NewInt(100))),
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr:    true,
			expErrMsg: "deposited 100invalid,100000stake, but gov accepts only the following denom(s): [stake]: invalid deposit denom",
		},
		"all good": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					initialDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr: false,
		},
		"all good with min deposit": {
			preRun: func() (*v1.MsgSubmitProposal, error) {
				return v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
			},
			expErr: false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			msg, err := tc.preRun()
			suite.Require().NoError(err)
			res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
			}
		})
	}
}

// TestSubmitMultipleChoiceProposal tests only multiple choice proposal specific logic.
// Internally the message uses MsgSubmitProposal, which is tested above.
func (suite *KeeperTestSuite) TestSubmitMultipleChoiceProposal() {
	suite.reset()
	addrs := suite.addrs
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	initialDeposit := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))

	cases := map[string]struct {
		preRun    func() (*v1.MsgSubmitMultipleChoiceProposal, error)
		expErr    bool
		expErrMsg string
	}{
		"empty options": {
			preRun: func() (*v1.MsgSubmitMultipleChoiceProposal, error) {
				return v1.NewMultipleChoiceMsgSubmitProposal(
					initialDeposit,
					proposerAddr,
					"mandatory metadata",
					"Proposal",
					"description of proposal",
					&v1.ProposalVoteOptions{},
				)
			},
			expErr:    true,
			expErrMsg: "vote options cannot be empty, two or more options must be provided",
		},
		"invalid options": {
			preRun: func() (*v1.MsgSubmitMultipleChoiceProposal, error) {
				return v1.NewMultipleChoiceMsgSubmitProposal(
					initialDeposit,
					proposerAddr,
					"mandatory metadata",
					"Proposal",
					"description of proposal",
					&v1.ProposalVoteOptions{
						OptionOne:  "Vote for me",
						OptionFour: "Vote for them",
					},
				)
			},
			expErr:    true,
			expErrMsg: "if a vote option is provided, the previous one must also be provided",
		},
		"valid proposal": {
			preRun: func() (*v1.MsgSubmitMultipleChoiceProposal, error) {
				return v1.NewMultipleChoiceMsgSubmitProposal(
					initialDeposit,
					proposerAddr,
					"mandatory metadata",
					"Proposal",
					"description of proposal",
					&v1.ProposalVoteOptions{
						OptionOne: "Vote for me",
						OptionTwo: "Vote for them",
					},
				)
			},
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			msg, err := tc.preRun()
			suite.Require().NoError(err)
			res, err := suite.msgSrvr.SubmitMultipleChoiceProposal(suite.ctx, msg)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgCancelProposal() {
	govAcct := suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress()
	addrs := suite.addrs
	proposer := addrs[0]
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(proposer)
	suite.Require().NoError(err)

	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(govAcct)
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	msg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{bankMsg},
		coins,
		proposerAddr,
		"", "title", "summary",
		v1.ProposalType_PROPOSAL_TYPE_STANDARD,
	)
	suite.Require().NoError(err)

	res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res.ProposalId)
	proposalID := res.ProposalId

	cases := map[string]struct {
		preRun     func() uint64
		expErr     bool
		expErrMsg  string
		proposalID uint64
		depositor  sdk.AccAddress
	}{
		"wrong proposal id": {
			preRun: func() uint64 {
				return 0
			},
			depositor: proposer,
			expErr:    true,
			expErrMsg: "proposal 0 doesn't exist",
		},
		"valid proposal but invalid proposer": {
			preRun: func() uint64 {
				return res.ProposalId
			},
			depositor: addrs[1],
			expErr:    true,
			expErrMsg: "invalid proposer",
		},
		"empty proposer": {
			preRun: func() uint64 {
				return proposalID
			},
			depositor: sdk.AccAddress{},
			expErr:    true,
			expErrMsg: "invalid proposer address: empty address string is not allowed",
		},
		"all good": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					coins,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			depositor: proposer,
			expErr:    false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			proposalID := tc.preRun()
			depositor, err := suite.acctKeeper.AddressCodec().BytesToString(tc.depositor)
			suite.Require().NoError(err)
			cancelProposalReq := v1.NewMsgCancelProposal(proposalID, depositor)
			_, err = suite.msgSrvr.CancelProposal(suite.ctx, cancelProposalReq)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgVote() {
	suite.reset()
	addrs := suite.addrs
	proposer := addrs[0]
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(proposer)
	suite.Require().NoError(err)
	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress())
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	msg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{bankMsg},
		minDeposit,
		proposerAddr,
		"",
		"Proposal",
		"description of proposal",
		v1.ProposalType_PROPOSAL_TYPE_STANDARD,
	)
	suite.Require().NoError(err)

	res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res.ProposalId)
	proposalID := res.ProposalId

	cases := map[string]struct {
		preRun        func() uint64
		expErr        bool
		expErrMsg     string
		expAddrErr    bool
		expAddrErrMsg string
		option        v1.VoteOption
		metadata      string
		voter         sdk.AccAddress
	}{
		"empty voter": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1.VoteOption_VOTE_OPTION_ONE,
			voter:     sdk.AccAddress{},
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid voter address",
		},
		"wrong vote option": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1.VoteOption(0x13),
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid vote option",
		},
		"optimistic proposal: wrong vote option": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:    v1.VoteOption_VOTE_OPTION_ONE,
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "optimistic proposals can only be rejected: invalid vote option",
		},
		"vote on inactive proposal": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					coins,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:    v1.VoteOption_VOTE_OPTION_ONE,
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "inactive proposal",
		},
		"metadata too long": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1.VoteOption_VOTE_OPTION_ONE,
			voter:     proposer,
			metadata:  strings.Repeat("a", 300),
			expErr:    true,
			expErrMsg: "metadata too long",
		},
		"voter error": {
			preRun: func() uint64 {
				return proposalID
			},
			option:        v1.VoteOption_VOTE_OPTION_ONE,
			voter:         sdk.AccAddress(strings.Repeat("a", 300)),
			metadata:      "",
			expErr:        true,
			expErrMsg:     "invalid voter address: empty address string is not allowed",
			expAddrErr:    true,
			expAddrErrMsg: longAddressError,
		},
		"all good": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:   v1.VoteOption_VOTE_OPTION_ONE,
			voter:    proposer,
			metadata: "",
			expErr:   false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			pID := tc.preRun()
			voter, err := suite.acctKeeper.AddressCodec().BytesToString(tc.voter)
			if tc.expAddrErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expAddrErrMsg)
			} else {
				suite.Require().NoError(err)
			}
			voteReq := v1.NewMsgVote(voter, pID, tc.option, tc.metadata)
			_, err = suite.msgSrvr.Vote(suite.ctx, voteReq)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgVoteWeighted() {
	suite.reset()
	govAcct := suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress()
	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(govAcct)
	suite.Require().NoError(err)

	proposer := simtestutil.AddTestAddrsIncremental(suite.bankKeeper, suite.stakingKeeper, suite.ctx, 1, sdkmath.NewInt(50000000))[0]
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(proposer)
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	msg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{bankMsg},
		minDeposit,
		proposerAddr,
		"",
		"Proposal",
		"description of proposal",
		v1.ProposalType_PROPOSAL_TYPE_STANDARD,
	)
	suite.Require().NoError(err)

	res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res.ProposalId)
	proposalID := res.ProposalId

	cases := map[string]struct {
		preRun        func() uint64
		vote          *v1.MsgVote
		expErr        bool
		expErrMsg     string
		expAddrErr    bool
		expAddrErrMsg string
		option        v1.WeightedVoteOptions
		metadata      string
		voter         sdk.AccAddress
	}{
		"empty voter": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1.WeightedVoteOptions{
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDec(1)),
			},
			voter:     sdk.AccAddress{},
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid voter address",
		},
		"weights sum > 1": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1.WeightedVoteOptions{
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDec(1)),
				v1.NewWeightedVoteOption(v1.OptionAbstain, sdkmath.LegacyNewDec(1)),
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "total weight overflow 1.00: invalid vote option",
		},
		"duplicate vote options": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1.WeightedVoteOptions{
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDecWithPrec(5, 1)),
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDecWithPrec(5, 1)),
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "duplicated vote option",
		},
		"zero weight": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1.WeightedVoteOptions{
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDec(0)),
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: `option:VOTE_OPTION_YES weight:"0.000000000000000000" : invalid vote option`,
		},
		"negative weight": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1.WeightedVoteOptions{
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDec(-1)),
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: `option:VOTE_OPTION_YES weight:"-1.000000000000000000" : invalid vote option`,
		},
		"individual weight > 1 but weights sum == 1": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1.WeightedVoteOptions{
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDec(2)),
				v1.NewWeightedVoteOption(v1.OptionNo, sdkmath.LegacyNewDec(-1)),
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: `option:VOTE_OPTION_YES weight:"2.000000000000000000" : invalid vote option`,
		},
		"empty options": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1.WeightedVoteOptions{},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid request",
		},
		"invalid vote option": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1.NewNonSplitVoteOption(v1.VoteOption(0x13)),
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid vote option",
		},
		"optimistic proposal: wrong vote option": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:    v1.NewNonSplitVoteOption(v1.VoteOption_VOTE_OPTION_ONE), // vote yes
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "optimistic proposals can only be rejected: invalid vote option",
		},
		"weights sum < 1": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1.WeightedVoteOptions{ // weight sum <1
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDecWithPrec(5, 1)),
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "total weight lower than 1.00: invalid vote option",
		},
		"vote on inactive proposal": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					coins,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:    v1.NewNonSplitVoteOption(v1.VoteOption_VOTE_OPTION_ONE),
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "inactive proposal",
		},
		"metadata too long": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1.NewNonSplitVoteOption(v1.VoteOption_VOTE_OPTION_ONE),
			voter:     proposer,
			metadata:  strings.Repeat("a", 300),
			expErr:    true,
			expErrMsg: "metadata too long",
		},
		"voter error": {
			preRun: func() uint64 {
				return proposalID
			},
			option:        v1.NewNonSplitVoteOption(v1.VoteOption_VOTE_OPTION_ONE),
			voter:         sdk.AccAddress(strings.Repeat("a", 300)),
			metadata:      "",
			expErr:        true,
			expErrMsg:     "invalid voter address: empty address string is not allowed",
			expAddrErr:    true,
			expAddrErrMsg: longAddressError,
		},
		"all good": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:   v1.NewNonSplitVoteOption(v1.VoteOption_VOTE_OPTION_ONE),
			voter:    proposer,
			metadata: "",
			expErr:   false,
		},
		"all good with split votes": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option: v1.WeightedVoteOptions{
				v1.NewWeightedVoteOption(v1.OptionYes, sdkmath.LegacyNewDecWithPrec(5, 1)),
				v1.NewWeightedVoteOption(v1.OptionAbstain, sdkmath.LegacyNewDecWithPrec(5, 1)),
			},
			voter:    proposer,
			metadata: "",
			expErr:   false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			pID := tc.preRun()
			voter, err := suite.acctKeeper.AddressCodec().BytesToString(tc.voter)
			if tc.expAddrErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expAddrErrMsg)
			} else {
				suite.Require().NoError(err)
			}
			voteReq := v1.NewMsgVoteWeighted(voter, pID, tc.option, tc.metadata)
			_, err = suite.msgSrvr.VoteWeighted(suite.ctx, voteReq)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgDeposit() {
	suite.reset()
	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress())
	suite.Require().NoError(err)
	addrs := suite.addrs
	proposer := addrs[0]
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(proposer)
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := sdk.Coins(params.MinDeposit)
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	msg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{bankMsg},
		coins,
		proposerAddr,
		"",
		"Proposal",
		"description of proposal",
		v1.ProposalType_PROPOSAL_TYPE_STANDARD,
	)
	suite.Require().NoError(err)

	res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res.ProposalId)
	pID := res.ProposalId

	cases := map[string]struct {
		preRun     func() uint64
		expErr     bool
		proposalID uint64
		depositor  sdk.AccAddress
		deposit    sdk.Coins
		expErrMsg  string
	}{
		"wrong proposal id": {
			preRun: func() uint64 {
				return 0
			},
			depositor: proposer,
			deposit:   coins,
			expErr:    true,
			expErrMsg: "not found",
		},
		"empty depositor": {
			preRun: func() uint64 {
				return pID
			},
			depositor: sdk.AccAddress{},
			deposit:   minDeposit,
			expErr:    true,
			expErrMsg: "invalid depositor address",
		},
		"invalid deposited coin ": {
			preRun: func() uint64 {
				return pID
			},
			depositor: proposer,
			deposit:   []sdk.Coin{sdk.NewCoin("ibc/badcoin", sdkmath.NewInt(1000))},
			expErr:    true,
			expErrMsg: "deposited 1000ibc/badcoin, but gov accepts only the following denom(s): [stake]",
		},
		"invalid deposited coin (multiple)": {
			preRun: func() uint64 {
				return pID
			},
			depositor: proposer,
			deposit:   minDeposit.Add(sdk.NewCoin("ibc/badcoin", sdkmath.NewInt(1000))),
			expErr:    true,
			expErrMsg: "deposited 1000ibc/badcoin,10000000stake, but gov accepts only the following denom(s): [stake]: invalid deposit denom",
		},
		"all good": {
			preRun: func() uint64 {
				return pID
			},
			depositor: proposer,
			deposit:   minDeposit,
			expErr:    false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			proposalID := tc.preRun()
			depositor, err := suite.acctKeeper.AddressCodec().BytesToString(tc.depositor)
			suite.Require().NoError(err)
			depositReq := v1.NewMsgDeposit(depositor, proposalID, tc.deposit)
			_, err = suite.msgSrvr.Deposit(suite.ctx, depositReq)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

// legacy msg server tests
func (suite *KeeperTestSuite) TestLegacyMsgSubmitProposal() {
	proposer, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(simtestutil.AddTestAddrsIncremental(suite.bankKeeper, suite.stakingKeeper, suite.ctx, 1, sdkmath.NewInt(50000000))[0])
	suite.Require().NoError(err)
	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	initialDeposit := coins
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	address, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(sdk.AccAddress{})
	suite.Require().NoError(err)
	cases := map[string]struct {
		preRun    func() (*v1beta1.MsgSubmitProposal, error)
		expErr    bool
		expErrMsg string
	}{
		"empty title": {
			preRun: func() (*v1beta1.MsgSubmitProposal, error) {
				content := v1beta1.NewTextProposal("", "I am test")
				return v1beta1.NewMsgSubmitProposal(
					content,
					initialDeposit,
					proposer,
				)
			},
			expErr:    true,
			expErrMsg: "proposal title cannot be blank",
		},
		"empty description": {
			preRun: func() (*v1beta1.MsgSubmitProposal, error) {
				content := v1beta1.NewTextProposal("test", "")
				return v1beta1.NewMsgSubmitProposal(
					content,
					initialDeposit,
					proposer,
				)
			},
			expErr:    true,
			expErrMsg: "proposal description cannot be blank",
		},
		"empty proposer": {
			preRun: func() (*v1beta1.MsgSubmitProposal, error) {
				content := v1beta1.NewTextProposal("test", "I am test")
				return v1beta1.NewMsgSubmitProposal(
					content,
					initialDeposit,
					address,
				)
			},
			expErr:    true,
			expErrMsg: "invalid proposer address: empty address string is not allowed",
		},
		"title text length > max limit allowed": {
			preRun: func() (*v1beta1.MsgSubmitProposal, error) {
				content := v1beta1.NewTextProposal(strings.Repeat("#", v1beta1.MaxTitleLength*2), "I am test")
				return v1beta1.NewMsgSubmitProposal(
					content,
					initialDeposit,
					proposer,
				)
			},
			expErr:    true,
			expErrMsg: "proposal title is longer than max length of 140: invalid proposal content",
		},
		"description text length > max limit allowed": {
			preRun: func() (*v1beta1.MsgSubmitProposal, error) {
				content := v1beta1.NewTextProposal("test", strings.Repeat("#", v1beta1.MaxDescriptionLength*2))
				return v1beta1.NewMsgSubmitProposal(
					content,
					initialDeposit,
					proposer,
				)
			},
			expErr:    true,
			expErrMsg: "proposal description is longer than max length of 10000: invalid proposal content",
		},
		"all good": {
			preRun: func() (*v1beta1.MsgSubmitProposal, error) {
				return v1beta1.NewMsgSubmitProposal(
					v1beta1.NewTextProposal("test", "I am test"),
					initialDeposit,
					proposer,
				)
			},
			expErr: false,
		},
		"all good with min deposit": {
			preRun: func() (*v1beta1.MsgSubmitProposal, error) {
				return v1beta1.NewMsgSubmitProposal(
					v1beta1.NewTextProposal("test", "I am test"),
					minDeposit,
					proposer,
				)
			},
			expErr: false,
		},
	}

	for name, c := range cases {
		suite.Run(name, func() {
			msg, err := c.preRun()
			suite.Require().NoError(err)
			res, err := suite.legacyMsgSrvr.SubmitProposal(suite.ctx, msg)
			if c.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), c.expErrMsg)
			} else {
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyMsgVote() {
	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress())
	suite.Require().NoError(err)
	addrs := suite.addrs
	proposer := addrs[0]
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(proposer)
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	msg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{bankMsg},
		minDeposit,
		proposerAddr,
		"",
		"Proposal",
		"description of proposal",
		v1.ProposalType_PROPOSAL_TYPE_STANDARD,
	)
	suite.Require().NoError(err)

	res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res.ProposalId)
	proposalID := res.ProposalId

	cases := map[string]struct {
		preRun        func() uint64
		expErr        bool
		expAddrErr    bool
		expErrMsg     string
		expAddrErrMsg string
		option        v1beta1.VoteOption
		metadata      string
		voter         sdk.AccAddress
	}{
		"empty voter": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1beta1.OptionYes,
			voter:     sdk.AccAddress{},
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid voter address",
		},
		"wrong vote option": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1beta1.VoteOption(0x13),
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid vote option",
		},
		"vote on inactive proposal": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					coins,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:    v1beta1.OptionYes,
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "inactive proposal",
		},
		"voter error": {
			preRun: func() uint64 {
				return proposalID
			},
			option:        v1beta1.OptionYes,
			voter:         sdk.AccAddress(strings.Repeat("a", 300)),
			metadata:      "",
			expErr:        true,
			expErrMsg:     "invalid voter address: empty address string is not allowed",
			expAddrErr:    true,
			expAddrErrMsg: longAddressError,
		},
		"all good": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option:   v1beta1.OptionYes,
			voter:    proposer,
			metadata: "",
			expErr:   false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			pID := tc.preRun()
			voter, err := suite.acctKeeper.AddressCodec().BytesToString(tc.voter)
			if tc.expAddrErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expAddrErrMsg)
			} else {
				suite.Require().NoError(err)
			}
			voteReq := v1beta1.NewMsgVote(voter, pID, tc.option)
			_, err = suite.legacyMsgSrvr.Vote(suite.ctx, voteReq)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyVoteWeighted() {
	suite.reset()
	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress())
	suite.Require().NoError(err)
	addrs := suite.addrs
	proposer := addrs[0]
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(proposer)
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	msg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{bankMsg},
		minDeposit,
		proposerAddr,
		"",
		"Proposal",
		"description of proposal",
		v1.ProposalType_PROPOSAL_TYPE_STANDARD,
	)
	suite.Require().NoError(err)

	res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res.ProposalId)
	proposalID := res.ProposalId

	cases := map[string]struct {
		preRun        func() uint64
		vote          *v1beta1.MsgVote
		expErr        bool
		expAddrErr    bool
		expErrMsg     string
		expAddrErrMsg string
		option        v1beta1.WeightedVoteOptions
		metadata      string
		voter         sdk.AccAddress
	}{
		"empty voter": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDec(1),
				},
			},
			voter:     sdk.AccAddress{},
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid voter address",
		},
		"weights sum > 1": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDec(1),
				},
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionAbstain,
					Weight: sdkmath.LegacyNewDec(1),
				},
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "total weight overflow 1.00: invalid vote option",
		},
		"duplicate vote options": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDecWithPrec(5, 1),
				},
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDecWithPrec(5, 1),
				},
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "duplicated vote option",
		},
		"zero weight": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDec(0),
				},
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: `option:VOTE_OPTION_YES weight:"0.000000000000000000" : invalid vote option`,
		},
		"negative weight": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDec(-1),
				},
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: `option:VOTE_OPTION_YES weight:"-1.000000000000000000" : invalid vote option`,
		},
		"empty options": {
			preRun: func() uint64 {
				return proposalID
			},
			option:    v1beta1.WeightedVoteOptions{},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid request",
		},
		"invalid vote option": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.VoteOption(0x13),
					Weight: sdkmath.LegacyNewDecWithPrec(5, 1),
				},
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "invalid vote option",
		},
		"weight sum < 1": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDecWithPrec(5, 1),
				},
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "total weight lower than 1.00: invalid vote option",
		},
		"vote on inactive proposal": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					coins,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDec(1),
				},
			},
			voter:     proposer,
			metadata:  "",
			expErr:    true,
			expErrMsg: "inactive proposal",
		},
		"voter error": {
			preRun: func() uint64 {
				return proposalID
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDec(1),
				},
			},
			voter:         sdk.AccAddress(strings.Repeat("a", 300)),
			metadata:      "",
			expErr:        true,
			expErrMsg:     "invalid voter address: empty address string is not allowed",
			expAddrErr:    true,
			expAddrErrMsg: longAddressError,
		},
		"all good": {
			preRun: func() uint64 {
				msg, err := v1.NewMsgSubmitProposal(
					[]sdk.Msg{bankMsg},
					minDeposit,
					proposerAddr,
					"",
					"Proposal",
					"description of proposal",
					v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				)
				suite.Require().NoError(err)

				res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
				suite.Require().NoError(err)
				suite.Require().NotNil(res.ProposalId)
				return res.ProposalId
			},
			option: v1beta1.WeightedVoteOptions{
				v1beta1.WeightedVoteOption{
					Option: v1beta1.OptionYes,
					Weight: sdkmath.LegacyNewDec(1),
				},
			},
			voter:    proposer,
			metadata: "",
			expErr:   false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			pID := tc.preRun()
			voter, err := suite.acctKeeper.AddressCodec().BytesToString(tc.voter)
			if tc.expAddrErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expAddrErrMsg)
			} else {
				suite.Require().NoError(err)
			}
			voteReq := v1beta1.NewMsgVoteWeighted(voter, pID, tc.option)
			_, err = suite.legacyMsgSrvr.VoteWeighted(suite.ctx, voteReq)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyMsgDeposit() {
	govStrAcct, err := suite.acctKeeper.AddressCodec().BytesToString(suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress())
	suite.Require().NoError(err)
	addrs := suite.addrs
	proposer := addrs[0]
	proposerAddr, err := suite.acctKeeper.AddressCodec().BytesToString(proposer)
	suite.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100000)))
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	bankMsg := &banktypes.MsgSend{
		FromAddress: govStrAcct,
		ToAddress:   proposerAddr,
		Amount:      coins,
	}

	msg, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{bankMsg},
		coins,
		proposerAddr,
		"",
		"Proposal",
		"description of proposal",
		v1.ProposalType_PROPOSAL_TYPE_STANDARD,
	)
	suite.Require().NoError(err)

	res, err := suite.msgSrvr.SubmitProposal(suite.ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res.ProposalId)
	pID := res.ProposalId

	cases := map[string]struct {
		preRun     func() uint64
		expErr     bool
		expErrMsg  string
		proposalID uint64
		depositor  sdk.AccAddress
		deposit    sdk.Coins
	}{
		"wrong proposal id": {
			preRun: func() uint64 {
				return 0
			},
			depositor: proposer,
			deposit:   coins,
			expErr:    true,
			expErrMsg: "not found",
		},
		"empty depositor": {
			preRun: func() uint64 {
				return pID
			},
			depositor: sdk.AccAddress{},
			deposit:   coins,
			expErr:    true,
			expErrMsg: "invalid depositor address: empty address string is not allowed",
		},
		"all good": {
			preRun: func() uint64 {
				return pID
			},
			depositor: proposer,
			deposit:   minDeposit,
			expErr:    false,
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			proposalID := tc.preRun()
			depositor, err := suite.acctKeeper.AddressCodec().BytesToString(tc.depositor)
			suite.Require().NoError(err)
			depositReq := v1beta1.NewMsgDeposit(depositor, proposalID, tc.deposit)
			_, err = suite.legacyMsgSrvr.Deposit(suite.ctx, depositReq)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgUpdateParams() {
	authority := suite.govKeeper.GetAuthority()
	params := v1.DefaultParams()
	testCases := []struct {
		name      string
		input     func() *v1.MsgUpdateParams
		expErr    bool
		expErrMsg string
	}{
		{
			name: "valid",
			input: func() *v1.MsgUpdateParams {
				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params,
				}
			},
			expErr: false,
		},
		{
			name: "invalid authority",
			input: func() *v1.MsgUpdateParams {
				return &v1.MsgUpdateParams{
					Authority: "authority",
					Params:    params,
				}
			},
			expErr:    true,
			expErrMsg: "invalid authority",
		},
		{
			name: "invalid min deposit",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.MinDeposit = nil

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "invalid minimum deposit",
		},
		{
			name: "negative deposit",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.MinDeposit = sdk.Coins{{
					Denom:  sdk.DefaultBondDenom,
					Amount: sdkmath.NewInt(-100),
				}}

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "invalid minimum deposit",
		},
		{
			name: "invalid max deposit period",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.MaxDepositPeriod = nil

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "maximum deposit period must not be nil",
		},
		{
			name: "zero max deposit period",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				duration := time.Duration(0)
				params1.MaxDepositPeriod = &duration

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "maximum deposit period must be positive",
		},
		{
			name: "invalid quorum",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.Quorum = abc

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "invalid quorum string",
		},
		{
			name: "negative quorum",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.Quorum = o1

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "quorum cannot be negative",
		},
		{
			name: "quorum > 1",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.Quorum = "2"

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "quorum too large",
		},
		{
			name: "invalid yes quorum",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.YesQuorum = abc

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "invalid yes_quorum string",
		},
		{
			name: "negative yes quorum",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.YesQuorum = o1

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "yes_quorum cannot be negative",
		},
		{
			name: "yes quorum > 1",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.YesQuorum = "2"

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "yes_quorum too large",
		},
		{
			name: "invalid expedited quorum",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.ExpeditedQuorum = abc

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "invalid expedited_quorum string",
		},
		{
			name: "negative expedited quorum",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.ExpeditedQuorum = o1

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "expedited_quorum cannot be negative",
		},
		{
			name: "expedited quorum > 1",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.ExpeditedQuorum = "2"

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "expedited_quorum too large",
		},
		{
			name: "invalid threshold",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.Threshold = abc

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "invalid threshold string",
		},
		{
			name: "negative threshold",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.Threshold = o1

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "vote threshold must be positive",
		},
		{
			name: "threshold > 1",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.Threshold = "2"

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "vote threshold too large",
		},
		{
			name: "invalid veto threshold",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.VetoThreshold = abc

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "invalid vetoThreshold string",
		},
		{
			name: "negative veto threshold",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.VetoThreshold = o1

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "veto threshold must be positive",
		},
		{
			name: "veto threshold > 1",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.VetoThreshold = "2"

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "veto threshold too large",
		},
		{
			name: "invalid voting period",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				params1.VotingPeriod = nil

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "voting period must not be nil",
		},
		{
			name: "zero voting period",
			input: func() *v1.MsgUpdateParams {
				params1 := params
				duration := time.Duration(0)
				params1.VotingPeriod = &duration

				return &v1.MsgUpdateParams{
					Authority: authority,
					Params:    params1,
				}
			},
			expErr:    true,
			expErrMsg: "voting period must be positive",
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			msg := tc.input()
			exec := func(updateParams *v1.MsgUpdateParams) error {
				if _, err := suite.msgSrvr.UpdateParams(suite.ctx, updateParams); err != nil {
					return err
				}
				return nil
			}

			err := exec(msg)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgUpdateMessageParams() {
	testCases := []struct {
		name      string
		input     *v1.MsgUpdateMessageParams
		expErrMsg string
	}{
		{
			name: "invalid authority",
			input: &v1.MsgUpdateMessageParams{
				Authority: "invalid",
				MsgUrl:    "",
				Params:    nil,
			},
			expErrMsg: "invalid authority",
		},
		{
			name: "invalid msg url (valid as not checked in msg handler)",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    "invalid",
				Params:    nil,
			},
			expErrMsg: "",
		},
		{
			name: "empty params (valid = deleting params)",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    "",
				Params:    nil,
			},
			expErrMsg: "",
		},
		{
			name: "invalid quorum",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    sdk.MsgTypeURL(&v1.MsgUpdateParams{}),
				Params: &v1.MessageBasedParams{
					VotingPeriod:  func() *time.Duration { d := time.Hour; return &d }(),
					Quorum:        "-0.334",
					YesQuorum:     "0.5",
					Threshold:     "0.5",
					VetoThreshold: "0.334",
				},
			},
			expErrMsg: "quorum cannot be negative",
		},
		{
			name: "invalid yes quorum",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    sdk.MsgTypeURL(&v1.MsgUpdateParams{}),
				Params: &v1.MessageBasedParams{
					VotingPeriod:  func() *time.Duration { d := time.Hour; return &d }(),
					Quorum:        "0.334",
					YesQuorum:     "-0.5",
					Threshold:     "0.5",
					VetoThreshold: "0.334",
				},
			},
			expErrMsg: "yes_quorum cannot be negative",
		},
		{
			name: "invalid threshold",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    sdk.MsgTypeURL(&v1.MsgUpdateParams{}),
				Params: &v1.MessageBasedParams{
					VotingPeriod:  func() *time.Duration { d := time.Hour; return &d }(),
					Quorum:        "0.334",
					YesQuorum:     "0.5",
					Threshold:     "-0.5",
					VetoThreshold: "0.334",
				},
			},
			expErrMsg: "vote threshold must be positive",
		},
		{
			name: "invalid veto threshold",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    sdk.MsgTypeURL(&v1.MsgUpdateParams{}),
				Params: &v1.MessageBasedParams{
					VotingPeriod:  func() *time.Duration { d := time.Hour; return &d }(),
					Quorum:        "0.334",
					YesQuorum:     "0.5",
					Threshold:     "0.5",
					VetoThreshold: "-0.334",
				},
			},
			expErrMsg: "veto threshold must be positive",
		},
		{
			name: "invalid voting period",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    sdk.MsgTypeURL(&v1.MsgUpdateParams{}),
				Params: &v1.MessageBasedParams{
					VotingPeriod:  func() *time.Duration { d := -time.Hour; return &d }(),
					Quorum:        "0.334",
					YesQuorum:     "0.5",
					Threshold:     "0.5",
					VetoThreshold: "0.334",
				},
			},
			expErrMsg: "voting period must be positive",
		},
		{
			name: "valid",
			input: &v1.MsgUpdateMessageParams{
				Authority: suite.govKeeper.GetAuthority(),
				MsgUrl:    sdk.MsgTypeURL(&v1.MsgUpdateParams{}),
				Params: &v1.MessageBasedParams{
					VotingPeriod:  func() *time.Duration { d := time.Hour; return &d }(),
					Quorum:        "0.334",
					YesQuorum:     "0",
					Threshold:     "0.5",
					VetoThreshold: "0.334",
				},
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			_, err := suite.msgSrvr.UpdateMessageParams(suite.ctx, tc.input)
			if tc.expErrMsg != "" {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestSubmitProposal_InitialDeposit() {
	const meetsDepositValue = baseDepositTestAmount * baseDepositTestPercent / 100
	baseDepositRatioDec := sdkmath.LegacyNewDec(baseDepositTestPercent).Quo(sdkmath.LegacyNewDec(100))

	testcases := map[string]struct {
		minDeposit             sdk.Coins
		minInitialDepositRatio sdkmath.LegacyDec
		initialDeposit         sdk.Coins
		accountBalance         sdk.Coins

		expectError bool
	}{
		"meets initial deposit, enough balance - success": {
			minDeposit:             sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(baseDepositTestAmount))),
			minInitialDepositRatio: baseDepositRatioDec,
			initialDeposit:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue))),
			accountBalance:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue))),
		},
		"does not meet initial deposit, enough balance - error": {
			minDeposit:             sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(baseDepositTestAmount))),
			minInitialDepositRatio: baseDepositRatioDec,
			initialDeposit:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue-1))),
			accountBalance:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue))),

			expectError: true,
		},
		"meets initial deposit, not enough balance - error": {
			minDeposit:             sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(baseDepositTestAmount))),
			minInitialDepositRatio: baseDepositRatioDec,
			initialDeposit:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue))),
			accountBalance:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue-1))),

			expectError: true,
		},
		"does not meet initial deposit and not enough balance - error": {
			minDeposit:             sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(baseDepositTestAmount))),
			minInitialDepositRatio: baseDepositRatioDec,
			initialDeposit:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue-1))),
			accountBalance:         sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(meetsDepositValue-1))),

			expectError: true,
		},
	}

	for name, tc := range testcases {
		suite.Run(name, func() {
			// Setup
			govKeeper, ctx := suite.govKeeper, suite.ctx
			address, err := suite.acctKeeper.AddressCodec().BytesToString(simtestutil.AddTestAddrs(suite.bankKeeper, suite.stakingKeeper, ctx, 1, tc.accountBalance[0].Amount)[0])
			suite.Require().NoError(err)

			params := v1.DefaultParams()
			params.MinDeposit = tc.minDeposit
			params.MinInitialDepositRatio = tc.minInitialDepositRatio.String()
			err = govKeeper.Params.Set(ctx, params)
			suite.Require().NoError(err)

			msg, err := v1.NewMsgSubmitProposal(TestProposal, tc.initialDeposit, address, "test", "Proposal", "description of proposal", v1.ProposalType_PROPOSAL_TYPE_STANDARD)
			suite.Require().NoError(err)

			// System under test
			_, err = suite.msgSrvr.SubmitProposal(ctx, msg)

			// Assertions
			if tc.expectError {
				suite.Require().Error(err)
				return
			}
			suite.Require().NoError(err)
		})
	}
}

func (suite *KeeperTestSuite) TestMsgSudoExec() {
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(suite.addrs[0])
	suite.Require().NoError(err)
	// setup for valid use case
	params, _ := suite.govKeeper.Params.Get(suite.ctx)
	minDeposit := params.MinDeposit
	proposal, err := v1.NewMsgSubmitProposal([]sdk.Msg{}, minDeposit, addr0Str, "{\"title\":\"Proposal\", \"summary\":\"description of proposal\"}", "Proposal", "description of proposal", v1.ProposalType_PROPOSAL_TYPE_STANDARD)
	suite.Require().NoError(err)
	proposalResp, err := suite.msgSrvr.SubmitProposal(suite.ctx, proposal)
	suite.Require().NoError(err)

	// governance makes a random account vote on a proposal
	// normally it isn't possible as governance isn't the signer.
	// governance needs to sudo the vote.
	validMsg := &v1.MsgSudoExec{Authority: suite.govKeeper.GetAuthority()}
	_, err = validMsg.SetSudoedMsg(v1.NewMsgVote(addr0Str, proposalResp.ProposalId, v1.OptionYes, ""))
	suite.Require().NoError(err)

	invalidMsg := &v1.MsgSudoExec{Authority: suite.govKeeper.GetAuthority()}
	_, err = invalidMsg.SetSudoedMsg(&types.Any{TypeUrl: "invalid"})
	suite.Require().NoError(err)

	testCases := []struct {
		name      string
		input     *v1.MsgSudoExec
		expErrMsg string
	}{
		{
			name:      "empty msg",
			input:     nil,
			expErrMsg: "sudo-ed message cannot be nil",
		},
		{
			name: "empty sudoed msg",
			input: &v1.MsgSudoExec{
				Authority: suite.govKeeper.GetAuthority(),
				Msg:       nil,
			},
			expErrMsg: "sudo-ed message cannot be nil",
		},
		{
			name: "invalid authority",
			input: &v1.MsgSudoExec{
				Authority: "invalid",
				Msg:       &types.Any{},
			},
			expErrMsg: "invalid authority",
		},
		{
			name: "invalid msg (not proper sdk message)",
			input: &v1.MsgSudoExec{
				Authority: suite.govKeeper.GetAuthority(),
				Msg:       &types.Any{TypeUrl: "invalid"},
			},
			expErrMsg: "invalid sudo-ed message",
		},
		{
			name:      "invalid msg (not registered)",
			input:     invalidMsg,
			expErrMsg: "unknown message",
		},
		{
			name:  "valid",
			input: validMsg,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			_, err := suite.msgSrvr.SudoExec(suite.ctx, tc.input)
			if tc.expErrMsg != "" {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}
