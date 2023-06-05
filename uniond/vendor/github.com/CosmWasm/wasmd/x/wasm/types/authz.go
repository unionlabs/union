package types

import (
	"strings"

	errorsmod "cosmossdk.io/errors"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	authztypes "github.com/cosmos/cosmos-sdk/x/authz"
	"github.com/cosmos/gogoproto/proto"
)

const gasDeserializationCostPerByte = uint64(1)

var (
	_ authztypes.Authorization         = &ContractExecutionAuthorization{}
	_ authztypes.Authorization         = &ContractMigrationAuthorization{}
	_ cdctypes.UnpackInterfacesMessage = &ContractExecutionAuthorization{}
	_ cdctypes.UnpackInterfacesMessage = &ContractMigrationAuthorization{}
)

// AuthzableWasmMsg is abstract wasm tx message that is supported in authz
type AuthzableWasmMsg interface {
	GetFunds() sdk.Coins
	GetMsg() RawContractMessage
	GetContract() string
	ValidateBasic() error
}

// NewContractExecutionAuthorization constructor
func NewContractExecutionAuthorization(grants ...ContractGrant) *ContractExecutionAuthorization {
	return &ContractExecutionAuthorization{
		Grants: grants,
	}
}

// MsgTypeURL implements Authorization.MsgTypeURL.
func (a ContractExecutionAuthorization) MsgTypeURL() string {
	return sdk.MsgTypeURL(&MsgExecuteContract{})
}

// NewAuthz factory method to create an Authorization with updated grants
func (a ContractExecutionAuthorization) NewAuthz(g []ContractGrant) authztypes.Authorization {
	return NewContractExecutionAuthorization(g...)
}

// Accept implements Authorization.Accept.
func (a *ContractExecutionAuthorization) Accept(ctx sdk.Context, msg sdk.Msg) (authztypes.AcceptResponse, error) {
	return AcceptGrantedMessage[*MsgExecuteContract](ctx, a.Grants, msg, a)
}

