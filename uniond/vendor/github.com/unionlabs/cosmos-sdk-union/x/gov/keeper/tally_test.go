package keeper_test

import (
	"context"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/gov/keeper"
	v1 "cosmossdk.io/x/gov/types/v1"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type tallyFixture struct {
	t        *testing.T
	proposal v1.Proposal
	valAddrs []sdk.ValAddress
	delAddrs []sdk.AccAddress
	keeper   *keeper.Keeper
	ctx      sdk.Context
	mocks    mocks
}

var (
	// handy functions
	setTotalBonded = func(s tallyFixture, n int64) {
		s.mocks.stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec("cosmosvaloper")).AnyTimes()
		s.mocks.stakingKeeper.EXPECT().TotalBondedTokens(gomock.Any()).Return(sdkmath.NewInt(n), nil)
	}
	delegatorVote = func(s tallyFixture, voter sdk.AccAddress, delegations []stakingtypes.Delegation, vote v1.VoteOption) {
		err := s.keeper.AddVote(s.ctx, s.proposal.Id, voter, v1.NewNonSplitVoteOption(vote), "")
		require.NoError(s.t, err)
		s.mocks.stakingKeeper.EXPECT().
			IterateDelegations(s.ctx, voter, gomock.Any()).
			DoAndReturn(
				func(ctx context.Context, voter sdk.AccAddress, fn func(index int64, d sdk.DelegationI) bool) error {
					for i, d := range delegations {
						fn(int64(i), d)
					}
					return nil
				})
	}
	validatorVote = func(s tallyFixture, voter sdk.ValAddress, vote v1.VoteOption) {
		// validatorVote is like delegatorVote but without delegations
		delegatorVote(s, sdk.AccAddress(voter), nil, vote)
	}
)

func TestTally_Standard(t *testing.T) {
	tests := []struct {
		name          string
		setup         func(tallyFixture)
		expectedPass  bool
		expectedBurn  bool
		expectedTally v1.TallyResult
		expectedError string
	}{
		{
			name: "no votes, no bonded tokens: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 0)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "no votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one validator votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "1000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "1000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one account votes without delegation: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				delegatorVote(s, s.delAddrs[0], nil, v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "42",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "42",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes yes, validator votes also yes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "1000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes yes, validator votes no: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "42",
				AbstainCount:     "0",
				NoCount:          "999958",
				NoWithVetoCount:  "0",
				OptionOneCount:   "42",
				OptionTwoCount:   "0",
				OptionThreeCount: "999958",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			// one delegator delegates 42 shares to 2 different validators (21 each)
			// delegator votes yes
			// first validator votes yes
			// second validator votes no
			// third validator (no delegation) votes abstain
			name: "delegator with mixed delegations: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				val1Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[1])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{
					{
						DelegatorAddress: del0Addr,
						ValidatorAddress: val0Addr,
						Shares:           sdkmath.LegacyNewDec(21),
					},
					{
						DelegatorAddress: del0Addr,
						ValidatorAddress: val1Addr,
						Shares:           sdkmath.LegacyNewDec(21),
					},
				}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "1000021",
				AbstainCount:     "1000000",
				NoCount:          "999979",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000021",
				OptionTwoCount:   "1000000",
				OptionThreeCount: "999979",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with only abstain: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "4000000",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "4000000",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with veto>1/3: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_FOUR)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_FOUR)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_FOUR)
			},
			expectedPass: false,
			expectedBurn: true,
			expectedTally: v1.TallyResult{
				YesCount:         "4000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "3000000",
				OptionOneCount:   "4000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "3000000",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with yes<=.5: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "2000000",
				AbstainCount:     "0",
				NoCount:          "2000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "2000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "2000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with yes>.5: prop succeeds",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_FOUR)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "4000000",
				AbstainCount:     "0",
				NoCount:          "2000000",
				NoWithVetoCount:  "1000000",
				OptionOneCount:   "4000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "2000000",
				OptionFourCount:  "1000000",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached thanks to abstain, yes>.5: prop succeeds",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "2000000",
				AbstainCount:     "3000000",
				NoCount:          "1000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "2000000",
				OptionTwoCount:   "3000000",
				OptionThreeCount: "1000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with spam > all other votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				// spam votes
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_SPAM)
			},
			expectedPass: false,
			expectedBurn: true,
			expectedTally: v1.TallyResult{
				YesCount:         "1000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "6000000",
			},
		},
		{
			name: "quorum reached, yes quorum not reached: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				params, _ := s.keeper.Params.Get(s.ctx)
				params.YesQuorum = "0.7"
				_ = s.keeper.Params.Set(s.ctx, params)

				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "3000000",
				AbstainCount:     "2000000",
				NoCount:          "1000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "3000000",
				OptionTwoCount:   "2000000",
				OptionThreeCount: "1000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			govKeeper, mocks, _, ctx := setupGovKeeper(t, mockAccountKeeperExpectations)
			params := v1.DefaultParams()
			// Ensure params value are different than false
			params.BurnVoteQuorum = true
			params.BurnVoteVeto = true
			err := govKeeper.Params.Set(ctx, params)
			require.NoError(t, err)
			var (
				numVals       = 10
				numDelegators = 5
				addrs         = simtestutil.CreateRandomAccounts(numVals + numDelegators)
				valAddrs      = simtestutil.ConvertAddrsToValAddrs(addrs[:numVals])
				delAddrs      = addrs[numVals:]
			)
			// Mocks a bunch of validators
			mocks.stakingKeeper.EXPECT().
				IterateBondedValidatorsByPower(ctx, gomock.Any()).
				DoAndReturn(
					func(ctx context.Context, fn func(index int64, validator sdk.ValidatorI) bool) error {
						for i := int64(0); i < int64(numVals); i++ {
							valAddr, err := mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(valAddrs[i])
							require.NoError(t, err)
							fn(i, stakingtypes.Validator{
								OperatorAddress: valAddr,
								Status:          stakingtypes.Bonded,
								Tokens:          sdkmath.NewInt(1000000),
								DelegatorShares: sdkmath.LegacyNewDec(1000000),
							})
						}
						return nil
					})

			// Submit and activate a proposal
			proposal, err := govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", delAddrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
			require.NoError(t, err)
			err = govKeeper.ActivateVotingPeriod(ctx, proposal)
			require.NoError(t, err)
			suite := tallyFixture{
				t:        t,
				proposal: proposal,
				valAddrs: valAddrs,
				delAddrs: delAddrs,
				ctx:      ctx,
				keeper:   govKeeper,
				mocks:    mocks,
			}
			tt.setup(suite)

			pass, burn, tally, err := govKeeper.Tally(ctx, proposal)

			require.NoError(t, err)
			assert.Equal(t, tt.expectedPass, pass, "wrong pass")
			assert.Equal(t, tt.expectedBurn, burn, "wrong burn")
			assert.Equal(t, tt.expectedTally, tally)
			// Assert votes removal after tally
			rng := collections.NewPrefixedPairRange[uint64, sdk.AccAddress](proposal.Id)
			_, err = suite.keeper.Votes.Iterate(suite.ctx, rng)
			assert.NoError(t, err)
		})
	}
}

