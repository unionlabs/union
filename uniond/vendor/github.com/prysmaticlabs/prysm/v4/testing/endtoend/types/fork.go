package types

import (
	"fmt"
	"math"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

func InitForkCfg(start, end int, c *params.BeaconChainConfig) *params.BeaconChainConfig {
	c = c.Copy()
	if end < start {
		panic("end fork is less than the start fork")
	}
	if start >= version.Altair {
		c.AltairForkEpoch = 0
	}
	if start >= version.Bellatrix {
		c.BellatrixForkEpoch = 0
	}
	if start >= version.Capella {
		c.CapellaForkEpoch = 0
	}
	if start >= version.Deneb {
		c.DenebForkEpoch = 0
	}
	if end < version.Deneb {
		c.DenebForkEpoch = math.MaxUint64
	}
	if end < version.Capella {
		c.CapellaForkEpoch = math.MaxUint64
	}
	if end < version.Bellatrix {
		c.BellatrixForkEpoch = math.MaxUint64
	}
	if end < version.Altair {
		c.AltairForkEpoch = math.MaxUint64
	}
	// Time TTD to line up roughly with the bellatrix fork epoch.
	// E2E sets EL block production rate equal to SecondsPerETH1Block to keep the math simple.
	ttd := uint64(c.BellatrixForkEpoch) * uint64(c.SlotsPerEpoch) * c.SecondsPerSlot
	c.TerminalTotalDifficulty = fmt.Sprintf("%d", ttd)
	return c
}
