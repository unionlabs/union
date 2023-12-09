package v7

import (
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// MigrateGenesis accepts an exported IBC client genesis file and migrates it to:
//
// - Update solo machine client state protobuf definition (v2 to v3)
// - Remove all solo machine consensus states
// - Remove localhost client
func MigrateGenesis(clientGenState *clienttypes.GenesisState, cdc codec.ProtoCodecMarshaler) (*clienttypes.GenesisState, error) {
	// To prune the client and consensus states, we will create new slices to fill up
	// with information we want to keep.
	var (
		clientsConsensus []clienttypes.ClientConsensusStates
		clients          []clienttypes.IdentifiedClientState
	)

	for _, client := range clientGenState.Clients {
		clientType, _, err := clienttypes.ParseClientIdentifier(client.ClientId)
		if err != nil {
			return nil, err
		}

		switch clientType {
		case exported.Solomachine:
			var clientState ClientState
			if err := cdc.Unmarshal(client.ClientState.Value, &clientState); err != nil {
				return nil, errorsmod.Wrap(err, "failed to unmarshal client state bytes into solo machine client state")
			}

			updatedClientState := migrateSolomachine(clientState)

			protoAny, err := clienttypes.PackClientState(&updatedClientState)
			if err != nil {
				return nil, err
			}

			clients = append(clients, clienttypes.IdentifiedClientState{
				ClientId:    client.ClientId,
				ClientState: protoAny,
			})

		case Localhost:
			// remove localhost client state by not adding client state

		default:
			// add all other client states
			clients = append(clients, client)
		}

		// iterate consensus states by client
		for _, clientConsensusStates := range clientGenState.ClientsConsensus {
			// look for consensus states for the current client
			if clientConsensusStates.ClientId == client.ClientId {
				switch clientType {
				case exported.Solomachine, Localhost:
					// remove all consensus states for the solo machine and localhost
					// do not add to new clientsConsensus

				default:
					// ensure all consensus states added for other client types
					clientsConsensus = append(clientsConsensus, clientConsensusStates)
				}
			}
		}
	}

	clientGenState.Clients = clients
	clientGenState.ClientsConsensus = clientsConsensus
	return clientGenState, nil
}
