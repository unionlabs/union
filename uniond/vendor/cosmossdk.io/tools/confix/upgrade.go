package confix

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/creachadair/atomicfile"
	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/transform"
	"github.com/spf13/viper"

	clientcfg "github.com/cosmos/cosmos-sdk/client/config"
	srvcfg "github.com/cosmos/cosmos-sdk/server/config"
)

// Upgrade reads the configuration file at configPath and applies any
// transformations necessary to Upgrade it to the current version. If this
// succeeds, the transformed output is written to outputPath. As a special
// case, if outputPath == "" the output is written to stdout.
//
// It is safe if outputPath == inputPath. If a regular file outputPath already
// exists, it is overwritten. In case of error, the output is not written.
//
// Upgrade is a convenience wrapper for calls to LoadConfig, ApplyFixes, and
// CheckValid. If the caller requires more control over the behavior of the
// Upgrade, call those functions directly.
func Upgrade(ctx context.Context, plan transform.Plan, doc *tomledit.Document, configPath, outputPath string, skipValidate bool) error {
	if configPath == "" {
		return errors.New("empty input configuration path")
	}

	// transforms doc and reports whether it succeeded.
	if err := plan.Apply(ctx, doc); err != nil {
		return fmt.Errorf("updating %q: %w", configPath, err)
	}

	var buf bytes.Buffer
	if err := tomledit.Format(&buf, doc); err != nil {
		return fmt.Errorf("formatting config: %w", err)
	}

	// ignore validation for serverv2 by checking any default field found in doc
	isServerV2 := doc.First(strings.Split("store.options.ss-pruning-option", ".")...) != nil

	// allow to skip validation
	if !skipValidate && !isServerV2 {
		// verify that file is valid after applying fixes
		if err := CheckValid(configPath, buf.Bytes()); err != nil {
			return fmt.Errorf("updated config is invalid: %w", err)
		}
	}

	var err error
	if outputPath == "" {
		_, err = os.Stdout.Write(buf.Bytes())
	} else {
		err = atomicfile.WriteData(outputPath, buf.Bytes(), 0o600)
	}

	return err
}

// CheckValid checks whether the specified config appears to be a valid Cosmos SDK config file.
// It tries to unmarshal the config into both the server and client config structs.
func CheckValid(fileName string, data []byte) error {
	v := viper.New()
	v.SetConfigType("toml")

	if err := v.ReadConfig(bytes.NewReader(data)); err != nil {
		return fmt.Errorf("reading config: %w", err)
	}

	switch {
	case strings.HasSuffix(fileName, AppConfig):
		var cfg srvcfg.Config
		if err := v.Unmarshal(&cfg); err != nil {
			return fmt.Errorf("failed to unmarshal as server config: %w", err)
		}

		if err := cfg.ValidateBasic(); err != nil {
			return fmt.Errorf("server config invalid: %w", err)
		}
	case strings.HasSuffix(fileName, ClientConfig):
		var cfg clientcfg.ClientConfig
		if err := v.Unmarshal(&cfg); err != nil {
			return fmt.Errorf("failed to unmarshal as client config: %w", err)
		}

		if cfg.ChainID == "" {
			return errors.New("client config invalid: chain-id is empty")
		}
	case strings.HasSuffix(fileName, CMTConfig):
		return errors.New("cometbft config is not supported")

	default:
		return fmt.Errorf("unknown config: %s", fileName)
	}

	return nil
}
