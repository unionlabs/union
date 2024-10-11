package types

import (
	"errors"
	"time"
)

// DefaultIndex is the default capability global index.
const DefaultIndex uint64 = 1

func NewGenesisState(epochs []EpochInfo) *GenesisState {
	return &GenesisState{Epochs: epochs}
}

// DefaultGenesis returns the default Capability genesis state.
func DefaultGenesis() *GenesisState {
	epochs := []EpochInfo{
		NewGenesisEpochInfo("day", time.Hour*24), // alphabetical order
		NewGenesisEpochInfo("hour", time.Hour),
		NewGenesisEpochInfo("minute", time.Minute),
		NewGenesisEpochInfo("week", time.Hour*24*7),
	}
	return NewGenesisState(epochs)
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (gs GenesisState) Validate() error {
	epochIdentifiers := map[string]bool{}
	for _, epoch := range gs.Epochs {
		if err := epoch.Validate(); err != nil {
			return err
		}
		if epochIdentifiers[epoch.Identifier] {
			return errors.New("epoch identifier should be unique")
		}
		epochIdentifiers[epoch.Identifier] = true
	}
	return nil
}

// Validate also validates epoch info.
func (epoch EpochInfo) Validate() error {
	if epoch.Identifier == "" {
		return errors.New("epoch identifier should NOT be empty")
	}
	if epoch.Duration == 0 {
		return errors.New("epoch duration should NOT be 0")
	}
	if epoch.CurrentEpoch < 0 {
		return errors.New("epoch CurrentEpoch must be non-negative")
	}
	if epoch.CurrentEpochStartHeight < 0 {
		return errors.New("epoch CurrentEpochStartHeight must be non-negative")
	}
	return nil
}

func NewGenesisEpochInfo(identifier string, duration time.Duration) EpochInfo {
	return EpochInfo{
		Identifier:              identifier,
		StartTime:               time.Time{},
		Duration:                duration,
		CurrentEpoch:            0,
		CurrentEpochStartHeight: 0,
		CurrentEpochStartTime:   time.Time{},
		EpochCountingStarted:    false,
	}
}
