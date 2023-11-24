package keeper

import (
	"context"
	"encoding/json"
	"fmt"

	"cosmossdk.io/errors"
	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

type msgServer struct {
	*Keeper
}

// NewMsgServerImpl returns an implementation of the gov MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(keeper *Keeper) v1.MsgServer {
	return &msgServer{Keeper: keeper}
}

var _ v1.MsgServer = msgServer{}

// SubmitProposal implements the MsgServer.SubmitProposal method.
func (k msgServer) SubmitProposal(goCtx context.Context, msg *v1.MsgSubmitProposal) (*v1.MsgSubmitProposalResponse, error) {
	if msg.Title == "" {
		return nil, errors.Wrap(sdkerrors.ErrInvalidRequest, "proposal title cannot be empty")
	}
	if msg.Summary == "" {
		return nil, errors.Wrap(sdkerrors.ErrInvalidRequest, "proposal summary cannot be empty")
	}

	proposer, err := k.authKeeper.AddressCodec().StringToBytes(msg.GetProposer())
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid proposer address: %s", err)
	}

	// check that either metadata or Msgs length is non nil.
	if len(msg.Messages) == 0 && len(msg.Metadata) == 0 {
		return nil, errors.Wrap(govtypes.ErrNoProposalMsgs, "either metadata or Msgs length must be non-nil")
	}

	// verify that if present, the metadata title and summary equals the proposal title and summary
	if len(msg.Metadata) != 0 {
		proposalMetadata := govtypes.ProposalMetadata{}
		if err := json.Unmarshal([]byte(msg.Metadata), &proposalMetadata); err == nil {
			if proposalMetadata.Title != msg.Title {
				return nil, errors.Wrapf(govtypes.ErrInvalidProposalContent, "metadata title '%s' must equal proposal title '%s'", proposalMetadata.Title, msg.Title)
			}

			if proposalMetadata.Summary != msg.Summary {
				return nil, errors.Wrapf(govtypes.ErrInvalidProposalContent, "metadata summary '%s' must equal proposal summary '%s'", proposalMetadata.Summary, msg.Summary)
			}
		}

		// if we can't unmarshal the metadata, this means the client didn't use the recommended metadata format
		// nothing can be done here, and this is still a valid case, so we ignore the error
	}

	proposalMsgs, err := msg.GetMsgs()
	if err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	initialDeposit := msg.GetInitialDeposit()

	params, err := k.Params.Get(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get governance parameters: %w", err)
	}

	if err := k.validateInitialDeposit(ctx, params, initialDeposit, msg.Expedited); err != nil {
		return nil, err
	}

	if err := k.validateDepositDenom(ctx, params, initialDeposit); err != nil {
		return nil, err
	}

	proposal, err := k.Keeper.SubmitProposal(ctx, proposalMsgs, msg.Metadata, msg.Title, msg.Summary, proposer, msg.Expedited)
	if err != nil {
		return nil, err
	}

	bytes, err := proposal.Marshal()
	if err != nil {
		return nil, err
	}

	// ref: https://github.com/cosmos/cosmos-sdk/issues/9683
	ctx.GasMeter().ConsumeGas(
		3*ctx.KVGasConfig().WriteCostPerByte*uint64(len(bytes)),
		"submit proposal",
	)

	votingStarted, err := k.Keeper.AddDeposit(ctx, proposal.Id, proposer, msg.GetInitialDeposit())
	if err != nil {
		return nil, err
	}

	if votingStarted {
		ctx.EventManager().EmitEvent(
			sdk.NewEvent(govtypes.EventTypeSubmitProposal,
				sdk.NewAttribute(govtypes.AttributeKeyVotingPeriodStart, fmt.Sprintf("%d", proposal.Id)),
			),
		)
	}

	return &v1.MsgSubmitProposalResponse{
		ProposalId: proposal.Id,
	}, nil
}

// CancelProposal implements the MsgServer.CancelProposal method.
func (k msgServer) CancelProposal(goCtx context.Context, msg *v1.MsgCancelProposal) (*v1.MsgCancelProposalResponse, error) {
	_, err := k.authKeeper.AddressCodec().StringToBytes(msg.Proposer)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid proposer address: %s", err)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	if err := k.Keeper.CancelProposal(ctx, msg.ProposalId, msg.Proposer); err != nil {
		return nil, err
	}

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			govtypes.EventTypeCancelProposal,
			sdk.NewAttribute(sdk.AttributeKeySender, msg.Proposer),
			sdk.NewAttribute(govtypes.AttributeKeyProposalID, fmt.Sprint(msg.ProposalId)),
		),
	)

	return &v1.MsgCancelProposalResponse{
		ProposalId:     msg.ProposalId,
		CanceledTime:   ctx.BlockTime(),
		CanceledHeight: uint64(ctx.BlockHeight()),
	}, nil
}

