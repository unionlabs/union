package debug

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"

	cfg "github.com/cometbft/cometbft/config"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
)

// dumpStatus gets node status state dump from the CometBFT RPC and writes it
// to file. It returns an error upon failure.
func dumpStatus(rpc *rpchttp.HTTP, dir, filename string) error {
	status, err := rpc.Status(context.Background())
	if err != nil {
		return fmt.Errorf("failed to get node status: %w", err)
	}

	return writeStateJSONToFile(status, dir, filename)
}

// dumpNetInfo gets network information state dump from the CometBFT RPC and
// writes it to file. It returns an error upon failure.
func dumpNetInfo(rpc *rpchttp.HTTP, dir, filename string) error {
	netInfo, err := rpc.NetInfo(context.Background())
	if err != nil {
		return fmt.Errorf("failed to get node network information: %w", err)
	}

	return writeStateJSONToFile(netInfo, dir, filename)
}

// dumpConsensusState gets consensus state dump from the CometBFT RPC and
// writes it to file. It returns an error upon failure.
func dumpConsensusState(rpc *rpchttp.HTTP, dir, filename string) error {
	consDump, err := rpc.DumpConsensusState(context.Background())
	if err != nil {
		return fmt.Errorf("failed to get node consensus dump: %w", err)
	}

	return writeStateJSONToFile(consDump, dir, filename)
}

// copyWAL copies the CometBFT node's WAL file. It returns an error if the
// WAL file cannot be read or copied.
func copyWAL(conf *cfg.Config, dir string) error {
	walPath := conf.Consensus.WalFile()
	walFile := filepath.Base(walPath)

	return copyFile(walPath, filepath.Join(dir, walFile))
}

// copyConfig copies the CometBFT node's config file. It returns an error if
// the config file cannot be read or copied.
func copyConfig(home, dir string) error {
	configFile := "config.toml"
	configPath := filepath.Join(home, "config", configFile)

	return copyFile(configPath, filepath.Join(dir, configFile))
}

func dumpProfile(dir, addr, profile string, debug int) error {
	endpoint := fmt.Sprintf("%s/debug/pprof/%s?debug=%d", addr, profile, debug)

	//nolint:gosec,nolintlint,noctx
	resp, err := http.Get(endpoint)
	if err != nil {
		return fmt.Errorf("failed to query for %s profile: %w", profile, err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read %s profile response body: %w", profile, err)
	}

	return os.WriteFile(filepath.Join(dir, profile+".out"), body, os.ModePerm)
}
