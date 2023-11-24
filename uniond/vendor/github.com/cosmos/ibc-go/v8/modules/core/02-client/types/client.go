package types

import (
	"fmt"
	"math"
	"sort"
	"strings"

	proto "github.com/cosmos/gogoproto/proto"

	errorsmod "cosmossdk.io/errors"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ codectypes.UnpackInterfacesMessage = (*IdentifiedClientState)(nil)
	_ codectypes.UnpackInterfacesMessage = (*ConsensusStateWithHeight)(nil)
)

// NewIdentifiedClientState creates a new IdentifiedClientState instance
func NewIdentifiedClientState(clientID string, clientState exported.ClientState) IdentifiedClientState {
	msg, ok := clientState.(proto.Message)
	if !ok {
		panic(fmt.Errorf("cannot proto marshal %T", clientState))
	}

	anyClientState, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		panic(err)
	}

	return IdentifiedClientState{
		ClientId:    clientID,
		ClientState: anyClientState,
	}
}

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (ics IdentifiedClientState) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(ics.ClientState, new(exported.ClientState))
}

var _ sort.Interface = (*IdentifiedClientStates)(nil)

// IdentifiedClientStates defines a slice of ClientConsensusStates that supports the sort interface
type IdentifiedClientStates []IdentifiedClientState

// Len implements sort.Interface
func (ics IdentifiedClientStates) Len() int { return len(ics) }

// Less implements sort.Interface
func (ics IdentifiedClientStates) Less(i, j int) bool { return ics[i].ClientId < ics[j].ClientId }

// Swap implements sort.Interface
func (ics IdentifiedClientStates) Swap(i, j int) { ics[i], ics[j] = ics[j], ics[i] }

// Sort is a helper function to sort the set of IdentifiedClientStates in place
func (ics IdentifiedClientStates) Sort() IdentifiedClientStates {
	sort.Sort(ics)
	return ics
}

// NewConsensusStateWithHeight creates a new ConsensusStateWithHeight instance
func NewConsensusStateWithHeight(height Height, consensusState exported.ConsensusState) ConsensusStateWithHeight {
	msg, ok := consensusState.(proto.Message)
	if !ok {
		panic(fmt.Errorf("cannot proto marshal %T", consensusState))
	}

	anyConsensusState, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		panic(err)
	}

	return ConsensusStateWithHeight{
		Height:         height,
		ConsensusState: anyConsensusState,
	}
}

// UnpackInterfaces implements UnpackInterfacesMesssage.UnpackInterfaces
func (cswh ConsensusStateWithHeight) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(cswh.ConsensusState, new(exported.ConsensusState))
}

// ValidateClientType validates the client type. It cannot be blank or empty. It must be a valid
// client identifier when used with '0' or the maximum uint64 as the sequence.
func ValidateClientType(clientType string) error {
	if strings.TrimSpace(clientType) == "" {
		return errorsmod.Wrap(ErrInvalidClientType, "client type cannot be blank")
	}

	smallestPossibleClientID := FormatClientIdentifier(clientType, 0)
	largestPossibleClientID := FormatClientIdentifier(clientType, uint64(math.MaxUint64))

	// IsValidClientID will check client type format and if the sequence is a uint64
	if !IsValidClientID(smallestPossibleClientID) {
		return errorsmod.Wrap(ErrInvalidClientType, "")
	}

	if err := host.ClientIdentifierValidator(smallestPossibleClientID); err != nil {
		return errorsmod.Wrap(err, "client type results in smallest client identifier being invalid")
	}
	if err := host.ClientIdentifierValidator(largestPossibleClientID); err != nil {
		return errorsmod.Wrap(err, "client type results in largest client identifier being invalid")
	}

	return nil
}