// ExecLegacyContent implements the MsgServer.ExecLegacyContent method.
func (k msgServer) ExecLegacyContent(goCtx context.Context, msg *v1.MsgExecLegacyContent) (*v1.MsgExecLegacyContentResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	govAcct := k.GetGovernanceAccount(ctx).GetAddress().String()
	if govAcct != msg.Authority {
		return nil, errors.Wrapf(govtypes.ErrInvalidSigner, "expected %s got %s", govAcct, msg.Authority)
	}

	content, err := v1.LegacyContentFromMessage(msg)
	if err != nil {
		return nil, errors.Wrapf(govtypes.ErrInvalidProposalContent, "%+v", err)
	}

	// Ensure that the content has a respective handler
	if !k.Keeper.legacyRouter.HasRoute(content.ProposalRoute()) {
		return nil, errors.Wrap(govtypes.ErrNoProposalHandlerExists, content.ProposalRoute())
	}

	handler := k.Keeper.legacyRouter.GetRoute(content.ProposalRoute())
	if err := handler(ctx, content); err != nil {
		return nil, errors.Wrapf(govtypes.ErrInvalidProposalContent, "failed to run legacy handler %s, %+v", content.ProposalRoute(), err)
	}

	return &v1.MsgExecLegacyContentResponse{}, nil
}

// Vote implements the MsgServer.Vote method.
func (k msgServer) Vote(goCtx context.Context, msg *v1.MsgVote) (*v1.MsgVoteResponse, error) {
	accAddr, err := k.authKeeper.AddressCodec().StringToBytes(msg.Voter)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid voter address: %s", err)
	}

	if !v1.ValidVoteOption(msg.Option) {
		return nil, errors.Wrap(govtypes.ErrInvalidVote, msg.Option.String())
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	err = k.Keeper.AddVote(ctx, msg.ProposalId, accAddr, v1.NewNonSplitVoteOption(msg.Option), msg.Metadata)
	if err != nil {
		return nil, err
	}

	return &v1.MsgVoteResponse{}, nil
}

// VoteWeighted implements the MsgServer.VoteWeighted method.
func (k msgServer) VoteWeighted(goCtx context.Context, msg *v1.MsgVoteWeighted) (*v1.MsgVoteWeightedResponse, error) {
	accAddr, accErr := k.authKeeper.AddressCodec().StringToBytes(msg.Voter)
	if accErr != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid voter address: %s", accErr)
	}

	if len(msg.Options) == 0 {
		return nil, errors.Wrap(sdkerrors.ErrInvalidRequest, v1.WeightedVoteOptions(msg.Options).String())
	}

	totalWeight := math.LegacyNewDec(0)
	usedOptions := make(map[v1.VoteOption]bool)
	for _, option := range msg.Options {
		if !option.IsValid() {
			return nil, errors.Wrap(govtypes.ErrInvalidVote, option.String())
		}
		weight, err := math.LegacyNewDecFromStr(option.Weight)
		if err != nil {
			return nil, errors.Wrapf(govtypes.ErrInvalidVote, "invalid weight: %s", err)
		}
		totalWeight = totalWeight.Add(weight)
		if usedOptions[option.Option] {
			return nil, errors.Wrap(govtypes.ErrInvalidVote, "duplicated vote option")
		}
		usedOptions[option.Option] = true
	}

	if totalWeight.GT(math.LegacyNewDec(1)) {
		return nil, errors.Wrap(govtypes.ErrInvalidVote, "total weight overflow 1.00")
	}

	if totalWeight.LT(math.LegacyNewDec(1)) {
		return nil, errors.Wrap(govtypes.ErrInvalidVote, "total weight lower than 1.00")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	err := k.Keeper.AddVote(ctx, msg.ProposalId, accAddr, msg.Options, msg.Metadata)
	if err != nil {
		return nil, err
	}

	return &v1.MsgVoteWeightedResponse{}, nil
}

// Deposit implements the MsgServer.Deposit method.
func (k msgServer) Deposit(goCtx context.Context, msg *v1.MsgDeposit) (*v1.MsgDepositResponse, error) {
	accAddr, err := k.authKeeper.AddressCodec().StringToBytes(msg.Depositor)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid depositor address: %s", err)
	}

	if err := validateDeposit(msg.Amount); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	votingStarted, err := k.Keeper.AddDeposit(ctx, msg.ProposalId, accAddr, msg.Amount)
	if err != nil {
		return nil, err
	}

	if votingStarted {
		ctx.EventManager().EmitEvent(
			sdk.NewEvent(
				govtypes.EventTypeProposalDeposit,
				sdk.NewAttribute(govtypes.AttributeKeyVotingPeriodStart, fmt.Sprintf("%d", msg.ProposalId)),
			),
		)
	}

	return &v1.MsgDepositResponse{}, nil
}

