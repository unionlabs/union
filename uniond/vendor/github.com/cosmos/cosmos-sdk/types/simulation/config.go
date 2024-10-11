package simulation

import "testing"

// Config contains the necessary configuration flags for the simulator
type Config struct {
	GenesisFile string // custom simulation genesis file; cannot be used with params file
	ParamsFile  string // custom simulation params file which overrides any random params; cannot be used with genesis

	ExportParamsPath   string // custom file path to save the exported params JSON
	ExportParamsHeight int    // height to which export the randomly generated params
	ExportStatePath    string // custom file path to save the exported app state JSON
	ExportStatsPath    string // custom file path to save the exported simulation statistics JSON

	Seed               int64  // simulation random seed
	InitialBlockHeight int    // initial block to start the simulation
	GenesisTime        int64  // genesis time to start the simulation
	NumBlocks          int    // number of new blocks to simulate from the initial block height
	BlockSize          int    // operations per block
	ChainID            string // chain-id used on the simulation

	Lean   bool // lean simulation log output
	Commit bool // have the simulation commit

	DBBackend   string // custom db backend type
	BlockMaxGas int64  // custom max gas for block
	FuzzSeed    []byte
	TB          testing.TB
	FauxMerkle  bool
}

func (c Config) shallowCopy() Config {
	return c
}

// With sets the values of t, seed, and fuzzSeed in a copy of the Config and returns the copy.
func (c Config) With(tb testing.TB, seed int64, fuzzSeed []byte) Config {
	tb.Helper()
	r := c.shallowCopy()
	r.TB = tb
	r.Seed = seed
	r.FuzzSeed = fuzzSeed
	return r
}
