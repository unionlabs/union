package simsx

import (
	"math/rand"

	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// WeightSource interface for retrieving weights based on a name and a default value.
type WeightSource interface {
	Get(name string, defaultValue uint32) uint32
}

// WeightSourceFn function adapter that implements WeightSource.
// Example:
//
//	weightSource := WeightSourceFn(func(name string, defaultValue uint32) uint32 {
//	  // implementation code...
//	})
type WeightSourceFn func(name string, defaultValue uint32) uint32

func (f WeightSourceFn) Get(name string, defaultValue uint32) uint32 {
	return f(name, defaultValue)
}

// ParamWeightSource is an adapter to the simtypes.AppParams object. This function returns a WeightSource
// implementation that retrieves weights
// based on a name and a default value. The implementation uses the provided AppParams
// to get or generate the weight value. If the weight value exists in the AppParams,
// it is decoded and returned. Otherwise, the provided ParamSimulator is used to generate
// a random value or default value.
//
// The WeightSource implementation is a WeightSourceFn function adapter that implements
// the WeightSource interface. It takes in a name string and a defaultValue uint32 as
// parameters and returns the weight value as a uint32.
//
// Example Usage:
//
//	appParams := simtypes.AppParams{}
//	// add parameters to appParams
//
//	weightSource := ParamWeightSource(appParams)
//	weightSource.Get("some_weight", 100)
func ParamWeightSource(p simtypes.AppParams) WeightSource {
	return WeightSourceFn(func(name string, defaultValue uint32) uint32 {
		var result uint32
		p.GetOrGenerate("op_weight_"+name, &result, nil, func(_ *rand.Rand) {
			result = defaultValue
		})
		return result
	})
}
