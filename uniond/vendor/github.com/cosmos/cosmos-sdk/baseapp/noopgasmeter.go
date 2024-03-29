package baseapp

import storetypes "cosmossdk.io/store/types"

type noopGasMeter struct{}

var _ storetypes.GasMeter = noopGasMeter{}

func (noopGasMeter) GasConsumed() storetypes.Gas        { return 0 }
func (noopGasMeter) GasConsumedToLimit() storetypes.Gas { return 0 }
func (noopGasMeter) GasRemaining() storetypes.Gas       { return 0 }
func (noopGasMeter) Limit() storetypes.Gas              { return 0 }
func (noopGasMeter) ConsumeGas(storetypes.Gas, string)  {}
func (noopGasMeter) RefundGas(storetypes.Gas, string)   {}
func (noopGasMeter) IsPastLimit() bool                  { return false }
func (noopGasMeter) IsOutOfGas() bool                   { return false }
func (noopGasMeter) String() string                     { return "noopGasMeter" }
