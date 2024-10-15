package keeper

import (
	"context"
	"encoding/json"
	"fmt"

	corecontext "cosmossdk.io/core/context"
	"cosmossdk.io/core/event"
	"cosmossdk.io/core/transaction"
	"cosmossdk.io/errors"
	"cosmossdk.io/math"
	govtypes "cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
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
func (k msgServer) SubmitProposal(ctx context.Context, msg *v1.MsgSubmitProposal) (*v1.MsgSubmitProposalResponse, error) {
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

	// This method checks that all message metadata, summary and title
	// has the expected length defined in the module configuration.
	if err := k.validateProposalLengths(msg.Metadata, msg.Title, msg.Summary); err != nil {
		return nil, err
	}

	proposalMsgs, err := msg.GetMsgs()
	if err != nil {
		return nil, err
	}

	params, err := k.Params.Get(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get governance parameters: %w", err)
	}

	if msg.Expedited { // checking for backward compatibility
		msg.ProposalType = v1.ProposalType_PROPOSAL_TYPE_EXPEDITED
	}
	if err := k.validateInitialDeposit(params, msg.GetInitialDeposit(), msg.ProposalType); err != nil {
		return nil, err
	}

	if err := k.validateDepositDenom(params, msg.GetInitialDeposit()); err != nil {
		return nil, err
	}

	proposal, err := k.Keeper.SubmitProposal(ctx, proposalMsgs, msg.Metadata, msg.Title, msg.Summary, proposer, msg.ProposalType)
	if err != nil {
		return nil, err
	}

	bytes, err := proposal.Marshal()
	if err != nil {
		return nil, err
	}

	// ref: https://github.com/cosmos/cosmos-sdk/issues/9683
	if err := k.GasService.GasMeter(ctx).Consume(
		3*k.GasService.GasConfig(ctx).WriteCostPerByte*uint64(len(bytes)),
		"submit proposal",
	); err != nil {
		return nil, err
	}

	votingStarted, err := k.Keeper.AddDeposit(ctx, proposal.Id, proposer, msg.GetInitialDeposit())
	if err != nil {
		return nil, err
	}

	if votingStarted {
		if err := k.EventService.EventManager(ctx).EmitKV(
			govtypes.EventTypeSubmitProposal,
			event.NewAttribute(govtypes.AttributeKeyVotingPeriodStart, fmt.Sprintf("%d", proposal.Id)),
		); err != nil {
			return nil, errors.Wrapf(err, "failed to emit event: %s", govtypes.EventTypeSubmitProposal)
		}
	}

	return &v1.MsgSubmitProposalResponse{
		ProposalId: proposal.Id,
	}, nil
}

// SubmitMultipleChoiceProposal implements the MsgServer.SubmitMultipleChoiceProposal method.
func (k msgServer) SubmitMultipleChoiceProposal(ctx context.Context, msg *v1.MsgSubmitMultipleChoiceProposal) (*v1.MsgSubmitMultipleChoiceProposalResponse, error) {
	resp, err := k.SubmitProposal(ctx, &v1.MsgSubmitProposal{
		InitialDeposit: msg.InitialDeposit,
		Proposer:       msg.Proposer,
		Title:          msg.Title,
		Summary:        msg.Summary,
		Metadata:       msg.Metadata,
		ProposalType:   v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE,
	})
	if err != nil {
		return nil, err
	}

	if msg.VoteOptions == nil {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("vote options cannot be nil")
	}

	// check that if a vote option is provided, the previous one is also provided
	if (msg.VoteOptions.OptionTwo != "" && msg.VoteOptions.OptionOne == "") ||
		(msg.VoteOptions.OptionThree != "" && msg.VoteOptions.OptionTwo == "") ||
		(msg.VoteOptions.OptionFour != "" && msg.VoteOptions.OptionThree == "") {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("if a vote option is provided, the previous one must also be provided")
	}

	// check that at least two vote options are provided
	if msg.VoteOptions.OptionOne == "" && msg.VoteOptions.OptionTwo == "" {
		return nil, sdkerrors.ErrInvalidRequest.Wrap("vote options cannot be empty, two or more options must be provided")
	}

	if err := k.ProposalVoteOptions.Set(ctx, resp.ProposalId, *msg.VoteOptions); err != nil {
		return nil, err
	}

	return &v1.MsgSubmitMultipleChoiceProposalResponse{
		ProposalId: resp.ProposalId,
	}, nil
}

// CancelProposal implements the MsgServer.CancelProposal method.
func (k msgServer) CancelProposal(ctx context.Context, msg *v1.MsgCancelProposal) (*v1.MsgCancelProposalResponse, error) {
	_, err := k.authKeeper.AddressCodec().StringToBytes(msg.Proposer)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid proposer address: %s", err)
	}

	if err := k.Keeper.CancelProposal(ctx, msg.ProposalId, msg.Proposer); err != nil {
		return nil, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		govtypes.EventTypeCancelProposal,
		event.NewAttribute(sdk.AttributeKeySender, msg.Proposer),
		event.NewAttribute(govtypes.AttributeKeyProposalID, fmt.Sprint(msg.ProposalId)),
	); err != nil {
		return nil, errors.Wrapf(err, "failed to emit event: %s", govtypes.EventTypeCancelProposal)
	}

	return &v1.MsgCancelProposalResponse{
		ProposalId:     msg.ProposalId,
		CanceledTime:   k.HeaderService.HeaderInfo(ctx).Time,
		CanceledHeight: uint64(k.HeaderService.HeaderInfo(ctx).Height),
	}, nil
}