// UpdateParams implements the MsgServer.UpdateParams method.
func (k msgServer) UpdateParams(goCtx context.Context, msg *v1.MsgUpdateParams) (*v1.MsgUpdateParamsResponse, error) {
	if k.authority != msg.Authority {
		return nil, errors.Wrapf(govtypes.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, msg.Authority)
	}

	if err := msg.Params.ValidateBasic(); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	if err := k.Params.Set(ctx, msg.Params); err != nil {
		return nil, err
	}

	return &v1.MsgUpdateParamsResponse{}, nil
}

type legacyMsgServer struct {
	govAcct string
	server  v1.MsgServer
}

// NewLegacyMsgServerImpl returns an implementation of the v1beta1 legacy MsgServer interface. It wraps around
// the current MsgServer
func NewLegacyMsgServerImpl(govAcct string, v1Server v1.MsgServer) v1beta1.MsgServer {
	return &legacyMsgServer{govAcct: govAcct, server: v1Server}
}

var _ v1beta1.MsgServer = legacyMsgServer{}

func (k legacyMsgServer) SubmitProposal(goCtx context.Context, msg *v1beta1.MsgSubmitProposal) (*v1beta1.MsgSubmitProposalResponse, error) {
	content := msg.GetContent()
	if content == nil {
		return nil, errors.Wrap(govtypes.ErrInvalidProposalContent, "missing content")
	}
	if !v1beta1.IsValidProposalType(content.ProposalType()) {
		return nil, errors.Wrap(govtypes.ErrInvalidProposalType, content.ProposalType())
	}
	if err := content.ValidateBasic(); err != nil {
		return nil, err
	}

	contentMsg, err := v1.NewLegacyContent(msg.GetContent(), k.govAcct)
	if err != nil {
		return nil, fmt.Errorf("error converting legacy content into proposal message: %w", err)
	}

	proposal, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{contentMsg},
		msg.InitialDeposit,
		msg.Proposer,
		"",
		msg.GetContent().GetTitle(),
		msg.GetContent().GetDescription(),
		false, // legacy proposals cannot be expedited
	)
	if err != nil {
		return nil, err
	}

	resp, err := k.server.SubmitProposal(goCtx, proposal)
	if err != nil {
		return nil, err
	}

	return &v1beta1.MsgSubmitProposalResponse{ProposalId: resp.ProposalId}, nil
}

func (k legacyMsgServer) Vote(goCtx context.Context, msg *v1beta1.MsgVote) (*v1beta1.MsgVoteResponse, error) {
	_, err := k.server.Vote(goCtx, &v1.MsgVote{
		ProposalId: msg.ProposalId,
		Voter:      msg.Voter,
		Option:     v1.VoteOption(msg.Option),
	})
	if err != nil {
		return nil, err
	}
	return &v1beta1.MsgVoteResponse{}, nil
}

func (k legacyMsgServer) VoteWeighted(goCtx context.Context, msg *v1beta1.MsgVoteWeighted) (*v1beta1.MsgVoteWeightedResponse, error) {
	opts := make([]*v1.WeightedVoteOption, len(msg.Options))
	for idx, opt := range msg.Options {
		opts[idx] = &v1.WeightedVoteOption{
			Option: v1.VoteOption(opt.Option),
			Weight: opt.Weight.String(),
		}
	}

	_, err := k.server.VoteWeighted(goCtx, &v1.MsgVoteWeighted{
		ProposalId: msg.ProposalId,
		Voter:      msg.Voter,
		Options:    opts,
	})
	if err != nil {
		return nil, err
	}
	return &v1beta1.MsgVoteWeightedResponse{}, nil
}

func (k legacyMsgServer) Deposit(goCtx context.Context, msg *v1beta1.MsgDeposit) (*v1beta1.MsgDepositResponse, error) {
	_, err := k.server.Deposit(goCtx, &v1.MsgDeposit{
		ProposalId: msg.ProposalId,
		Depositor:  msg.Depositor,
		Amount:     msg.Amount,
	})
	if err != nil {
		return nil, err
	}
	return &v1beta1.MsgDepositResponse{}, nil
}

// validateDeposit validates the deposit amount, do not use for initial deposit.
func validateDeposit(amount sdk.Coins) error {
	if !amount.IsValid() || !amount.IsAllPositive() {
		return sdkerrors.ErrInvalidCoins.Wrap(amount.String())
	}

	return nil
}
