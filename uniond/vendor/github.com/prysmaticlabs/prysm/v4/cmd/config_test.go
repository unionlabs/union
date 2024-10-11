package cmd

import (
	"flag"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/urfave/cli/v2"
)

func TestOverrideConfig(t *testing.T) {
	cfg := &Flags{
		MinimalConfig: true,
	}
	reset := InitWithReset(cfg)
	defer reset()
	c := Get()
	assert.Equal(t, true, c.MinimalConfig)
}

func TestDefaultConfig(t *testing.T) {
	cfg := &Flags{
		MaxRPCPageSize: params.BeaconConfig().DefaultPageSize,
	}
	c := Get()
	assert.DeepEqual(t, c, cfg)

	reset := InitWithReset(cfg)
	defer reset()
	c = Get()
	assert.DeepEqual(t, c, cfg)
}

func TestConfigureBeaconConfig(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.Bool(MinimalConfigFlag.Name, true, "test")
	context := cli.NewContext(&app, set, nil)
	require.NoError(t, ConfigureBeaconChain(context))
	c := Get()
	assert.Equal(t, true, c.MinimalConfig)
}