func TestTally_Expedited(t *testing.T) {
	tests := []struct {
		name          string
		setup         func(tallyFixture)
		expectedPass  bool
		expectedBurn  bool
		expectedTally v1.TallyResult
		expectedError string
	}{
		{
			name: "no votes, no bonded tokens: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 0)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "no votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one validator votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "1000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "1000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one account votes without delegation: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				delegatorVote(s, s.delAddrs[0], nil, v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "42",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "42",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes yes, validator votes also yes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "1000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes yes, validator votes no: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "42",
				AbstainCount:     "0",
				NoCount:          "999958",
				NoWithVetoCount:  "0",
				OptionOneCount:   "42",
				OptionTwoCount:   "0",
				OptionThreeCount: "999958",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			// one delegator delegates 42 shares to 2 different validators (21 each)
			// delegator votes yes
			// first validator votes yes
			// second validator votes no
			// third validator (no delegation) votes abstain
			name: "delegator with mixed delegations: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				val1Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[1])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{
					{
						DelegatorAddress: del0Addr,
						ValidatorAddress: val0Addr,
						Shares:           sdkmath.LegacyNewDec(21),
					},
					{
						DelegatorAddress: del0Addr,
						ValidatorAddress: val1Addr,
						Shares:           sdkmath.LegacyNewDec(21),
					},
				}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "1000021",
				AbstainCount:     "1000000",
				NoCount:          "999979",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000021",
				OptionTwoCount:   "1000000",
				OptionThreeCount: "999979",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with only abstain: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "5000000",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "5000000",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with veto>1/3: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_FOUR)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_FOUR)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_FOUR)
			},
			expectedPass: false,
			expectedBurn: true,
			expectedTally: v1.TallyResult{
				YesCount:         "4000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "3000000",
				OptionOneCount:   "4000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "3000000",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with yes<=.5: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "3000000",
				AbstainCount:     "0",
				NoCount:          "3000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "3000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "3000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with yes<=.667: expedited prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_FOUR)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "4000000",
				AbstainCount:     "0",
				NoCount:          "2000000",
				NoWithVetoCount:  "1000000",
				OptionOneCount:   "4000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "2000000",
				OptionFourCount:  "1000000",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with yes>.667: expedited prop succeeds",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_FOUR)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "5000000",
				AbstainCount:     "0",
				NoCount:          "1000000",
				NoWithVetoCount:  "1000000",
				OptionOneCount:   "5000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "1000000",
				OptionFourCount:  "1000000",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with spam > all other votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				// spam votes
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_SPAM)
			},
			expectedPass: false,
			expectedBurn: true,
			expectedTally: v1.TallyResult{
				YesCount:         "1000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "6000000",
			},
		},
		{
			name: "quorum reached, yes quorum not reached: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				params, _ := s.keeper.Params.Get(s.ctx)
				params.YesQuorum = "0.7"
				_ = s.keeper.Params.Set(s.ctx, params)

				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "3000000",
				AbstainCount:     "2000000",
				NoCount:          "1000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "3000000",
				OptionTwoCount:   "2000000",
				OptionThreeCount: "1000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			govKeeper, mocks, _, ctx := setupGovKeeper(t, mockAccountKeeperExpectations)
			params := v1.DefaultParams()
			// Ensure params value are different than false
			params.BurnVoteQuorum = true
			params.BurnVoteVeto = true
			err := govKeeper.Params.Set(ctx, params)
			require.NoError(t, err)
			var (
				numVals       = 10
				numDelegators = 5
				addrs         = simtestutil.CreateRandomAccounts(numVals + numDelegators)
				valAddrs      = simtestutil.ConvertAddrsToValAddrs(addrs[:numVals])
				delAddrs      = addrs[numVals:]
			)
			// Mocks a bunch of validators
			mocks.stakingKeeper.EXPECT().
				IterateBondedValidatorsByPower(ctx, gomock.Any()).
				DoAndReturn(
					func(ctx context.Context, fn func(index int64, validator sdk.ValidatorI) bool) error {
						for i := int64(0); i < int64(numVals); i++ {
							valAddr, err := mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(valAddrs[i])
							require.NoError(t, err)
							fn(i, stakingtypes.Validator{
								OperatorAddress: valAddr,
								Status:          stakingtypes.Bonded,
								Tokens:          sdkmath.NewInt(1000000),
								DelegatorShares: sdkmath.LegacyNewDec(1000000),
							})
						}
						return nil
					})

			// Submit and activate a proposal
			proposal, err := govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", delAddrs[0], v1.ProposalType_PROPOSAL_TYPE_EXPEDITED)
			require.NoError(t, err)
			err = govKeeper.ActivateVotingPeriod(ctx, proposal)
			require.NoError(t, err)
			suite := tallyFixture{
				t:        t,
				proposal: proposal,
				valAddrs: valAddrs,
				delAddrs: delAddrs,
				ctx:      ctx,
				keeper:   govKeeper,
				mocks:    mocks,
			}
			tt.setup(suite)

			pass, burn, tally, err := govKeeper.Tally(ctx, proposal)

			require.NoError(t, err)
			assert.Equal(t, tt.expectedPass, pass, "wrong pass")
			assert.Equal(t, tt.expectedBurn, burn, "wrong burn")
			assert.Equal(t, tt.expectedTally, tally)
			// Assert votes removal after tally
			rng := collections.NewPrefixedPairRange[uint64, sdk.AccAddress](proposal.Id)
			_, err = suite.keeper.Votes.Iterate(suite.ctx, rng)
			assert.NoError(t, err)
		})
	}
}

