package nft

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ValidateGenesis checks that the given genesis state has no integrity issues
func ValidateGenesis(data GenesisState) error {
	for _, class := range data.Classes {
		if len(class.Id) == 0 {
			return ErrEmptyClassID
		}
	}
	for _, entry := range data.Entries {
		for _, nft := range entry.Nfts {
			if len(nft.Id) == 0 {
				return ErrEmptyNFTID
			}
			if _, err := sdk.AccAddressFromBech32(entry.Owner); err != nil {
				return err
			}
		}
	}
	return nil
}

// DefaultGenesisState - Returns a default genesis state
func DefaultGenesisState() *GenesisState {
	return &GenesisState{}
}
