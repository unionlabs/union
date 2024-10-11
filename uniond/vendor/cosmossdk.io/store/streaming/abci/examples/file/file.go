package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/hashicorp/go-plugin"

	streamingabci "cosmossdk.io/store/streaming/abci"
	store "cosmossdk.io/store/types"
)

// FilePlugin is the implementation of the baseapp.ABCIListener interface
// For Go plugins this is all that is required to process data sent over gRPC.
type FilePlugin struct {
	BlockHeight int64
}

func (a *FilePlugin) writeToFile(file string, data []byte) error {
	home, err := os.UserHomeDir()
	if err != nil {
		return err
	}

	filename := fmt.Sprintf("%s/%s.txt", home, file)
	f, err := os.OpenFile(filepath.Clean(filename), os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o600)
	if err != nil {
		return err
	}

	if _, err := f.Write(data); err != nil {
		f.Close() // ignore error; Write error takes precedence
		return err
	}

	if err := f.Close(); err != nil {
		return err
	}

	return nil
}

func (a *FilePlugin) ListenFinalizeBlock(ctx context.Context, req abci.FinalizeBlockRequest, res abci.FinalizeBlockResponse) error {
	d1 := []byte(fmt.Sprintf("%d:::%v\n", a.BlockHeight, req))
	d2 := []byte(fmt.Sprintf("%d:::%v\n", a.BlockHeight, req))
	if err := a.writeToFile("finalize-block-req", d1); err != nil {
		return err
	}
	if err := a.writeToFile("finalize-block-res", d2); err != nil {
		return err
	}
	return nil
}

func (a *FilePlugin) ListenCommit(ctx context.Context, res abci.CommitResponse, changeSet []*store.StoreKVPair) error {
	fmt.Printf("listen-commit: block_height=%d data=%v", res.RetainHeight, changeSet)
	d1 := []byte(fmt.Sprintf("%d:::%v\n", a.BlockHeight, res))
	d2 := []byte(fmt.Sprintf("%d:::%v\n", a.BlockHeight, changeSet))
	if err := a.writeToFile("commit-res", d1); err != nil {
		return err
	}
	if err := a.writeToFile("state-change", d2); err != nil {
		return err
	}
	return nil
}

func main() {
	plugin.Serve(&plugin.ServeConfig{
		HandshakeConfig: streamingabci.Handshake,
		Plugins: map[string]plugin.Plugin{
			"abci": &streamingabci.ListenerGRPCPlugin{Impl: &FilePlugin{}},
		},

		// A non-nil value here enables gRPC serving for this streaming...
		GRPCServer: plugin.DefaultGRPCServer,
	})
}