func TestTally_Optimistic(t *testing.T) {
	tests := []struct {
		name          string
		setup         func(tallyFixture)
		expectedPass  bool
		expectedBurn  bool
		expectedTally v1.TallyResult
		expectedError string
	}{
		{
			name: "no votes, no bonded tokens: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 0)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "no votes: prop passes",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes: threshold no not reached, prop passes",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "42",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "42",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "no vote threshold reached: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "4000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "4000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			govKeeper, mocks, _, ctx := setupGovKeeper(t, mockAccountKeeperExpectations)
			params := v1.DefaultParams()
			// Ensure params value are different than false
			params.BurnVoteQuorum = true
			params.BurnVoteVeto = true
			err := govKeeper.Params.Set(ctx, params)
			require.NoError(t, err)
			var (
				numVals       = 10
				numDelegators = 5
				addrs         = simtestutil.CreateRandomAccounts(numVals + numDelegators)
				valAddrs      = simtestutil.ConvertAddrsToValAddrs(addrs[:numVals])
				delAddrs      = addrs[numVals:]
			)
			// Mocks a bunch of validators
			mocks.stakingKeeper.EXPECT().
				IterateBondedValidatorsByPower(ctx, gomock.Any()).
				DoAndReturn(
					func(ctx context.Context, fn func(index int64, validator sdk.ValidatorI) bool) error {
						for i := int64(0); i < int64(numVals); i++ {
							valAddr, err := mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(valAddrs[i])
							require.NoError(t, err)
							fn(i, stakingtypes.Validator{
								OperatorAddress: valAddr,
								Status:          stakingtypes.Bonded,
								Tokens:          sdkmath.NewInt(1000000),
								DelegatorShares: sdkmath.LegacyNewDec(1000000),
							})
						}
						return nil
					})

			// Submit and activate a proposal
			proposal, err := govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", delAddrs[0], v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC)
			require.NoError(t, err)
			err = govKeeper.ActivateVotingPeriod(ctx, proposal)
			require.NoError(t, err)
			suite := tallyFixture{
				t:        t,
				proposal: proposal,
				valAddrs: valAddrs,
				delAddrs: delAddrs,
				ctx:      ctx,
				keeper:   govKeeper,
				mocks:    mocks,
			}
			tt.setup(suite)

			pass, burn, tally, err := govKeeper.Tally(ctx, proposal)

			require.NoError(t, err)
			assert.Equal(t, tt.expectedPass, pass, "wrong pass")
			assert.Equal(t, tt.expectedBurn, burn, "wrong burn")
			assert.Equal(t, tt.expectedTally, tally)
			// Assert votes removal after tally
			rng := collections.NewPrefixedPairRange[uint64, sdk.AccAddress](proposal.Id)
			_, err = suite.keeper.Votes.Iterate(suite.ctx, rng)
			assert.NoError(t, err)
		})
	}
}

