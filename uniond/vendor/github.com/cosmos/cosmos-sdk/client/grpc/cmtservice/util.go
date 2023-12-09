package cmtservice

import (
	cmtprototypes "github.com/cometbft/cometbft/proto/tendermint/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// convertHeader converts CometBFT header to sdk header
func convertHeader(h cmtprototypes.Header) Header {
	return Header{
		Version:            h.Version,
		ChainID:            h.ChainID,
		Height:             h.Height,
		Time:               h.Time,
		LastBlockId:        h.LastBlockId,
		ValidatorsHash:     h.ValidatorsHash,
		NextValidatorsHash: h.NextValidatorsHash,
		ConsensusHash:      h.ConsensusHash,
		AppHash:            h.AppHash,
		DataHash:           h.DataHash,
		EvidenceHash:       h.EvidenceHash,
		LastResultsHash:    h.LastResultsHash,
		LastCommitHash:     h.LastCommitHash,
		ProposerAddress:    sdk.ConsAddress(h.ProposerAddress).String(),
	}
}

// convertBlock converts CometBFT block to sdk block
func convertBlock(cmtblock *cmtprototypes.Block) *Block {
	b := new(Block)

	b.Header = convertHeader(cmtblock.Header)
	b.LastCommit = cmtblock.LastCommit
	b.Data = cmtblock.Data
	b.Evidence = cmtblock.Evidence

	return b
}
