package group

import (
	"fmt"
	"time"

	proto "github.com/cosmos/gogoproto/proto"

	errorsmod "cosmossdk.io/errors"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	"github.com/cosmos/cosmos-sdk/x/group/internal/math"
	"github.com/cosmos/cosmos-sdk/x/group/internal/orm"
)

// DecisionPolicyResult is the result of whether a proposal passes or not a
// decision policy.
type DecisionPolicyResult struct {
	// Allow determines if the proposal is allowed to pass.
	Allow bool
	// Final determines if the tally result is final or not. If final, then
	// votes are pruned, and the tally result is saved in the proposal's
	// `FinalTallyResult` field.
	Final bool
}

// DecisionPolicy is the persistent set of rules to determine the result of election on a proposal.
type DecisionPolicy interface {
	proto.Message

	// GetVotingPeriod returns the duration after proposal submission where
	// votes are accepted.
	GetVotingPeriod() time.Duration
	// GetMinExecutionPeriod returns the minimum duration after submission
	// where we can execution a proposal. It can be set to 0 or to a value
	// lesser than VotingPeriod to allow TRY_EXEC.
	GetMinExecutionPeriod() time.Duration
	// Allow defines policy-specific logic to allow a proposal to pass or not,
	// based on its tally result, the group's total power and the time since
	// the proposal was submitted.
	Allow(tallyResult TallyResult, totalPower string) (DecisionPolicyResult, error)

	ValidateBasic() error
	Validate(g GroupInfo, config Config) error
}

// Implements DecisionPolicy Interface
var _ DecisionPolicy = &ThresholdDecisionPolicy{}

// NewThresholdDecisionPolicy creates a threshold DecisionPolicy
func NewThresholdDecisionPolicy(threshold string, votingPeriod, minExecutionPeriod time.Duration) DecisionPolicy {
	return &ThresholdDecisionPolicy{threshold, &DecisionPolicyWindows{votingPeriod, minExecutionPeriod}}
}

// GetVotingPeriod returns the voitng period of ThresholdDecisionPolicy
func (p ThresholdDecisionPolicy) GetVotingPeriod() time.Duration {
	return p.Windows.VotingPeriod
}

// GetMinExecutionPeriod returns the minimum execution period of ThresholdDecisionPolicy
func (p ThresholdDecisionPolicy) GetMinExecutionPeriod() time.Duration {
	return p.Windows.MinExecutionPeriod
}

// ValidateBasic does basic validation on ThresholdDecisionPolicy
func (p ThresholdDecisionPolicy) ValidateBasic() error {
	if _, err := math.NewPositiveDecFromString(p.Threshold); err != nil {
		return errorsmod.Wrap(err, "threshold")
	}

	if p.Windows == nil || p.Windows.VotingPeriod == 0 {
		return errorsmod.Wrap(errors.ErrInvalid, "voting period cannot be zero")
	}

	return nil
}

// Allow allows a proposal to pass when the tally of yes votes equals or exceeds the threshold before the timeout.
func (p ThresholdDecisionPolicy) Allow(tallyResult TallyResult, totalPower string) (DecisionPolicyResult, error) {
	threshold, err := math.NewPositiveDecFromString(p.Threshold)
	if err != nil {
		return DecisionPolicyResult{}, errorsmod.Wrap(err, "threshold")
	}
	yesCount, err := math.NewNonNegativeDecFromString(tallyResult.YesCount)
	if err != nil {
		return DecisionPolicyResult{}, errorsmod.Wrap(err, "yes count")
	}

	totalPowerDec, err := math.NewNonNegativeDecFromString(totalPower)
	if err != nil {
		return DecisionPolicyResult{}, errorsmod.Wrap(err, "total power")
	}

	// the real threshold of the policy is `min(threshold,total_weight)`. If
	// the group member weights changes (member leaving, member weight update)
	// and the threshold doesn't, we can end up with threshold > total_weight.
	// In this case, as long as everyone votes yes (in which case
	// `yesCount`==`realThreshold`), then the proposal still passes.
	realThreshold := min(threshold, totalPowerDec)

	if yesCount.Cmp(realThreshold) >= 0 {
		return DecisionPolicyResult{Allow: true, Final: true}, nil
	}

	totalCounts, err := tallyResult.TotalCounts()
	if err != nil {
		return DecisionPolicyResult{}, err
	}
	undecided, err := math.SubNonNegative(totalPowerDec, totalCounts)
	if err != nil {
		return DecisionPolicyResult{}, err
	}
	// maxYesCount is the max potential number of yes count, i.e the current yes count
	// plus all undecided count (supposing they all vote yes).
	maxYesCount, err := yesCount.Add(undecided)
	if err != nil {
		return DecisionPolicyResult{}, err
	}

	if maxYesCount.Cmp(realThreshold) < 0 {
		return DecisionPolicyResult{Allow: false, Final: true}, nil
	}
	return DecisionPolicyResult{Allow: false, Final: false}, nil
}