func TestTally_MultipleChoice(t *testing.T) {
	tests := []struct {
		name          string
		setup         func(tallyFixture)
		expectedPass  bool
		expectedBurn  bool
		expectedTally v1.TallyResult
		expectedError string
	}{
		{
			name: "no votes, no bonded tokens: prop fails",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 0)
			},
			expectedPass: false,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "no votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one validator votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "1000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "1000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one account votes without delegation: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				delegatorVote(s, s.delAddrs[0], nil, v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "42",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "42",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes yes, validator votes also yes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "1000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "one delegator votes yes, validator votes no: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{{
					DelegatorAddress: del0Addr,
					ValidatorAddress: val0Addr,
					Shares:           sdkmath.LegacyNewDec(42),
				}}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "42",
				AbstainCount:     "0",
				NoCount:          "999958",
				NoWithVetoCount:  "0",
				OptionOneCount:   "42",
				OptionTwoCount:   "0",
				OptionThreeCount: "999958",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			// one delegator delegates 42 shares to 2 different validators (21 each)
			// delegator votes yes
			// first validator votes yes
			// second validator votes no
			// third validator (no delegation) votes abstain
			name: "delegator with mixed delegations: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				del0Addr, err := s.mocks.acctKeeper.AddressCodec().BytesToString(s.delAddrs[0])
				require.NoError(t, err)
				val0Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[0])
				require.NoError(t, err)
				val1Addr, err := s.mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(s.valAddrs[1])
				require.NoError(t, err)
				delegations := []stakingtypes.Delegation{
					{
						DelegatorAddress: del0Addr,
						ValidatorAddress: val0Addr,
						Shares:           sdkmath.LegacyNewDec(21),
					},
					{
						DelegatorAddress: del0Addr,
						ValidatorAddress: val1Addr,
						Shares:           sdkmath.LegacyNewDec(21),
					},
				}
				delegatorVote(s, s.delAddrs[0], delegations, v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: false,
			expectedBurn: true, // burn because quorum not reached
			expectedTally: v1.TallyResult{
				YesCount:         "1000021",
				AbstainCount:     "1000000",
				NoCount:          "999979",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000021",
				OptionTwoCount:   "1000000",
				OptionThreeCount: "999979",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with only abstain: always passes in multiple choice tally",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_TWO)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_TWO)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "0",
				AbstainCount:     "4000000",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "0",
				OptionTwoCount:   "4000000",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with a majority of option 4: always passes in multiple choice tally",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_FOUR)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_FOUR)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_FOUR)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "4000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "3000000",
				OptionOneCount:   "4000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "3000000",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached, equality: always passes in multiple choice tally",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_ONE)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_THREE)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_THREE)
			},
			expectedPass: true,
			expectedBurn: false,
			expectedTally: v1.TallyResult{
				YesCount:         "2000000",
				AbstainCount:     "0",
				NoCount:          "2000000",
				NoWithVetoCount:  "0",
				OptionOneCount:   "2000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "2000000",
				OptionFourCount:  "0",
				SpamCount:        "0",
			},
		},
		{
			name: "quorum reached with spam > all other votes: prop fails/burn deposit",
			setup: func(s tallyFixture) {
				setTotalBonded(s, 10000000)
				validatorVote(s, s.valAddrs[0], v1.VoteOption_VOTE_OPTION_ONE)
				// spam votes
				validatorVote(s, s.valAddrs[1], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[2], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[3], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[4], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[5], v1.VoteOption_VOTE_OPTION_SPAM)
				validatorVote(s, s.valAddrs[6], v1.VoteOption_VOTE_OPTION_SPAM)
			},
			expectedPass: false,
			expectedBurn: true,
			expectedTally: v1.TallyResult{
				YesCount:         "1000000",
				AbstainCount:     "0",
				NoCount:          "0",
				NoWithVetoCount:  "0",
				OptionOneCount:   "1000000",
				OptionTwoCount:   "0",
				OptionThreeCount: "0",
				OptionFourCount:  "0",
				SpamCount:        "6000000",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			govKeeper, mocks, _, ctx := setupGovKeeper(t, mockAccountKeeperExpectations)
			params := v1.DefaultParams()
			// Ensure params value are different than false
			params.BurnVoteQuorum = true
			params.BurnVoteVeto = true
			err := govKeeper.Params.Set(ctx, params)
			require.NoError(t, err)
			var (
				numVals       = 10
				numDelegators = 5
				addrs         = simtestutil.CreateRandomAccounts(numVals + numDelegators)
				valAddrs      = simtestutil.ConvertAddrsToValAddrs(addrs[:numVals])
				delAddrs      = addrs[numVals:]
			)
			// Mocks a bunch of validators
			mocks.stakingKeeper.EXPECT().
				IterateBondedValidatorsByPower(ctx, gomock.Any()).
				DoAndReturn(
					func(ctx context.Context, fn func(index int64, validator sdk.ValidatorI) bool) error {
						for i := int64(0); i < int64(numVals); i++ {
							valAddr, err := mocks.stakingKeeper.ValidatorAddressCodec().BytesToString(valAddrs[i])
							require.NoError(t, err)
							fn(i, stakingtypes.Validator{
								OperatorAddress: valAddr,
								Status:          stakingtypes.Bonded,
								Tokens:          sdkmath.NewInt(1000000),
								DelegatorShares: sdkmath.LegacyNewDec(1000000),
							})
						}
						return nil
					})

			// Submit and activate a proposal
			proposal, err := govKeeper.SubmitProposal(ctx, nil, "", "title", "summary", delAddrs[0], v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE)
			require.NoError(t, err)
			err = govKeeper.ProposalVoteOptions.Set(ctx, proposal.Id, v1.ProposalVoteOptions{
				OptionOne:   "Vote Option 1",
				OptionTwo:   "Vote Option 2",
				OptionThree: "Vote Option 3",
				OptionFour:  "Vote Option 4",
			})
			require.NoError(t, err)
			err = govKeeper.ActivateVotingPeriod(ctx, proposal)
			require.NoError(t, err)
			suite := tallyFixture{
				t:        t,
				proposal: proposal,
				valAddrs: valAddrs,
				delAddrs: delAddrs,
				ctx:      ctx,
				keeper:   govKeeper,
				mocks:    mocks,
			}
			tt.setup(suite)

			pass, burn, tally, err := govKeeper.Tally(ctx, proposal)

			require.NoError(t, err)
			assert.Equal(t, tt.expectedPass, pass, "wrong pass")
			assert.Equal(t, tt.expectedBurn, burn, "wrong burn")
			assert.Equal(t, tt.expectedTally, tally)
			// Assert votes removal after tally
			rng := collections.NewPrefixedPairRange[uint64, sdk.AccAddress](proposal.Id)
			_, err = suite.keeper.Votes.Iterate(suite.ctx, rng)
			assert.NoError(t, err)
		})
	}
}