// ValidateBasic implements Authorization.ValidateBasic.
func (a ContractExecutionAuthorization) ValidateBasic() error {
	return validateGrants(a.Grants)
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (a ContractExecutionAuthorization) UnpackInterfaces(unpacker cdctypes.AnyUnpacker) error {
	for _, g := range a.Grants {
		if err := g.UnpackInterfaces(unpacker); err != nil {
			return err
		}
	}
	return nil
}

// NewContractMigrationAuthorization constructor
func NewContractMigrationAuthorization(grants ...ContractGrant) *ContractMigrationAuthorization {
	return &ContractMigrationAuthorization{
		Grants: grants,
	}
}

// MsgTypeURL implements Authorization.MsgTypeURL.
func (a ContractMigrationAuthorization) MsgTypeURL() string {
	return sdk.MsgTypeURL(&MsgMigrateContract{})
}

// Accept implements Authorization.Accept.
func (a *ContractMigrationAuthorization) Accept(ctx sdk.Context, msg sdk.Msg) (authztypes.AcceptResponse, error) {
	return AcceptGrantedMessage[*MsgMigrateContract](ctx, a.Grants, msg, a)
}

// NewAuthz factory method to create an Authorization with updated grants
func (a ContractMigrationAuthorization) NewAuthz(g []ContractGrant) authztypes.Authorization {
	return NewContractMigrationAuthorization(g...)
}

// ValidateBasic implements Authorization.ValidateBasic.
func (a ContractMigrationAuthorization) ValidateBasic() error {
	return validateGrants(a.Grants)
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (a ContractMigrationAuthorization) UnpackInterfaces(unpacker cdctypes.AnyUnpacker) error {
	for _, g := range a.Grants {
		if err := g.UnpackInterfaces(unpacker); err != nil {
			return err
		}
	}
	return nil
}

func validateGrants(g []ContractGrant) error {
	if len(g) == 0 {
		return ErrEmpty.Wrap("grants")
	}
	for i, v := range g {
		if err := v.ValidateBasic(); err != nil {
			return errorsmod.Wrapf(err, "position %d", i)
		}
	}
	// allow multiple grants for a contract:
	// contractA:doThis:1,doThat:*  has with different counters for different methods
	return nil
}

// ContractAuthzFactory factory to create an updated Authorization object
type ContractAuthzFactory interface {
	NewAuthz([]ContractGrant) authztypes.Authorization
}

// AcceptGrantedMessage determines whether this grant permits the provided sdk.Msg to be performed,
// and if so provides an upgraded authorization instance.
func AcceptGrantedMessage[T AuthzableWasmMsg](ctx sdk.Context, grants []ContractGrant, msg sdk.Msg, factory ContractAuthzFactory) (authztypes.AcceptResponse, error) {
	exec, ok := msg.(T)
	if !ok {
		return authztypes.AcceptResponse{}, sdkerrors.ErrInvalidType.Wrap("type mismatch")
	}
	if exec.GetMsg() == nil {
		return authztypes.AcceptResponse{}, sdkerrors.ErrInvalidType.Wrap("empty message")
	}
	if err := exec.ValidateBasic(); err != nil {
		return authztypes.AcceptResponse{}, err
	}

	// iterate though all grants
	for i, g := range grants {
		if g.Contract != exec.GetContract() {
			continue
		}

		// first check limits
		result, err := g.GetLimit().Accept(ctx, exec)
		switch {
		case err != nil:
			return authztypes.AcceptResponse{}, errorsmod.Wrap(err, "limit")
		case result == nil: // sanity check
			return authztypes.AcceptResponse{}, sdkerrors.ErrInvalidType.Wrap("limit result must not be nil")
		case !result.Accepted:
			// not applicable, continue with next grant
			continue
		}

		// then check permission set
		ok, err := g.GetFilter().Accept(ctx, exec.GetMsg())
		switch {
		case err != nil:
			return authztypes.AcceptResponse{}, errorsmod.Wrap(err, "filter")
		case !ok:
			// no limit update and continue with next grant
			continue
		}

		// finally do limit state updates in result
		switch {
		case result.DeleteLimit:
			updatedGrants := append(grants[0:i], grants[i+1:]...) //nolint:gocritic
			if len(updatedGrants) == 0 {                          // remove when empty
				return authztypes.AcceptResponse{Accept: true, Delete: true}, nil
			}
			newAuthz := factory.NewAuthz(updatedGrants)
			if err := newAuthz.ValidateBasic(); err != nil { // sanity check
				return authztypes.AcceptResponse{}, ErrInvalid.Wrapf("new grant state: %s", err)
			}
			return authztypes.AcceptResponse{Accept: true, Updated: newAuthz}, nil
		case result.UpdateLimit != nil:
			obj, err := g.WithNewLimits(result.UpdateLimit)
			if err != nil {
				return authztypes.AcceptResponse{}, err
			}
			newAuthz := factory.NewAuthz(append(append(grants[0:i], *obj), grants[i+1:]...))
			if err := newAuthz.ValidateBasic(); err != nil { // sanity check
				return authztypes.AcceptResponse{}, ErrInvalid.Wrapf("new grant state: %s", err)
			}
			return authztypes.AcceptResponse{Accept: true, Updated: newAuthz}, nil
		default: // accepted without a limit state update
			return authztypes.AcceptResponse{Accept: true}, nil
		}
	}
	return authztypes.AcceptResponse{Accept: false}, nil
}

// ContractAuthzLimitX  define execution limits that are enforced and updated when the grant
// is applied. When the limit lapsed the grant is removed.
type ContractAuthzLimitX interface {
	Accept(ctx sdk.Context, msg AuthzableWasmMsg) (*ContractAuthzLimitAcceptResult, error)
	ValidateBasic() error
}

// ContractAuthzLimitAcceptResult result of the ContractAuthzLimitX.Accept method
type ContractAuthzLimitAcceptResult struct {
	// Accepted is true when limit applies
	Accepted bool
	// DeleteLimit when set it is the end of life for this limit. Grant is removed from persistent store
	DeleteLimit bool
	// UpdateLimit update persistent state with new value
	UpdateLimit ContractAuthzLimitX
}

// ContractAuthzFilterX define more fine-grained control on the message payload passed
// to the contract in the operation. When no filter applies on execution, the
// operation is prohibited.
type ContractAuthzFilterX interface {
	// Accept returns applicable or error
	Accept(ctx sdk.Context, msg RawContractMessage) (bool, error)
	ValidateBasic() error
}

var _ cdctypes.UnpackInterfacesMessage = &ContractGrant{}

// NewContractGrant constructor
func NewContractGrant(contract sdk.AccAddress, limit ContractAuthzLimitX, filter ContractAuthzFilterX) (*ContractGrant, error) {
	pFilter, ok := filter.(proto.Message)
	if !ok {
		return nil, sdkerrors.ErrInvalidType.Wrap("filter is not a proto type")
	}
	anyFilter, err := cdctypes.NewAnyWithValue(pFilter)
	if err != nil {
		return nil, errorsmod.Wrap(err, "filter")
	}
	return ContractGrant{
		Contract: contract.String(),
		Filter:   anyFilter,
	}.WithNewLimits(limit)
}

// WithNewLimits factory method to create a new grant with given limit
func (g ContractGrant) WithNewLimits(limit ContractAuthzLimitX) (*ContractGrant, error) {
	pLimit, ok := limit.(proto.Message)
	if !ok {
		return nil, sdkerrors.ErrInvalidType.Wrap("limit is not a proto type")
	}
	anyLimit, err := cdctypes.NewAnyWithValue(pLimit)
	if err != nil {
		return nil, errorsmod.Wrap(err, "limit")
	}

	return &ContractGrant{
		Contract: g.Contract,
		Limit:    anyLimit,
		Filter:   g.Filter,
	}, nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (g ContractGrant) UnpackInterfaces(unpacker cdctypes.AnyUnpacker) error {
	var f ContractAuthzFilterX
	if err := unpacker.UnpackAny(g.Filter, &f); err != nil {
		return errorsmod.Wrap(err, "filter")
	}
	var l ContractAuthzLimitX
	if err := unpacker.UnpackAny(g.Limit, &l); err != nil {
		return errorsmod.Wrap(err, "limit")
	}
	return nil
}

// GetLimit returns the cached value from the ContractGrant.Limit if present.
func (g ContractGrant) GetLimit() ContractAuthzLimitX {
	if g.Limit == nil {
		return &UndefinedLimit{}
	}
	a, ok := g.Limit.GetCachedValue().(ContractAuthzLimitX)
	if !ok {
		return &UndefinedLimit{}
	}
	return a
}

// GetFilter returns the cached value from the ContractGrant.Filter if present.
func (g ContractGrant) GetFilter() ContractAuthzFilterX {
	if g.Filter == nil {
		return &UndefinedFilter{}
	}
	a, ok := g.Filter.GetCachedValue().(ContractAuthzFilterX)
	if !ok {
		return &UndefinedFilter{}
	}
	return a
}

// ValidateBasic validates the grant
func (g ContractGrant) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(g.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	// execution limits
	if err := g.GetLimit().ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "limit")
	}
	// filter
	if err := g.GetFilter().ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "filter")
	}
	return nil
}