// ExecLegacyContent implements the MsgServer.ExecLegacyContent method.
func (k msgServer) ExecLegacyContent(ctx context.Context, msg *v1.MsgExecLegacyContent) (*v1.MsgExecLegacyContentResponse, error) {
	govAcct, err := k.authKeeper.AddressCodec().BytesToString(k.GetGovernanceAccount(ctx).GetAddress())
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid governance account address: %s", err)
	}
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

	// NOTE: the support of legacy gov proposal in server/v2 is different than for baseapp.
	// Legacy proposal in server/v2 can only access services provided by the gov module environment.
	if err := handler(context.WithValue(ctx, corecontext.EnvironmentContextKey, k.Environment), content); err != nil {
		return nil, errors.Wrapf(govtypes.ErrInvalidProposalContent, "failed to run legacy handler %s, %+v", content.ProposalRoute(), err)
	}

	return &v1.MsgExecLegacyContentResponse{}, nil
}

// Vote implements the MsgServer.Vote method.
func (k msgServer) Vote(ctx context.Context, msg *v1.MsgVote) (*v1.MsgVoteResponse, error) {
	accAddr, err := k.authKeeper.AddressCodec().StringToBytes(msg.Voter)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid voter address: %s", err)
	}

	if !v1.ValidVoteOption(msg.Option) {
		return nil, errors.Wrap(govtypes.ErrInvalidVote, msg.Option.String())
	}

	if err = k.Keeper.AddVote(ctx, msg.ProposalId, accAddr, v1.NewNonSplitVoteOption(msg.Option), msg.Metadata); err != nil {
		return nil, err
	}

	return &v1.MsgVoteResponse{}, nil
}

// VoteWeighted implements the MsgServer.VoteWeighted method.
func (k msgServer) VoteWeighted(ctx context.Context, msg *v1.MsgVoteWeighted) (*v1.MsgVoteWeightedResponse, error) {
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

	err := k.Keeper.AddVote(ctx, msg.ProposalId, accAddr, msg.Options, msg.Metadata)
	if err != nil {
		return nil, err
	}

	return &v1.MsgVoteWeightedResponse{}, nil
}

// Deposit implements the MsgServer.Deposit method.
func (k msgServer) Deposit(ctx context.Context, msg *v1.MsgDeposit) (*v1.MsgDepositResponse, error) {
	accAddr, err := k.authKeeper.AddressCodec().StringToBytes(msg.Depositor)
	if err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid depositor address: %s", err)
	}

	if err := validateDeposit(msg.Amount); err != nil {
		return nil, err
	}

	votingStarted, err := k.Keeper.AddDeposit(ctx, msg.ProposalId, accAddr, msg.Amount)
	if err != nil {
		return nil, err
	}

	if votingStarted {
		if err := k.EventService.EventManager(ctx).EmitKV(
			govtypes.EventTypeProposalDeposit,
			event.NewAttribute(govtypes.AttributeKeyVotingPeriodStart, fmt.Sprintf("%d", msg.ProposalId)),
		); err != nil {
			return nil, errors.Wrapf(err, "failed to emit event: %s", govtypes.EventTypeProposalDeposit)
		}
	}

	return &v1.MsgDepositResponse{}, nil
}

