package types

import (
	"fmt"
	"sort"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ codectypes.UnpackInterfacesMessage = (*IdentifiedClientState)(nil)
	_ codectypes.UnpackInterfacesMessage = (*ClientsConsensusStates)(nil)
	_ codectypes.UnpackInterfacesMessage = (*ClientConsensusStates)(nil)
	_ codectypes.UnpackInterfacesMessage = (*GenesisState)(nil)

	_ sort.Interface           = (*ClientsConsensusStates)(nil)
	_ exported.GenesisMetadata = (*GenesisMetadata)(nil)
)

// ClientsConsensusStates defines a slice of ClientConsensusStates that supports the sort interface
type ClientsConsensusStates []ClientConsensusStates

// Len implements sort.Interface
func (ccs ClientsConsensusStates) Len() int { return len(ccs) }

// Less implements sort.Interface
func (ccs ClientsConsensusStates) Less(i, j int) bool { return ccs[i].ClientId < ccs[j].ClientId }

// Swap implements sort.Interface
func (ccs ClientsConsensusStates) Swap(i, j int) { ccs[i], ccs[j] = ccs[j], ccs[i] }

// Sort is a helper function to sort the set of ClientsConsensusStates in place
func (ccs ClientsConsensusStates) Sort() ClientsConsensusStates {
	sort.Sort(ccs)
	return ccs
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (ccs ClientsConsensusStates) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	for _, clientConsensus := range ccs {
		if err := clientConsensus.UnpackInterfaces(unpacker); err != nil {
			return err
		}
	}
	return nil
}

// NewClientConsensusStates creates a new ClientConsensusStates instance.
func NewClientConsensusStates(clientID string, consensusStates []ConsensusStateWithHeight) ClientConsensusStates {
	return ClientConsensusStates{
		ClientId:        clientID,
		ConsensusStates: consensusStates,
	}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (ccs ClientConsensusStates) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	for _, consStateWithHeight := range ccs.ConsensusStates {
		if err := consStateWithHeight.UnpackInterfaces(unpacker); err != nil {
			return err
		}
	}
	return nil
}

// NewGenesisState creates a GenesisState instance.
func NewGenesisState(
	clients []IdentifiedClientState, clientsConsensus ClientsConsensusStates, clientsMetadata []IdentifiedGenesisMetadata,
	params Params, createLocalhost bool, nextClientSequence uint64,
) GenesisState {
	return GenesisState{
		Clients:            clients,
		ClientsConsensus:   clientsConsensus,
		ClientsMetadata:    clientsMetadata,
		Params:             params,
		CreateLocalhost:    createLocalhost,
		NextClientSequence: nextClientSequence,
	}
}

