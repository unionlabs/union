package cmd

import (
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/urfave/cli/v2"
)

// Flags is a struct to represent which features the client will perform on runtime.
type Flags struct {
	// Configuration related flags.
	MinimalConfig  bool // MinimalConfig as defined in the spec.
	MaxRPCPageSize int  // MaxRPCPageSize is used for a cap of page sizes in RPC requests.
}

var sharedConfig *Flags

// Get retrieves feature config.
func Get() *Flags {
	if sharedConfig == nil {
		return &Flags{
			MaxRPCPageSize: params.BeaconConfig().DefaultPageSize,
		}
	}
	return sharedConfig
}

// Init sets the global config equal to the config that is passed in.
func Init(c *Flags) {
	sharedConfig = c
}

// InitWithReset sets the global config and returns function that is used to reset configuration.
func InitWithReset(c *Flags) func() {
	resetFunc := func() {
		Init(nil)
	}
	Init(c)
	return resetFunc
}

// ConfigureBeaconChain sets the global config based
// on what flags are enabled for the beacon-chain client.
func ConfigureBeaconChain(ctx *cli.Context) error {
	cfg, err := newConfig(ctx)
	if err != nil {
		return err
	}
	if ctx.IsSet(RPCMaxPageSizeFlag.Name) {
		cfg.MaxRPCPageSize = ctx.Int(RPCMaxPageSizeFlag.Name)
		log.Warnf("Starting beacon chain with max RPC page size of %d", cfg.MaxRPCPageSize)
	}
	Init(cfg)
	return nil
}

// ConfigureValidator sets the global config based
// on what flags are enabled for the validator client.
func ConfigureValidator(ctx *cli.Context) error {
	cfg, err := newConfig(ctx)
	if err != nil {
		return err
	}
	Init(cfg)
	return nil
}

func newConfig(ctx *cli.Context) (*Flags, error) {
	cfg := Get()
	if ctx.Bool(MinimalConfigFlag.Name) {
		log.Warn("Using minimal config")
		cfg.MinimalConfig = true
		if err := params.SetActive(params.MinimalSpecConfig().Copy()); err != nil {
			return nil, err
		}
	}
	if ctx.Bool(E2EConfigFlag.Name) {
		log.Warn("Using end-to-end testing config")
		switch fieldparams.Preset {
		case "mainnet":
			if err := params.SetActive(params.E2EMainnetTestConfig().Copy()); err != nil {
				return nil, err
			}
		case "minimal":
			if err := params.SetActive(params.E2ETestConfig().Copy()); err != nil {
				return nil, err
			}
		default:
			log.Fatalf("Unrecognized preset being used: %s", fieldparams.Preset)
		}
	}
	return cfg, nil
}
