package simulation

import (
	"math"
	"math/rand"
	"time"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/gov/keeper"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/simulation"
)

var initialProposalID = uint64(100000000000000)

// Governance message types and routes
var (
	TypeMsgDeposit        = sdk.MsgTypeURL(&v1.MsgDeposit{})
	TypeMsgVote           = sdk.MsgTypeURL(&v1.MsgVote{})
	TypeMsgVoteWeighted   = sdk.MsgTypeURL(&v1.MsgVoteWeighted{})
	TypeMsgSubmitProposal = sdk.MsgTypeURL(&v1.MsgSubmitProposal{})
)

// Simulation operation weights constants
//
//nolint:gosec // these are not hard-coded credentials.
const (
	OpWeightMsgDeposit      = "op_weight_msg_deposit"
	OpWeightMsgVote         = "op_weight_msg_vote"
	OpWeightMsgVoteWeighted = "op_weight_msg_weighted_vote"

	DefaultWeightMsgDeposit      = 100
	DefaultWeightMsgVote         = 67
	DefaultWeightMsgVoteWeighted = 33
	DefaultWeightTextProposal    = 5
)

// WeightedOperations returns all the operations from the module with their respective weights
func WeightedOperations(appParams simtypes.AppParams, cdc codec.JSONCodec, ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper, wMsgs []simtypes.WeightedProposalMsg, wContents []simtypes.WeightedProposalContent) simulation.WeightedOperations { //nolint:staticcheck
	var (
		weightMsgDeposit      int
		weightMsgVote         int
		weightMsgVoteWeighted int
	)

	appParams.GetOrGenerate(cdc, OpWeightMsgDeposit, &weightMsgDeposit, nil,
		func(_ *rand.Rand) {
			weightMsgDeposit = DefaultWeightMsgDeposit
		},
	)

	appParams.GetOrGenerate(cdc, OpWeightMsgVote, &weightMsgVote, nil,
		func(_ *rand.Rand) {
			weightMsgVote = DefaultWeightMsgVote
		},
	)

	appParams.GetOrGenerate(cdc, OpWeightMsgVoteWeighted, &weightMsgVoteWeighted, nil,
		func(_ *rand.Rand) {
			weightMsgVoteWeighted = DefaultWeightMsgVoteWeighted
		},
	)

	// generate the weighted operations for the proposal contents
	var wProposalOps simulation.WeightedOperations
	for _, wMsg := range wMsgs {
		wMsg := wMsg // pin variable
		var weight int
		appParams.GetOrGenerate(cdc, wMsg.AppParamsKey(), &weight, nil,
			func(_ *rand.Rand) { weight = wMsg.DefaultWeight() },
		)

		wProposalOps = append(
			wProposalOps,
			simulation.NewWeightedOperation(
				weight,
				SimulateMsgSubmitProposal(ak, bk, k, wMsg.MsgSimulatorFn()),
			),
		)
	}

	// generate the weighted operations for the proposal contents
	var wLegacyProposalOps simulation.WeightedOperations
	for _, wContent := range wContents {
		wContent := wContent // pin variable
		var weight int
		appParams.GetOrGenerate(cdc, wContent.AppParamsKey(), &weight, nil,
			func(_ *rand.Rand) { weight = wContent.DefaultWeight() },
		)

		wLegacyProposalOps = append(
			wLegacyProposalOps,
			simulation.NewWeightedOperation(
				weight,
				SimulateMsgSubmitLegacyProposal(ak, bk, k, wContent.ContentSimulatorFn()),
			),
		)
	}

	wGovOps := simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgDeposit,
			SimulateMsgDeposit(ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgVote,
			SimulateMsgVote(ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgVoteWeighted,
			SimulateMsgVoteWeighted(ak, bk, k),
		),
	}

	return append(wGovOps, append(wProposalOps, wLegacyProposalOps...)...)
}

// SimulateMsgSubmitProposal simulates creating a msg Submit Proposal
// voting on the proposal, and subsequently slashing the proposal. It is implemented using
// future operations.
func SimulateMsgSubmitProposal(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper, msgSim simtypes.MsgSimulatorFn) simtypes.Operation {
	return func(r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgs := []sdk.Msg{}
		proposalMsg := msgSim(r, ctx, accs)
		if proposalMsg != nil {
			msgs = append(msgs, proposalMsg)
		}

		return simulateMsgSubmitProposal(ak, bk, k, msgs)(r, app, ctx, accs, chainID)
	}
}

