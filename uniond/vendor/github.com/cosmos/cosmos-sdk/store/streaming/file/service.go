package file

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"path"
	"sort"
	"sync"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ baseapp.StreamingService = &StreamingService{}

// StreamingService is a concrete implementation of StreamingService that writes
// state changes out to files.
type StreamingService struct {
	storeListeners []*types.MemoryListener // a series of KVStore listeners for each KVStore
	filePrefix     string                  // optional prefix for each of the generated files
	writeDir       string                  // directory to write files into
	codec          codec.BinaryCodec       // marshaller used for re-marshalling the ABCI messages to write them out to the destination files
	logger         log.Logger

	currentBlockNumber int64
	blockMetadata      types.BlockMetadata

	// outputMetadata, if true, writes additional metadata to file per block
	outputMetadata bool

	// stopNodeOnErr, if true, will panic and stop the node during ABCI Commit
	// to ensure eventual consistency of the output, otherwise, any errors are
	// logged and ignored which could yield data loss in streamed output.
	stopNodeOnErr bool

	// fsync, if true, will execute file Sync to make sure the data is persisted
	// onto disk, otherwise there is a risk of data loss during any crash.
	fsync bool
}

func NewStreamingService(
	writeDir, filePrefix string,
	storeKeys []types.StoreKey,
	cdc codec.BinaryCodec,
	logger log.Logger,
	outputMetadata, stopNodeOnErr, fsync bool,
) (*StreamingService, error) {
	// sort storeKeys for deterministic output
	sort.SliceStable(storeKeys, func(i, j int) bool {
		return storeKeys[i].Name() < storeKeys[j].Name()
	})

	// NOTE: We use the same listener for each store.
	listeners := make([]*types.MemoryListener, len(storeKeys))
	for i, key := range storeKeys {
		listeners[i] = types.NewMemoryListener(key)
	}

	// Check that the writeDir exists and is writable so that we can catch the
	// error here at initialization. If it is not we don't open a dstFile until we
	// receive our first ABCI message.
	if err := isDirWriteable(writeDir); err != nil {
		return nil, err
	}

	return &StreamingService{
		storeListeners: listeners,
		filePrefix:     filePrefix,
		writeDir:       writeDir,
		codec:          cdc,
		logger:         logger,
		outputMetadata: outputMetadata,
		stopNodeOnErr:  stopNodeOnErr,
		fsync:          fsync,
	}, nil
}

// Listeners satisfies the StreamingService interface. It returns the
// StreamingService's underlying WriteListeners. Use for registering the
// underlying WriteListeners with the BaseApp.
func (fss *StreamingService) Listeners() map[types.StoreKey][]types.WriteListener {
	listeners := make(map[types.StoreKey][]types.WriteListener, len(fss.storeListeners))
	for _, listener := range fss.storeListeners {
		listeners[listener.StoreKey()] = []types.WriteListener{listener}
	}

	return listeners
}

// ListenBeginBlock satisfies the ABCIListener interface. It sets the received
// BeginBlock request, response and the current block number. Note, these are
// not written to file until ListenCommit is executed and outputMetadata is set,
// after which it will be reset again on the next block.
func (fss *StreamingService) ListenBeginBlock(ctx context.Context, req abci.RequestBeginBlock, res abci.ResponseBeginBlock) error {
	fss.blockMetadata.RequestBeginBlock = &req
	fss.blockMetadata.ResponseBeginBlock = &res
	fss.currentBlockNumber = req.Header.Height
	return nil
}

// ListenDeliverTx satisfies the ABCIListener interface. It appends the received
// DeliverTx request and response to a list of DeliverTxs objects. Note, these
// are not written to file until ListenCommit is executed and outputMetadata is
// set, after which it will be reset again on the next block.
func (fss *StreamingService) ListenDeliverTx(ctx context.Context, req abci.RequestDeliverTx, res abci.ResponseDeliverTx) error {
	fss.blockMetadata.DeliverTxs = append(fss.blockMetadata.DeliverTxs, &types.BlockMetadata_DeliverTx{
		Request:  &req,
		Response: &res,
	})

	return nil
}