// DefaultGenesisState returns the ibc client submodule's default genesis state.
func DefaultGenesisState() GenesisState {
	return GenesisState{
		Clients:            []IdentifiedClientState{},
		ClientsConsensus:   ClientsConsensusStates{},
		Params:             DefaultParams(),
		CreateLocalhost:    false,
		NextClientSequence: 0,
	}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (gs GenesisState) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	for _, client := range gs.Clients {
		if err := client.UnpackInterfaces(unpacker); err != nil {
			return err
		}
	}

	return gs.ClientsConsensus.UnpackInterfaces(unpacker)
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (gs GenesisState) Validate() error {
	// keep track of the max sequence to ensure it is less than
	// the next sequence used in creating client identifers.
	var maxSequence uint64

	if err := gs.Params.Validate(); err != nil {
		return err
	}

	validClients := make(map[string]string)

	for i, client := range gs.Clients {
		if err := host.ClientIdentifierValidator(client.ClientId); err != nil {
			return fmt.Errorf("invalid client consensus state identifier %s index %d: %w", client.ClientId, i, err)
		}

		clientState, ok := client.ClientState.GetCachedValue().(exported.ClientState)
		if !ok {
			return fmt.Errorf("invalid client state with ID %s", client.ClientId)
		}

		if !gs.Params.IsAllowedClient(clientState.ClientType()) {
			return fmt.Errorf("client type %s not allowed by genesis params", clientState.ClientType())
		}
		if err := clientState.Validate(); err != nil {
			return fmt.Errorf("invalid client %v index %d: %w", client, i, err)
		}

		clientType, sequence, err := ParseClientIdentifier(client.ClientId)
		if err != nil {
			return err
		}

		if clientType != clientState.ClientType() {
			return fmt.Errorf("client state type %s does not equal client type in client identifier %s", clientState.ClientType(), clientType)
		}

		if err := ValidateClientType(clientType); err != nil {
			return err
		}

		if sequence > maxSequence {
			maxSequence = sequence
		}

		// add client id to validClients map
		validClients[client.ClientId] = clientState.ClientType()
	}

	for _, cc := range gs.ClientsConsensus {
		// check that consensus state is for a client in the genesis clients list
		clientType, ok := validClients[cc.ClientId]
		if !ok {
			return fmt.Errorf("consensus state in genesis has a client id %s that does not map to a genesis client", cc.ClientId)
		}

		for i, consensusState := range cc.ConsensusStates {
			if consensusState.Height.IsZero() {
				return fmt.Errorf("consensus state height cannot be zero")
			}

			cs, ok := consensusState.ConsensusState.GetCachedValue().(exported.ConsensusState)
			if !ok {
				return fmt.Errorf("invalid consensus state with client ID %s at height %s", cc.ClientId, consensusState.Height)
			}

			if err := cs.ValidateBasic(); err != nil {
				return fmt.Errorf("invalid client consensus state %v clientID %s index %d: %w", cs, cc.ClientId, i, err)
			}

			// ensure consensus state type matches client state type
			if clientType != cs.ClientType() {
				return fmt.Errorf("consensus state client type %s does not equal client state client type %s", cs.ClientType(), clientType)
			}

		}
	}

	for _, clientMetadata := range gs.ClientsMetadata {
		// check that metadata is for a client in the genesis clients list
		_, ok := validClients[clientMetadata.ClientId]
		if !ok {
			return fmt.Errorf("metadata in genesis has a client id %s that does not map to a genesis client", clientMetadata.ClientId)
		}

		for i, gm := range clientMetadata.ClientMetadata {
			if err := gm.Validate(); err != nil {
				return fmt.Errorf("invalid client metadata %v clientID %s index %d: %w", gm, clientMetadata.ClientId, i, err)
			}
		}

	}

	if maxSequence != 0 && maxSequence >= gs.NextClientSequence {
		return fmt.Errorf("next client identifier sequence %d must be greater than the maximum sequence used in the provided client identifiers %d", gs.NextClientSequence, maxSequence)
	}

	return nil
}

// NewGenesisMetadata is a constructor for GenesisMetadata
func NewGenesisMetadata(key, val []byte) GenesisMetadata {
	return GenesisMetadata{
		Key:   key,
		Value: val,
	}
}

// GetKey returns the key of metadata. Implements exported.GenesisMetadata interface.
func (gm GenesisMetadata) GetKey() []byte {
	return gm.Key
}

// GetValue returns the value of metadata. Implements exported.GenesisMetadata interface.
func (gm GenesisMetadata) GetValue() []byte {
	return gm.Value
}

// Validate ensures key and value of metadata are not empty
func (gm GenesisMetadata) Validate() error {
	if len(gm.Key) == 0 {
		return fmt.Errorf("genesis metadata key cannot be empty")
	}
	if len(gm.Value) == 0 {
		return fmt.Errorf("genesis metadata value cannot be empty")
	}
	return nil
}

// NewIdentifiedGenesisMetadata takes in a client ID and list of genesis metadata for that client
// and constructs a new IdentifiedGenesisMetadata.
func NewIdentifiedGenesisMetadata(clientID string, gms []GenesisMetadata) IdentifiedGenesisMetadata {
	return IdentifiedGenesisMetadata{
		ClientId:       clientID,
		ClientMetadata: gms,
	}
}