func min(a, b math.Dec) math.Dec {
	if a.Cmp(b) < 0 {
		return a
	}
	return b
}

// Validate validates the policy against the group. Note that the threshold
// can actually be greater than the group's total weight: in the Allow method
// we check the tally weight against `min(threshold,total_weight)`.
func (p *ThresholdDecisionPolicy) Validate(g GroupInfo, config Config) error {
	_, err := math.NewPositiveDecFromString(p.Threshold)
	if err != nil {
		return errorsmod.Wrap(err, "threshold")
	}
	_, err = math.NewNonNegativeDecFromString(g.TotalWeight)
	if err != nil {
		return errorsmod.Wrap(err, "group total weight")
	}

	if p.Windows.MinExecutionPeriod > p.Windows.VotingPeriod+config.MaxExecutionPeriod {
		return errorsmod.Wrap(errors.ErrInvalid, "min_execution_period should be smaller than voting_period + max_execution_period")
	}
	return nil
}

// Implements DecisionPolicy Interface
var _ DecisionPolicy = &PercentageDecisionPolicy{}

// NewPercentageDecisionPolicy creates a new percentage DecisionPolicy
func NewPercentageDecisionPolicy(percentage string, votingPeriod, executionPeriod time.Duration) DecisionPolicy {
	return &PercentageDecisionPolicy{percentage, &DecisionPolicyWindows{votingPeriod, executionPeriod}}
}

// GetVotingPeriod returns the voitng period of PercentageDecisionPolicy
func (p PercentageDecisionPolicy) GetVotingPeriod() time.Duration {
	return p.Windows.VotingPeriod
}

// GetMinExecutionPeriod returns the minimum execution period of PercentageDecisionPolicy
func (p PercentageDecisionPolicy) GetMinExecutionPeriod() time.Duration {
	return p.Windows.MinExecutionPeriod
}

// ValidateBasic does basic validation on PercentageDecisionPolicy
func (p PercentageDecisionPolicy) ValidateBasic() error {
	percentage, err := math.NewPositiveDecFromString(p.Percentage)
	if err != nil {
		return errorsmod.Wrap(err, "percentage threshold")
	}
	if percentage.Cmp(math.NewDecFromInt64(1)) == 1 {
		return errorsmod.Wrap(errors.ErrInvalid, "percentage must be > 0 and <= 1")
	}

	if p.Windows == nil || p.Windows.VotingPeriod == 0 {
		return errorsmod.Wrap(errors.ErrInvalid, "voting period cannot be 0")
	}

	return nil
}

// Validate validates the policy against the group.
func (p *PercentageDecisionPolicy) Validate(g GroupInfo, config Config) error {
	if p.Windows.MinExecutionPeriod > p.Windows.VotingPeriod+config.MaxExecutionPeriod {
		return errorsmod.Wrap(errors.ErrInvalid, "min_execution_period should be smaller than voting_period + max_execution_period")
	}
	return nil
}