// SimulateMsgSubmitLegacyProposal simulates creating a msg Submit Proposal
// voting on the proposal, and subsequently slashing the proposal. It is implemented using
// future operations.
func SimulateMsgSubmitLegacyProposal(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper, contentSim simtypes.ContentSimulatorFn) simtypes.Operation { //nolint:staticcheck
	return func(r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		// 1) submit proposal now
		content := contentSim(r, ctx, accs)
		if content == nil {
			return simtypes.NoOpMsg(types.ModuleName, TypeMsgSubmitProposal, "content is nil"), nil, nil
		}

		govacc := k.GetGovernanceAccount(ctx)
		contentMsg, err := v1.NewLegacyContent(content, govacc.GetAddress().String())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, TypeMsgSubmitProposal, "error converting legacy content into proposal message"), nil, err
		}

		return simulateMsgSubmitProposal(ak, bk, k, []sdk.Msg{contentMsg})(r, app, ctx, accs, chainID)
	}
}

func simulateMsgSubmitProposal(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper, proposalMsgs []sdk.Msg) simtypes.Operation {
	// The states are:
	// column 1: All validators vote
	// column 2: 90% vote
	// column 3: 75% vote
	// column 4: 40% vote
	// column 5: 15% vote
	// column 6: noone votes
	// All columns sum to 100 for simplicity, values chosen by @valardragon semi-arbitrarily,
	// feel free to change.
	numVotesTransitionMatrix, _ := simulation.CreateTransitionMatrix([][]int{
		{20, 10, 0, 0, 0, 0},
		{55, 50, 20, 10, 0, 0},
		{25, 25, 30, 25, 30, 15},
		{0, 15, 30, 25, 30, 30},
		{0, 0, 20, 30, 30, 30},
		{0, 0, 0, 10, 10, 25},
	})

	statePercentageArray := []float64{1, .9, .75, .4, .15, 0}
	curNumVotesState := 1

	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		simAccount, _ := simtypes.RandomAcc(r, accs)
		deposit, skip, err := randomDeposit(r, ctx, ak, bk, k, simAccount.Address, true)
		switch {
		case skip:
			return simtypes.NoOpMsg(types.ModuleName, TypeMsgSubmitProposal, "skip deposit"), nil, nil
		case err != nil:
			return simtypes.NoOpMsg(types.ModuleName, TypeMsgSubmitProposal, "unable to generate deposit"), nil, err
		}

		msg, err := v1.NewMsgSubmitProposal(
			proposalMsgs,
			deposit,
			simAccount.Address.String(),
			simtypes.RandStringOfLength(r, 100),
			simtypes.RandStringOfLength(r, 100),
			simtypes.RandStringOfLength(r, 100),
		)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msg.Type(), "unable to generate a submit proposal msg"), nil, err
		}

		account := ak.GetAccount(ctx, simAccount.Address)
		txGen := moduletestutil.MakeTestEncodingConfig().TxConfig
		tx, err := simtestutil.GenSignedMockTx(
			r,
			txGen,
			[]sdk.Msg{msg},
			sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 0)},
			simtestutil.DefaultGenTxGas,
			chainID,
			[]uint64{account.GetAccountNumber()},
			[]uint64{account.GetSequence()},
			simAccount.PrivKey,
		)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msg.Type(), "unable to generate mock tx"), nil, err
		}

		_, _, err = app.SimDeliver(txGen.TxEncoder(), tx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msg.Type(), "unable to deliver tx"), nil, err
		}

		opMsg := simtypes.NewOperationMsg(msg, true, "", nil)

		// get the submitted proposal ID
		proposalID, err := k.GetProposalID(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msg.Type(), "unable to generate proposalID"), nil, err
		}

		// 2) Schedule operations for votes
		// 2.1) first pick a number of people to vote.
		curNumVotesState = numVotesTransitionMatrix.NextState(r, curNumVotesState)
		numVotes := int(math.Ceil(float64(len(accs)) * statePercentageArray[curNumVotesState]))

		// 2.2) select who votes and when
		whoVotes := r.Perm(len(accs))

		// didntVote := whoVotes[numVotes:]
		whoVotes = whoVotes[:numVotes]
		votingPeriod := k.GetParams(ctx).VotingPeriod

		fops := make([]simtypes.FutureOperation, numVotes+1)
		for i := 0; i < numVotes; i++ {
			whenVote := ctx.BlockHeader().Time.Add(time.Duration(r.Int63n(int64(votingPeriod.Seconds()))) * time.Second)
			fops[i] = simtypes.FutureOperation{
				BlockTime: whenVote,
				Op:        operationSimulateMsgVote(ak, bk, k, accs[whoVotes[i]], int64(proposalID)),
			}
		}

		return opMsg, fops, nil
	}
}

