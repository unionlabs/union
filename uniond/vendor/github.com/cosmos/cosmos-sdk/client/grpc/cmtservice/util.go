package cmtservice

import (
	cmtprototypes "github.com/cometbft/cometbft/api/cometbft/types/v1"

	"cosmossdk.io/core/address"
)

// convertHeader converts CometBFT header to sdk header
func convertHeader(h cmtprototypes.Header, ac address.Codec) (Header, error) {
	proposerAddr, err := ac.BytesToString(h.ProposerAddress)
	if err != nil {
		return Header{}, err
	}

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
		ProposerAddress:    proposerAddr,
	}, nil
}

// convertBlock converts CometBFT block to sdk block
func convertBlock(cmtblock *cmtprototypes.Block, ac address.Codec) (*Block, error) {
	b := new(Block)
	var err error
	b.Header, err = convertHeader(cmtblock.Header, ac)
	if err != nil {
		return nil, err
	}
	b.LastCommit = cmtblock.LastCommit
	b.Data = cmtblock.Data
	b.Evidence = cmtblock.Evidence

	return b, nil
}