// Allow allows a proposal to pass when the tally of yes votes equals or exceeds the percentage threshold before the timeout.
func (p PercentageDecisionPolicy) Allow(tally TallyResult, totalPower string) (DecisionPolicyResult, error) {
	percentage, err := math.NewPositiveDecFromString(p.Percentage)
	if err != nil {
		return DecisionPolicyResult{}, errorsmod.Wrap(err, "percentage")
	}
	yesCount, err := math.NewNonNegativeDecFromString(tally.YesCount)
	if err != nil {
		return DecisionPolicyResult{}, errorsmod.Wrap(err, "yes count")
	}
	totalPowerDec, err := math.NewNonNegativeDecFromString(totalPower)
	if err != nil {
		return DecisionPolicyResult{}, errorsmod.Wrap(err, "total power")
	}

	yesPercentage, err := yesCount.Quo(totalPowerDec)
	if err != nil {
		return DecisionPolicyResult{}, err
	}

	if yesPercentage.Cmp(percentage) >= 0 {
		return DecisionPolicyResult{Allow: true, Final: true}, nil
	}

	totalCounts, err := tally.TotalCounts()
	if err != nil {
		return DecisionPolicyResult{}, err
	}
	undecided, err := math.SubNonNegative(totalPowerDec, totalCounts)
	if err != nil {
		return DecisionPolicyResult{}, err
	}
	sum, err := yesCount.Add(undecided)
	if err != nil {
		return DecisionPolicyResult{}, err
	}
	sumPercentage, err := sum.Quo(totalPowerDec)
	if err != nil {
		return DecisionPolicyResult{}, err
	}
	if sumPercentage.Cmp(percentage) < 0 {
		return DecisionPolicyResult{Allow: false, Final: true}, nil
	}
	return DecisionPolicyResult{Allow: false, Final: false}, nil
}

var _ orm.Validateable = GroupPolicyInfo{}

// NewGroupPolicyInfo creates a new GroupPolicyInfo instance
func NewGroupPolicyInfo(address sdk.AccAddress, group uint64, admin sdk.AccAddress, metadata string,
	version uint64, decisionPolicy DecisionPolicy, createdAt time.Time,
) (GroupPolicyInfo, error) {
	p := GroupPolicyInfo{
		Address:   address.String(),
		GroupId:   group,
		Admin:     admin.String(),
		Metadata:  metadata,
		Version:   version,
		CreatedAt: createdAt,
	}

	err := p.SetDecisionPolicy(decisionPolicy)
	if err != nil {
		return GroupPolicyInfo{}, err
	}

	return p, nil
}

// SetDecisionPolicy sets the decision policy for GroupPolicyInfo.
func (g *GroupPolicyInfo) SetDecisionPolicy(decisionPolicy DecisionPolicy) error {
	any, err := codectypes.NewAnyWithValue(decisionPolicy)
	if err != nil {
		return err
	}
	g.DecisionPolicy = any
	return nil
}

// GetDecisionPolicy gets the decision policy of GroupPolicyInfo
func (g GroupPolicyInfo) GetDecisionPolicy() (DecisionPolicy, error) {
	decisionPolicy, ok := g.DecisionPolicy.GetCachedValue().(DecisionPolicy)
	if !ok {
		return nil, sdkerrors.ErrInvalidType.Wrapf("expected %T, got %T", (DecisionPolicy)(nil), g.DecisionPolicy.GetCachedValue())
	}

	return decisionPolicy, nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (g GroupPolicyInfo) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	var decisionPolicy DecisionPolicy
	return unpacker.UnpackAny(g.DecisionPolicy, &decisionPolicy)
}

func (g GroupInfo) PrimaryKeyFields() []interface{} {
	return []interface{}{g.Id}
}

// ValidateBasic does basic validation on group info.
func (g GroupInfo) ValidateBasic() error {
	if g.Id == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "group's GroupId")
	}

	_, err := sdk.AccAddressFromBech32(g.Admin)
	if err != nil {
		return errorsmod.Wrap(err, "admin")
	}

	if _, err := math.NewNonNegativeDecFromString(g.TotalWeight); err != nil {
		return errorsmod.Wrap(err, "total weight")
	}
	if g.Version == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "version")
	}
	return nil
}

func (g GroupPolicyInfo) PrimaryKeyFields() []interface{} {
	addr := sdk.MustAccAddressFromBech32(g.Address)

	return []interface{}{addr.Bytes()}
}

func (g Proposal) PrimaryKeyFields() []interface{} {
	return []interface{}{g.Id}
}

// ValidateBasic does basic validation on group policy info.
func (g GroupPolicyInfo) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(g.Admin)
	if err != nil {
		return errorsmod.Wrap(err, "group policy admin")
	}
	_, err = sdk.AccAddressFromBech32(g.Address)
	if err != nil {
		return errorsmod.Wrap(err, "group policy account address")
	}

	if g.GroupId == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "group policy's group id")
	}
	if g.Version == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "group policy version")
	}
	policy, err := g.GetDecisionPolicy()
	if err != nil {
		return errorsmod.Wrap(err, "group policy decision policy")
	}

	if err := policy.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "group policy's decision policy")
	}
	return nil
}