// SimulateMsgDeposit generates a MsgDeposit with random values.
func SimulateMsgDeposit(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context,
		accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		simAccount, _ := simtypes.RandomAcc(r, accs)
		proposalID, ok := randomProposalID(r, k, ctx, v1.StatusDepositPeriod)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, TypeMsgDeposit, "unable to generate proposalID"), nil, nil
		}

		deposit, skip, err := randomDeposit(r, ctx, ak, bk, k, simAccount.Address, false)
		switch {
		case skip:
			return simtypes.NoOpMsg(types.ModuleName, TypeMsgDeposit, "skip deposit"), nil, nil
		case err != nil:
			return simtypes.NoOpMsg(types.ModuleName, TypeMsgDeposit, "unable to generate deposit"), nil, err
		}

		msg := v1.NewMsgDeposit(simAccount.Address, proposalID, deposit)

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		var fees sdk.Coins
		coins, hasNeg := spendable.SafeSub(deposit...)
		if !hasNeg {
			fees, err = simtypes.RandomFees(r, ctx, coins)
			if err != nil {
				return simtypes.NoOpMsg(types.ModuleName, msg.Type(), "unable to generate fees"), nil, err
			}
		}

		txCtx := simulation.OperationInput{
			R:             r,
			App:           app,
			TxGen:         moduletestutil.MakeTestEncodingConfig().TxConfig,
			Cdc:           nil,
			Msg:           msg,
			MsgType:       msg.Type(),
			Context:       ctx,
			SimAccount:    simAccount,
			AccountKeeper: ak,
			ModuleName:    types.ModuleName,
		}

		return simulation.GenAndDeliverTx(txCtx, fees)
	}
}

// SimulateMsgVote generates a MsgVote with random values.
func SimulateMsgVote(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper) simtypes.Operation {
	return operationSimulateMsgVote(ak, bk, k, simtypes.Account{}, -1)
}

func operationSimulateMsgVote(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper, simAccount simtypes.Account, proposalIDInt int64) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context,
		accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		if simAccount.Equals(simtypes.Account{}) {
			simAccount, _ = simtypes.RandomAcc(r, accs)
		}

		var proposalID uint64

		switch {
		case proposalIDInt < 0:
			var ok bool
			proposalID, ok = randomProposalID(r, k, ctx, v1.StatusVotingPeriod)
			if !ok {
				return simtypes.NoOpMsg(types.ModuleName, TypeMsgVote, "unable to generate proposalID"), nil, nil
			}
		default:
			proposalID = uint64(proposalIDInt)
		}

		option := randomVotingOption(r)
		msg := v1.NewMsgVote(simAccount.Address, proposalID, option, "")

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           moduletestutil.MakeTestEncodingConfig().TxConfig,
			Cdc:             nil,
			Msg:             msg,
			MsgType:         msg.Type(),
			Context:         ctx,
			SimAccount:      simAccount,
			AccountKeeper:   ak,
			Bankkeeper:      bk,
			ModuleName:      types.ModuleName,
			CoinsSpentInMsg: spendable,
		}

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// SimulateMsgVoteWeighted generates a MsgVoteWeighted with random values.
func SimulateMsgVoteWeighted(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper) simtypes.Operation {
	return operationSimulateMsgVoteWeighted(ak, bk, k, simtypes.Account{}, -1)
}