// ListenEndBlock satisfies the ABCIListener interface. It sets the received
// EndBlock request, response and the current block number. Note, these are
// not written to file until ListenCommit is executed and outputMetadata is set,
// after which it will be reset again on the next block.
func (fss *StreamingService) ListenEndBlock(ctx context.Context, req abci.RequestEndBlock, res abci.ResponseEndBlock) error {
	fss.blockMetadata.RequestEndBlock = &req
	fss.blockMetadata.ResponseEndBlock = &res
	return nil
}

// ListenCommit satisfies the ABCIListener interface. It is executed during the
// ABCI Commit request and is responsible for writing all staged data to files.
// It will only return a non-nil error when stopNodeOnErr is set.
func (fss *StreamingService) ListenCommit(ctx context.Context, res abci.ResponseCommit) error {
	if err := fss.doListenCommit(ctx, res); err != nil {
		fss.logger.Error("Listen commit failed", "height", fss.currentBlockNumber, "err", err)
		if fss.stopNodeOnErr {
			return err
		}
	}

	return nil
}

func (fss *StreamingService) doListenCommit(ctx context.Context, res abci.ResponseCommit) (err error) {
	fss.blockMetadata.ResponseCommit = &res

	// Write to target files, the file size is written at the beginning, which can
	// be used to detect completeness.
	metaFileName := fmt.Sprintf("block-%d-meta", fss.currentBlockNumber)
	dataFileName := fmt.Sprintf("block-%d-data", fss.currentBlockNumber)

	if fss.filePrefix != "" {
		metaFileName = fmt.Sprintf("%s-%s", fss.filePrefix, metaFileName)
		dataFileName = fmt.Sprintf("%s-%s", fss.filePrefix, dataFileName)
	}

	if fss.outputMetadata {
		bz, err := fss.codec.Marshal(&fss.blockMetadata)
		if err != nil {
			return err
		}

		if err := writeLengthPrefixedFile(path.Join(fss.writeDir, metaFileName), bz, fss.fsync); err != nil {
			return err
		}
	}

	var buf bytes.Buffer
	if err := fss.writeBlockData(&buf); err != nil {
		return err
	}

	return writeLengthPrefixedFile(path.Join(fss.writeDir, dataFileName), buf.Bytes(), fss.fsync)
}

func (fss *StreamingService) writeBlockData(writer io.Writer) error {
	for _, listener := range fss.storeListeners {
		cache := listener.PopStateCache()

		for i := range cache {
			bz, err := fss.codec.MarshalLengthPrefixed(&cache[i])
			if err != nil {
				return err
			}

			if _, err := writer.Write(bz); err != nil {
				return err
			}
		}
	}

	return nil
}

// Stream satisfies the StreamingService interface. It performs a no-op.
func (fss *StreamingService) Stream(wg *sync.WaitGroup) error { return nil }

// Close satisfies the StreamingService interface. It performs a no-op.
func (fss *StreamingService) Close() error { return nil }

// isDirWriteable checks if dir is writable by writing and removing a file
// to dir. It returns nil if dir is writable. We have to do this as there is no
// platform-independent way of determining if a directory is writeable.
func isDirWriteable(dir string) error {
	f := path.Join(dir, ".touch")
	if err := os.WriteFile(f, []byte(""), 0o600); err != nil {
		return err
	}

	return os.Remove(f)
}

func writeLengthPrefixedFile(path string, data []byte, fsync bool) (err error) {
	var f *os.File
	f, err = os.OpenFile(path, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0o600)
	if err != nil {
		return sdkerrors.Wrapf(err, "open file failed: %s", path)
	}

	defer func() {
		// avoid overriding the real error with file close error
		if err1 := f.Close(); err1 != nil && err == nil {
			err = sdkerrors.Wrapf(err, "close file failed: %s", path)
		}
	}()

	_, err = f.Write(sdk.Uint64ToBigEndian(uint64(len(data))))
	if err != nil {
		return sdkerrors.Wrapf(err, "write length prefix failed: %s", path)
	}

	_, err = f.Write(data)
	if err != nil {
		return sdkerrors.Wrapf(err, "write block data failed: %s", path)
	}

	if fsync {
		err = f.Sync()
		if err != nil {
			return sdkerrors.Wrapf(err, "fsync failed: %s", path)
		}
	}

	return err
}
