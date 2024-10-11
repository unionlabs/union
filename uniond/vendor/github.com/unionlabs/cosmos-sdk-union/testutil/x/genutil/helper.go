package genutil

import (
	"context"
	"fmt"
	"path/filepath"

	cmtcfg "github.com/cometbft/cometbft/config"
	"github.com/spf13/viper"

	corectx "cosmossdk.io/core/context"
	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/types/module"
	genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
)

func ExecInitCmd(mm *module.Manager, home string, cdc codec.Codec) error {
	logger := log.NewNopLogger()
	viper := viper.New()
	cmd := genutilcli.InitCmd(mm)
	cfg, _ := CreateDefaultCometConfig(home)
	err := WriteAndTrackCometConfig(viper, home, cfg)
	if err != nil {
		return err
	}
	clientCtx := client.Context{}.WithCodec(cdc).WithHomeDir(home)

	_, out := testutil.ApplyMockIO(cmd)
	clientCtx = clientCtx.WithOutput(out)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)
	ctx = context.WithValue(ctx, corectx.ViperContextKey, viper)
	ctx = context.WithValue(ctx, corectx.LoggerContextKey, logger)

	cmd.SetArgs([]string{"appnode-test"})

	return cmd.ExecuteContext(ctx)
}

func CreateDefaultCometConfig(rootDir string) (*cmtcfg.Config, error) {
	conf := cmtcfg.DefaultConfig()
	conf.SetRoot(rootDir)
	cmtcfg.EnsureRoot(rootDir)

	if err := conf.ValidateBasic(); err != nil {
		return nil, fmt.Errorf("error in config file: %w", err)
	}

	return conf, nil
}

func WriteAndTrackCometConfig(v *viper.Viper, home string, cfg *cmtcfg.Config) error {
	cmtcfg.WriteConfigFile(filepath.Join(home, "config", "config.toml"), cfg)

	v.Set(flags.FlagHome, home)
	v.SetConfigType("toml")
	v.SetConfigName("config")
	v.AddConfigPath(filepath.Join(home, "config"))
	return v.ReadInConfig()
}

func TrackCometConfig(v *viper.Viper, home string) error {
	v.Set(flags.FlagHome, home)
	v.SetConfigType("toml")
	v.SetConfigName("config")
	v.AddConfigPath(filepath.Join(home, "config"))
	return v.ReadInConfig()
}
