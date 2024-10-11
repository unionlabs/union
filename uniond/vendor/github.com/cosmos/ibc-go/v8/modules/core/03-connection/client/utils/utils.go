package utils

import (
	"context"
	"errors"
	"fmt"
	"os"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"

	clientutils "github.com/cosmos/ibc-go/v8/modules/core/02-client/client/utils"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcclient "github.com/cosmos/ibc-go/v8/modules/core/client"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// QueryConnection returns a connection end.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryConnection(
	clientCtx client.Context, connectionID string, prove bool,
) (*types.QueryConnectionResponse, error) {
	if prove {
		return queryConnectionABCI(clientCtx, connectionID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryConnectionRequest{
		ConnectionId: connectionID,
	}

	return queryClient.Connection(context.Background(), req)
}

func queryConnectionABCI(clientCtx client.Context, connectionID string) (*types.QueryConnectionResponse, error) {
	key := host.ConnectionKey(connectionID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if connection exists
	if len(value) == 0 {
		return nil, errorsmod.Wrap(types.ErrConnectionNotFound, connectionID)
	}

	cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

	var connection types.ConnectionEnd
	if err := cdc.Unmarshal(value, &connection); err != nil {
		return nil, err
	}

	return types.NewQueryConnectionResponse(connection, proofBz, proofHeight), nil
}

// QueryClientConnections queries the connection paths registered for a particular client.
// If prove is true, it performs an ABCI store query in order to retrieve the merkle proof. Otherwise,
// it uses the gRPC query client.
func QueryClientConnections(
	clientCtx client.Context, clientID string, prove bool,
) (*types.QueryClientConnectionsResponse, error) {
	if prove {
		return queryClientConnectionsABCI(clientCtx, clientID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryClientConnectionsRequest{
		ClientId: clientID,
	}

	return queryClient.ClientConnections(context.Background(), req)
}

func queryClientConnectionsABCI(clientCtx client.Context, clientID string) (*types.QueryClientConnectionsResponse, error) {
	key := host.ClientConnectionsKey(clientID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if connection paths exist
	if len(value) == 0 {
		return nil, errorsmod.Wrap(types.ErrClientConnectionPathsNotFound, clientID)
	}

	var paths []string
	if err := clientCtx.LegacyAmino.Unmarshal(value, &paths); err != nil {
		return nil, err
	}

	return types.NewQueryClientConnectionsResponse(paths, proofBz, proofHeight), nil
}

// QueryConnectionClientState returns the ClientState of a connection end. If
// prove is true, it performs an ABCI store query in order to retrieve the
// merkle proof. Otherwise, it uses the gRPC query client.
func QueryConnectionClientState(
	clientCtx client.Context, connectionID string, prove bool,
) (*types.QueryConnectionClientStateResponse, error) {
	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryConnectionClientStateRequest{
		ConnectionId: connectionID,
	}

	res, err := queryClient.ConnectionClientState(context.Background(), req)
	if err != nil {
		return nil, err
	}

	if prove {
		clientStateRes, err := clientutils.QueryClientStateABCI(clientCtx, res.IdentifiedClientState.ClientId)
		if err != nil {
			return nil, err
		}

		// use client state returned from ABCI query in case query height differs
		identifiedClientState := clienttypes.IdentifiedClientState{
			ClientId:    res.IdentifiedClientState.ClientId,
			ClientState: clientStateRes.ClientState,
		}

		res = types.NewQueryConnectionClientStateResponse(identifiedClientState, clientStateRes.Proof, clientStateRes.ProofHeight)
	}

	return res, nil
}

// QueryConnectionConsensusState returns the ConsensusState of a connection end. If
// prove is true, it performs an ABCI store query in order to retrieve the
// merkle proof. Otherwise, it uses the gRPC query client.
func QueryConnectionConsensusState(
	clientCtx client.Context, connectionID string, height clienttypes.Height, prove bool,
) (*types.QueryConnectionConsensusStateResponse, error) {
	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryConnectionConsensusStateRequest{
		ConnectionId:   connectionID,
		RevisionNumber: height.RevisionNumber,
		RevisionHeight: height.RevisionHeight,
	}

	res, err := queryClient.ConnectionConsensusState(context.Background(), req)
	if err != nil {
		return nil, err
	}

	if prove {
		consensusStateRes, err := clientutils.QueryConsensusStateABCI(clientCtx, res.ClientId, height)
		if err != nil {
			return nil, err
		}

		res = types.NewQueryConnectionConsensusStateResponse(res.ClientId, consensusStateRes.ConsensusState, height, consensusStateRes.Proof, consensusStateRes.ProofHeight)
	}

	return res, nil
}

// ParseClientState unmarshals a cmd input argument from a JSON string to a client state
// If the input is not a JSON, it looks for a path to the JSON file
func ParseClientState(cdc *codec.LegacyAmino, arg string) (exported.ClientState, error) {
	var clientState exported.ClientState
	if err := cdc.UnmarshalJSON([]byte(arg), &clientState); err != nil {
		// check for file path if JSON input is not provided
		contents, err := os.ReadFile(arg)
		if err != nil {
			return nil, errors.New("either JSON input nor path to .json file were provided")
		}
		if err := cdc.UnmarshalJSON(contents, &clientState); err != nil {
			return nil, fmt.Errorf("error unmarshalling client state: %w", err)
		}
	}
	return clientState, nil
}

// ParsePrefix unmarshals an cmd input argument from a JSON string to a commitment
// Prefix. If the input is not a JSON, it looks for a path to the JSON file.
func ParsePrefix(cdc *codec.LegacyAmino, arg string) (commitmenttypes.MerklePrefix, error) {
	var prefix commitmenttypes.MerklePrefix
	if err := cdc.UnmarshalJSON([]byte(arg), &prefix); err != nil {
		// check for file path if JSON input is not provided
		contents, err := os.ReadFile(arg)
		if err != nil {
			return commitmenttypes.MerklePrefix{}, errors.New("neither JSON input nor path to .json file were provided")
		}
		if err := cdc.UnmarshalJSON(contents, &prefix); err != nil {
			return commitmenttypes.MerklePrefix{}, fmt.Errorf("error unmarshalling commitment prefix: %w", err)
		}
	}
	return prefix, nil
}

// ParseProof unmarshals a cmd input argument from a JSON string to a commitment
// Proof. If the input is not a JSON, it looks for a path to the JSON file. It
// then marshals the commitment proof into a proto encoded byte array.
func ParseProof(cdc *codec.LegacyAmino, arg string) ([]byte, error) {
	var merkleProof commitmenttypes.MerkleProof
	if err := cdc.UnmarshalJSON([]byte(arg), &merkleProof); err != nil {
		// check for file path if JSON input is not provided
		contents, err := os.ReadFile(arg)
		if err != nil {
			return nil, errors.New("neither JSON input nor path to .json file were provided")
		}
		if err := cdc.UnmarshalJSON(contents, &merkleProof); err != nil {
			return nil, fmt.Errorf("error unmarshalling commitment proof: %w", err)
		}
	}

	return cdc.Marshal(&merkleProof)
}
