package types

import (
	"encoding/json"
	"time"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

type (
	exportMetadataInnerPayload struct{}
	exportMetadataPayload      struct {
		ExportMetadata exportMetadataInnerPayload `json:"export_metadata"`
	}
)

// ExportMetadata exports all the consensus metadata in the client store so they
// can be included in clients genesis and imported by a ClientKeeper
func (cs ClientState) ExportMetadata(store sdk.KVStore) []exported.GenesisMetadata {
	var payload exportMetadataPayload

	encodedData, err := json.Marshal(payload)
	if err != nil {
		panic(err)
	}

	ctx := sdk.NewContext(nil, tmproto.Header{Height: 1, Time: time.Now()}, true, nil) // context with infinite gas meter
	response, err := queryContractWithStore(ctx, store, cs.CodeId, encodedData)
	if err != nil {
		panic(err)
	}

	var output queryResponse
	if err := json.Unmarshal(response, &output); err != nil {
		panic(err)
	}

	genesisMetadata := make([]exported.GenesisMetadata, len(output.GenesisMetadata))
	for i, metadata := range output.GenesisMetadata {
		genesisMetadata[i] = metadata
	}

	return genesisMetadata
}