// UndefinedFilter null object that is always rejected in execution
type UndefinedFilter struct{}

// Accept always returns error
func (f *UndefinedFilter) Accept(_ sdk.Context, _ RawContractMessage) (bool, error) {
	return false, sdkerrors.ErrNotFound.Wrapf("undefined filter")
}

// ValidateBasic always returns error
func (f UndefinedFilter) ValidateBasic() error {
	return sdkerrors.ErrInvalidType.Wrapf("undefined filter")
}

// NewAllowAllMessagesFilter constructor
func NewAllowAllMessagesFilter() *AllowAllMessagesFilter {
	return &AllowAllMessagesFilter{}
}

// Accept accepts any valid json message content.
func (f *AllowAllMessagesFilter) Accept(_ sdk.Context, msg RawContractMessage) (bool, error) {
	return true, msg.ValidateBasic()
}

// ValidateBasic returns always nil
func (f AllowAllMessagesFilter) ValidateBasic() error {
	return nil
}

// NewAcceptedMessageKeysFilter constructor
func NewAcceptedMessageKeysFilter(acceptedKeys ...string) *AcceptedMessageKeysFilter {
	return &AcceptedMessageKeysFilter{Keys: acceptedKeys}
}

// Accept only payload messages which contain one of the accepted key names in the json object.
func (f *AcceptedMessageKeysFilter) Accept(ctx sdk.Context, msg RawContractMessage) (bool, error) {
	gasForDeserialization := gasDeserializationCostPerByte * uint64(len(msg))
	ctx.GasMeter().ConsumeGas(gasForDeserialization, "contract authorization")

	ok, err := isJSONObjectWithTopLevelKey(msg, f.Keys)
	if err != nil {
		return false, sdkerrors.ErrUnauthorized.Wrapf("not an allowed msg: %s", err.Error())
	}
	return ok, nil
}