func (g GroupMember) PrimaryKeyFields() []interface{} {
	addr := sdk.MustAccAddressFromBech32(g.Member.Address)

	return []interface{}{g.GroupId, addr.Bytes()}
}

// ValidateBasic does basic validation on group member.
func (g GroupMember) ValidateBasic() error {
	if g.GroupId == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "group member's group id")
	}

	if _, err := sdk.AccAddressFromBech32(g.Member.Address); err != nil {
		return errorsmod.Wrap(err, "group member's address")
	}

	if _, err := math.NewNonNegativeDecFromString(g.Member.Weight); err != nil {
		return errorsmod.Wrap(err, "weight must be non negative")
	}

	return nil
}

// MemberToMemberRequest converts a `Member` (used for storage)
// to a `MemberRequest` (used in requests). The only difference
// between the two is that `MemberRequest` doesn't have any `AddedAt` field
// since it cannot be set as part of requests.
func MemberToMemberRequest(m *Member) MemberRequest {
	return MemberRequest{
		Address:  m.Address,
		Weight:   m.Weight,
		Metadata: m.Metadata,
	}
}

// ValidateBasic does basic validation on proposal.
func (g Proposal) ValidateBasic() error {
	if g.Id == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "proposal id")
	}
	_, err := sdk.AccAddressFromBech32(g.GroupPolicyAddress)
	if err != nil {
		return errorsmod.Wrap(err, "proposal group policy address")
	}
	if g.GroupVersion == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "proposal group version")
	}
	if g.GroupPolicyVersion == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "proposal group policy version")
	}
	_, err = g.FinalTallyResult.GetYesCount()
	if err != nil {
		return errorsmod.Wrap(err, "proposal FinalTallyResult yes count")
	}
	_, err = g.FinalTallyResult.GetNoCount()
	if err != nil {
		return errorsmod.Wrap(err, "proposal FinalTallyResult no count")
	}
	_, err = g.FinalTallyResult.GetAbstainCount()
	if err != nil {
		return errorsmod.Wrap(err, "proposal FinalTallyResult abstain count")
	}
	_, err = g.FinalTallyResult.GetNoWithVetoCount()
	if err != nil {
		return errorsmod.Wrap(err, "proposal FinalTallyResult veto count")
	}
	return nil
}

func (v Vote) PrimaryKeyFields() []interface{} {
	addr := sdk.MustAccAddressFromBech32(v.Voter)

	return []interface{}{v.ProposalId, addr.Bytes()}
}

var _ orm.Validateable = Vote{}

// ValidateBasic does basic validation on vote.
func (v Vote) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(v.Voter)
	if err != nil {
		return errorsmod.Wrap(err, "voter")
	}
	if v.ProposalId == 0 {
		return errorsmod.Wrap(errors.ErrEmpty, "voter ProposalId")
	}
	if v.Option == VOTE_OPTION_UNSPECIFIED {
		return errorsmod.Wrap(errors.ErrEmpty, "voter vote option")
	}
	if _, ok := VoteOption_name[int32(v.Option)]; !ok {
		return errorsmod.Wrap(errors.ErrInvalid, "vote option")
	}
	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (q QueryGroupPoliciesByGroupResponse) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpackGroupPolicies(unpacker, q.GroupPolicies)
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (q QueryGroupPoliciesByAdminResponse) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpackGroupPolicies(unpacker, q.GroupPolicies)
}

func unpackGroupPolicies(unpacker codectypes.AnyUnpacker, accs []*GroupPolicyInfo) error {
	for _, g := range accs {
		err := g.UnpackInterfaces(unpacker)
		if err != nil {
			return err
		}
	}

	return nil
}

type operation func(x, y math.Dec) (math.Dec, error)

