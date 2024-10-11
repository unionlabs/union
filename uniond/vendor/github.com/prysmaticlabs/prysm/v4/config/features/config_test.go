package features

import (
	"flag"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/urfave/cli/v2"
)

func TestInitFeatureConfig(t *testing.T) {
	defer Init(&Flags{})
	cfg := &Flags{
		EnableSlasher: true,
	}
	Init(cfg)
	c := Get()
	assert.Equal(t, true, c.EnableSlasher)
}

func TestInitWithReset(t *testing.T) {
	defer Init(&Flags{})
	Init(&Flags{
		EnableSlasher: true,
	})
	assert.Equal(t, true, Get().EnableSlasher)

	// Overwrite previously set value (value that didn't come by default).
	resetCfg := InitWithReset(&Flags{
		EnableSlasher: false,
	})
	assert.Equal(t, false, Get().EnableSlasher)

	// Reset must get to previously set configuration (not to default config values).
	resetCfg()
	assert.Equal(t, true, Get().EnableSlasher)
}

func TestConfigureBeaconConfig(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.Bool(enableSlasherFlag.Name, true, "test")
	context := cli.NewContext(&app, set, nil)
	require.NoError(t, ConfigureBeaconChain(context))
	c := Get()
	assert.Equal(t, true, c.EnableSlasher)
}