// ValidateBasic validates the filter
func (f AcceptedMessageKeysFilter) ValidateBasic() error {
	if len(f.Keys) == 0 {
		return ErrEmpty.Wrap("keys")
	}
	idx := make(map[string]struct{}, len(f.Keys))
	for _, m := range f.Keys {
		if m == "" {
			return ErrEmpty.Wrap("key")
		}
		if m != strings.TrimSpace(m) {
			return ErrInvalid.Wrapf("key %q contains whitespaces", m)
		}
		if _, exists := idx[m]; exists {
			return ErrDuplicate.Wrapf("key %q", m)
		}
		idx[m] = struct{}{}
	}
	return nil
}

// NewAcceptedMessagesFilter constructor
func NewAcceptedMessagesFilter(msgs ...RawContractMessage) *AcceptedMessagesFilter {
	return &AcceptedMessagesFilter{Messages: msgs}
}

// Accept only payload messages which are equal to the granted one.
func (f *AcceptedMessagesFilter) Accept(_ sdk.Context, msg RawContractMessage) (bool, error) {
	for _, v := range f.Messages {
		if v.Equal(msg) {
			return true, nil
		}
	}
	return false, nil
}

// ValidateBasic validates the filter
func (f AcceptedMessagesFilter) ValidateBasic() error {
	if len(f.Messages) == 0 {
		return ErrEmpty.Wrap("messages")
	}
	idx := make(map[string]struct{}, len(f.Messages))
	for _, m := range f.Messages {
		if len(m) == 0 {
			return ErrEmpty.Wrap("message")
		}
		if err := m.ValidateBasic(); err != nil {
			return err
		}
		if _, exists := idx[string(m)]; exists {
			return ErrDuplicate.Wrap("message")
		}
		idx[string(m)] = struct{}{}
	}
	return nil
}

var (
	_ ContractAuthzLimitX = &UndefinedLimit{}
	_ ContractAuthzLimitX = &MaxCallsLimit{}
	_ ContractAuthzLimitX = &MaxFundsLimit{}
	_ ContractAuthzLimitX = &CombinedLimit{}
)

// UndefinedLimit null object that is always rejected in execution
type UndefinedLimit struct{}

// ValidateBasic always returns error
func (u UndefinedLimit) ValidateBasic() error {
	return sdkerrors.ErrInvalidType.Wrapf("undefined limit")
}

// Accept always returns error
func (u UndefinedLimit) Accept(_ sdk.Context, _ AuthzableWasmMsg) (*ContractAuthzLimitAcceptResult, error) {
	return nil, sdkerrors.ErrNotFound.Wrapf("undefined filter")
}

// NewMaxCallsLimit constructor
func NewMaxCallsLimit(number uint64) *MaxCallsLimit {
	return &MaxCallsLimit{Remaining: number}
}