func (t *TallyResult) operation(vote Vote, weight string, op operation) error {
	weightDec, err := math.NewPositiveDecFromString(weight)
	if err != nil {
		return err
	}

	yesCount, err := t.GetYesCount()
	if err != nil {
		return errorsmod.Wrap(err, "yes count")
	}
	noCount, err := t.GetNoCount()
	if err != nil {
		return errorsmod.Wrap(err, "no count")
	}
	abstainCount, err := t.GetAbstainCount()
	if err != nil {
		return errorsmod.Wrap(err, "abstain count")
	}
	vetoCount, err := t.GetNoWithVetoCount()
	if err != nil {
		return errorsmod.Wrap(err, "veto count")
	}

	switch vote.Option {
	case VOTE_OPTION_YES:
		yesCount, err := op(yesCount, weightDec)
		if err != nil {
			return errorsmod.Wrap(err, "yes count")
		}
		t.YesCount = yesCount.String()
	case VOTE_OPTION_NO:
		noCount, err := op(noCount, weightDec)
		if err != nil {
			return errorsmod.Wrap(err, "no count")
		}
		t.NoCount = noCount.String()
	case VOTE_OPTION_ABSTAIN:
		abstainCount, err := op(abstainCount, weightDec)
		if err != nil {
			return errorsmod.Wrap(err, "abstain count")
		}
		t.AbstainCount = abstainCount.String()
	case VOTE_OPTION_NO_WITH_VETO:
		vetoCount, err := op(vetoCount, weightDec)
		if err != nil {
			return errorsmod.Wrap(err, "veto count")
		}
		t.NoWithVetoCount = vetoCount.String()
	default:
		return errorsmod.Wrapf(errors.ErrInvalid, "unknown vote option %s", vote.Option.String())
	}
	return nil
}

// GetYesCount returns the number of yes counts from tally result.
func (t TallyResult) GetYesCount() (math.Dec, error) {
	yesCount, err := math.NewNonNegativeDecFromString(t.YesCount)
	if err != nil {
		return math.Dec{}, err
	}
	return yesCount, nil
}

// GetNoCount returns the number of no counts from tally result.
func (t TallyResult) GetNoCount() (math.Dec, error) {
	noCount, err := math.NewNonNegativeDecFromString(t.NoCount)
	if err != nil {
		return math.Dec{}, err
	}
	return noCount, nil
}

// GetAbstainCount returns the number of abstain counts from tally result.
func (t TallyResult) GetAbstainCount() (math.Dec, error) {
	abstainCount, err := math.NewNonNegativeDecFromString(t.AbstainCount)
	if err != nil {
		return math.Dec{}, err
	}
	return abstainCount, nil
}

// GetNoWithVetoCount returns the number of no with veto counts from tally result.
func (t TallyResult) GetNoWithVetoCount() (math.Dec, error) {
	vetoCount, err := math.NewNonNegativeDecFromString(t.NoWithVetoCount)
	if err != nil {
		return math.Dec{}, err
	}
	return vetoCount, nil
}

func (t *TallyResult) Add(vote Vote, weight string) error {
	if err := t.operation(vote, weight, math.Add); err != nil {
		return err
	}
	return nil
}

// TotalCounts is the sum of all weights.
func (t TallyResult) TotalCounts() (math.Dec, error) {
	yesCount, err := t.GetYesCount()
	if err != nil {
		return math.Dec{}, errorsmod.Wrap(err, "yes count")
	}
	noCount, err := t.GetNoCount()
	if err != nil {
		return math.Dec{}, errorsmod.Wrap(err, "no count")
	}
	abstainCount, err := t.GetAbstainCount()
	if err != nil {
		return math.Dec{}, errorsmod.Wrap(err, "abstain count")
	}
	vetoCount, err := t.GetNoWithVetoCount()
	if err != nil {
		return math.Dec{}, errorsmod.Wrap(err, "veto count")
	}

	totalCounts := math.NewDecFromInt64(0)
	totalCounts, err = totalCounts.Add(yesCount)
	if err != nil {
		return math.Dec{}, err
	}
	totalCounts, err = totalCounts.Add(noCount)
	if err != nil {
		return math.Dec{}, err
	}
	totalCounts, err = totalCounts.Add(abstainCount)
	if err != nil {
		return math.Dec{}, err
	}
	totalCounts, err = totalCounts.Add(vetoCount)
	if err != nil {
		return math.Dec{}, err
	}
	return totalCounts, nil
}

// DefaultTallyResult returns a TallyResult with all counts set to 0.
func DefaultTallyResult() TallyResult {
	return TallyResult{
		YesCount:        "0",
		NoCount:         "0",
		NoWithVetoCount: "0",
		AbstainCount:    "0",
	}
}

// VoteOptionFromString returns a VoteOption from a string. It returns an error
// if the string is invalid.
func VoteOptionFromString(str string) (VoteOption, error) {
	vo, ok := VoteOption_value[str]
	if !ok {
		return VOTE_OPTION_UNSPECIFIED, fmt.Errorf("'%s' is not a valid vote option", str)
	}
	return VoteOption(vo), nil
}