// UpdateParams implements the v1.UpdateParams method.
func (k msgServer) UpdateParams(ctx context.Context, msg *v1.MsgUpdateParams) (*v1.MsgUpdateParamsResponse, error) {
	if k.authority != msg.Authority {
		return nil, errors.Wrapf(govtypes.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, msg.Authority)
	}

	if err := msg.Params.ValidateBasic(k.authKeeper.AddressCodec()); err != nil {
		return nil, err
	}

	if err := k.Params.Set(ctx, msg.Params); err != nil {
		return nil, err
	}

	return &v1.MsgUpdateParamsResponse{}, nil
}

// UpdateMessageParams implements the v1.MsgServer method
func (k msgServer) UpdateMessageParams(ctx context.Context, msg *v1.MsgUpdateMessageParams) (*v1.MsgUpdateMessageParamsResponse, error) {
	if k.authority != msg.Authority {
		return nil, errors.Wrapf(govtypes.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, msg.Authority)
	}

	// delete the message params if the params are empty
	if msg.Params == nil || *msg.Params == (v1.MessageBasedParams{}) {
		if err := k.MessageBasedParams.Remove(ctx, msg.MsgUrl); err != nil {
			return nil, err
		}

		return &v1.MsgUpdateMessageParamsResponse{}, nil
	}

	if err := msg.Params.ValidateBasic(); err != nil {
		return nil, err
	}

	// note: we don't need to validate the message URL here, as it is gov gated
	// a chain may want to configure proposal messages before having an upgrade
	// adding new messages.

	if err := k.MessageBasedParams.Set(ctx, msg.MsgUrl, *msg.Params); err != nil {
		return nil, err
	}

	return &v1.MsgUpdateMessageParamsResponse{}, nil
}

// SudoExec implements the v1.MsgServer method
func (k msgServer) SudoExec(ctx context.Context, msg *v1.MsgSudoExec) (*v1.MsgSudoExecResponse, error) {
	if msg == nil || msg.Msg == nil {
		return nil, errors.Wrap(govtypes.ErrInvalidProposal, "sudo-ed message cannot be nil")
	}

	if k.authority != msg.Authority {
		return nil, errors.Wrapf(govtypes.ErrInvalidSigner, "invalid authority; expected %s, got %s", k.authority, msg.Authority)
	}

	sudoedMsg, err := msg.GetSudoedMsg()
	if err != nil {
		return nil, errors.Wrapf(govtypes.ErrInvalidProposal, "invalid sudo-ed message: %s", err)
	}

	// check if the message implements the HasValidateBasic interface
	if m, ok := sudoedMsg.(sdk.HasValidateBasic); ok {
		if err := m.ValidateBasic(); err != nil {
			return nil, errors.Wrapf(govtypes.ErrInvalidProposal, "invalid sudo-ed message: %s", err)
		}
	}

	var msgResp transaction.Msg
	if err := k.BranchService.Execute(ctx, func(ctx context.Context) error {
		// TODO add route check here
		if err := k.MsgRouterService.CanInvoke(ctx, sdk.MsgTypeURL(sudoedMsg)); err != nil {
			return errors.Wrap(govtypes.ErrInvalidProposal, err.Error())
		}

		msgResp, err = k.MsgRouterService.Invoke(ctx, sudoedMsg)
		if err != nil {
			return errors.Wrapf(err, "failed to execute sudo-ed message; message %v", sudoedMsg)
		}

		return nil
	}); err != nil {
		return nil, err
	}

	// TODO(@julienrbrt): check if events are properly emitted

	msgRespBytes, err := k.cdc.MarshalJSON(msgResp)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to marshal sudo-ed message response; message %v", msgResp)
	}

	return &v1.MsgSudoExecResponse{
		Result: msgRespBytes,
	}, nil
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
		v1.ProposalType_PROPOSAL_TYPE_STANDARD, // legacy proposals can only be standard
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