// Accept only the defined number of message calls. No token transfers to the contract allowed.
func (m MaxCallsLimit) Accept(_ sdk.Context, msg AuthzableWasmMsg) (*ContractAuthzLimitAcceptResult, error) {
	if !msg.GetFunds().Empty() {
		return &ContractAuthzLimitAcceptResult{Accepted: false}, nil
	}
	switch n := m.Remaining; n {
	case 0: // sanity check
		return nil, sdkerrors.ErrUnauthorized.Wrap("no calls left")
	case 1:
		return &ContractAuthzLimitAcceptResult{Accepted: true, DeleteLimit: true}, nil
	default:
		return &ContractAuthzLimitAcceptResult{Accepted: true, UpdateLimit: &MaxCallsLimit{Remaining: n - 1}}, nil
	}
}

// ValidateBasic validates the limit
func (m MaxCallsLimit) ValidateBasic() error {
	if m.Remaining == 0 {
		return ErrEmpty.Wrap("remaining calls")
	}
	return nil
}

// NewMaxFundsLimit constructor
// A panic will occur if the coin set is not valid.
func NewMaxFundsLimit(max ...sdk.Coin) *MaxFundsLimit {
	return &MaxFundsLimit{Amounts: sdk.NewCoins(max...)}
}

// Accept until the defined budget for token transfers to the contract is spent
func (m MaxFundsLimit) Accept(_ sdk.Context, msg AuthzableWasmMsg) (*ContractAuthzLimitAcceptResult, error) {
	if msg.GetFunds().Empty() { // no state changes required
		return &ContractAuthzLimitAcceptResult{Accepted: true}, nil
	}
	if !msg.GetFunds().IsAllLTE(m.Amounts) {
		return &ContractAuthzLimitAcceptResult{Accepted: false}, nil
	}
	newAmounts := m.Amounts.Sub(msg.GetFunds()...)
	if newAmounts.IsZero() {
		return &ContractAuthzLimitAcceptResult{Accepted: true, DeleteLimit: true}, nil
	}
	return &ContractAuthzLimitAcceptResult{Accepted: true, UpdateLimit: &MaxFundsLimit{Amounts: newAmounts}}, nil
}

// ValidateBasic validates the limit
func (m MaxFundsLimit) ValidateBasic() error {
	if err := m.Amounts.Validate(); err != nil {
		return err
	}
	if m.Amounts.IsZero() {
		return ErrEmpty.Wrap("amounts")
	}
	return nil
}

// NewCombinedLimit constructor
// A panic will occur if the coin set is not valid.
func NewCombinedLimit(maxCalls uint64, maxAmounts ...sdk.Coin) *CombinedLimit {
	return &CombinedLimit{CallsRemaining: maxCalls, Amounts: sdk.NewCoins(maxAmounts...)}
}

// Accept until the max calls is reached or the token budget is spent.
func (l CombinedLimit) Accept(_ sdk.Context, msg AuthzableWasmMsg) (*ContractAuthzLimitAcceptResult, error) {
	transferFunds := msg.GetFunds()
	if !transferFunds.IsAllLTE(l.Amounts) {
		return &ContractAuthzLimitAcceptResult{Accepted: false}, nil // does not apply
	}
	switch n := l.CallsRemaining; n {
	case 0: // sanity check
		return nil, sdkerrors.ErrUnauthorized.Wrap("no calls left")
	case 1:
		return &ContractAuthzLimitAcceptResult{Accepted: true, DeleteLimit: true}, nil
	default:
		remainingAmounts := l.Amounts.Sub(transferFunds...)
		if remainingAmounts.IsZero() {
			return &ContractAuthzLimitAcceptResult{Accepted: true, DeleteLimit: true}, nil
		}
		return &ContractAuthzLimitAcceptResult{
			Accepted:    true,
			UpdateLimit: NewCombinedLimit(n-1, remainingAmounts...),
		}, nil
	}
}

// ValidateBasic validates the limit
func (l CombinedLimit) ValidateBasic() error {
	if l.CallsRemaining == 0 {
		return ErrEmpty.Wrap("remaining calls")
	}
	if l.Amounts.IsZero() {
		return ErrEmpty.Wrap("amounts")
	}
	if err := l.Amounts.Validate(); err != nil {
		return errorsmod.Wrap(err, "amounts")
	}
	return nil
}