func operationSimulateMsgVoteWeighted(ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper, simAccount simtypes.Account, proposalIDInt int64) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context,
		accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		if simAccount.Equals(simtypes.Account{}) {
			simAccount, _ = simtypes.RandomAcc(r, accs)
		}

		var proposalID uint64

		switch {
		case proposalIDInt < 0:
			var ok bool
			proposalID, ok = randomProposalID(r, k, ctx, v1.StatusVotingPeriod)
			if !ok {
				return simtypes.NoOpMsg(types.ModuleName, TypeMsgVoteWeighted, "unable to generate proposalID"), nil, nil
			}
		default:
			proposalID = uint64(proposalIDInt)
		}

		options := randomWeightedVotingOptions(r)
		msg := v1.NewMsgVoteWeighted(simAccount.Address, proposalID, options, "")

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           moduletestutil.MakeTestEncodingConfig().TxConfig,
			Cdc:             nil,
			Msg:             msg,
			MsgType:         msg.Type(),
			Context:         ctx,
			SimAccount:      simAccount,
			AccountKeeper:   ak,
			Bankkeeper:      bk,
			ModuleName:      types.ModuleName,
			CoinsSpentInMsg: spendable,
		}

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// Pick a random deposit with a random denomination with a
// deposit amount between (0, min(balance, minDepositAmount))
// This is to simulate multiple users depositing to get the
// proposal above the minimum deposit amount
func randomDeposit(
	r *rand.Rand,
	ctx sdk.Context,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
	addr sdk.AccAddress,
	useMinAmount bool,
) (deposit sdk.Coins, skip bool, err error) {
	account := ak.GetAccount(ctx, addr)
	spendable := bk.SpendableCoins(ctx, account.GetAddress())

	if spendable.Empty() {
		return nil, true, nil // skip
	}

	params := k.GetParams(ctx)
	minDeposit := params.MinDeposit
	denomIndex := r.Intn(len(minDeposit))
	denom := minDeposit[denomIndex].Denom

	spendableBalance := spendable.AmountOf(denom)
	if spendableBalance.IsZero() {
		return nil, true, nil
	}

	minDepositAmount := minDeposit[denomIndex].Amount

	minAmount := sdk.ZeroInt()
	if useMinAmount {
		minDepositPercent, err := sdk.NewDecFromStr(params.MinInitialDepositRatio)
		if err != nil {
			return nil, false, err
		}

		minAmount = sdk.NewDecFromInt(minDepositAmount).Mul(minDepositPercent).TruncateInt()
	}

	amount, err := simtypes.RandPositiveInt(r, minDepositAmount.Sub(minAmount))
	if err != nil {
		return nil, false, err
	}
	amount = amount.Add(minAmount)

	if amount.GT(spendableBalance) {
		return nil, true, nil
	}

	return sdk.Coins{sdk.NewCoin(denom, amount)}, false, nil
}

// Pick a random proposal ID between the initial proposal ID
// (defined in gov GenesisState) and the latest proposal ID
// that matches a given Status.
// It does not provide a default ID.
func randomProposalID(r *rand.Rand, k *keeper.Keeper, ctx sdk.Context, status v1.ProposalStatus) (proposalID uint64, found bool) {
	proposalID, _ = k.GetProposalID(ctx)

	switch {
	case proposalID > initialProposalID:
		// select a random ID between [initialProposalID, proposalID]
		proposalID = uint64(simtypes.RandIntBetween(r, int(initialProposalID), int(proposalID)))

	default:
		// This is called on the first call to this funcion
		// in order to update the global variable
		initialProposalID = proposalID
	}

	proposal, ok := k.GetProposal(ctx, proposalID)
	if !ok || proposal.Status != status {
		return proposalID, false
	}

	return proposalID, true
}

// Pick a random voting option
func randomVotingOption(r *rand.Rand) v1.VoteOption {
	switch r.Intn(4) {
	case 0:
		return v1.OptionYes
	case 1:
		return v1.OptionAbstain
	case 2:
		return v1.OptionNo
	case 3:
		return v1.OptionNoWithVeto
	default:
		panic("invalid vote option")
	}
}

// Pick a random weighted voting options
func randomWeightedVotingOptions(r *rand.Rand) v1.WeightedVoteOptions {
	w1 := r.Intn(100 + 1)
	w2 := r.Intn(100 - w1 + 1)
	w3 := r.Intn(100 - w1 - w2 + 1)
	w4 := 100 - w1 - w2 - w3
	weightedVoteOptions := v1.WeightedVoteOptions{}
	if w1 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionYes,
			Weight: sdk.NewDecWithPrec(int64(w1), 2).String(),
		})
	}
	if w2 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionAbstain,
			Weight: sdk.NewDecWithPrec(int64(w2), 2).String(),
		})
	}
	if w3 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionNo,
			Weight: sdk.NewDecWithPrec(int64(w3), 2).String(),
		})
	}
	if w4 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionNoWithVeto,
			Weight: sdk.NewDecWithPrec(int64(w4), 2).String(),
		})
	}
	return weightedVoteOptions
}
