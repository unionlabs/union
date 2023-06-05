package keeper

import (
	"encoding/hex"
	"io"

	errorsmod "cosmossdk.io/errors"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	snapshot "github.com/cosmos/cosmos-sdk/snapshots/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/ioutils"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var _ snapshot.ExtensionSnapshotter = &WasmSnapshotter{}

// SnapshotFormat format 1 is just gzipped wasm byte code for each item payload. No protobuf envelope, no metadata.
const SnapshotFormat = 1

type WasmSnapshotter struct {
	wasm *Keeper
	cms  sdk.MultiStore
}

func NewWasmSnapshotter(cms sdk.MultiStore, wasm *Keeper) *WasmSnapshotter {
	return &WasmSnapshotter{
		wasm: wasm,
		cms:  cms,
	}
}

func (ws *WasmSnapshotter) SnapshotName() string {
	return types.ModuleName
}

func (ws *WasmSnapshotter) SnapshotFormat() uint32 {
	return SnapshotFormat
}

func (ws *WasmSnapshotter) SupportedFormats() []uint32 {
	// If we support older formats, add them here and handle them in Restore
	return []uint32{SnapshotFormat}
}

func (ws *WasmSnapshotter) SnapshotExtension(height uint64, payloadWriter snapshot.ExtensionPayloadWriter) error {
	cacheMS, err := ws.cms.CacheMultiStoreWithVersion(int64(height))
	if err != nil {
		return err
	}

	ctx := sdk.NewContext(cacheMS, tmproto.Header{}, false, log.NewNopLogger())
	seenBefore := make(map[string]bool)
	var rerr error

	ws.wasm.IterateCodeInfos(ctx, func(id uint64, info types.CodeInfo) bool {
		// Many code ids may point to the same code hash... only sync it once
		hexHash := hex.EncodeToString(info.CodeHash)
		// if seenBefore, just skip this one and move to the next
		if seenBefore[hexHash] {
			return false
		}
		seenBefore[hexHash] = true

		// load code and abort on error
		wasmBytes, err := ws.wasm.GetByteCode(ctx, id)
		if err != nil {
			rerr = err
			return true
		}

		compressedWasm, err := ioutils.GzipIt(wasmBytes)
		if err != nil {
			rerr = err
			return true
		}

		err = payloadWriter(compressedWasm)
		if err != nil {
			rerr = err
			return true
		}

		return false
	})

	return rerr
}

func (ws *WasmSnapshotter) RestoreExtension(height uint64, format uint32, payloadReader snapshot.ExtensionPayloadReader) error {
	if format == SnapshotFormat {
		return ws.processAllItems(height, payloadReader, restoreV1, finalizeV1)
	}
	return snapshot.ErrUnknownFormat
}

func restoreV1(_ sdk.Context, k *Keeper, compressedCode []byte) error {
	if !ioutils.IsGzip(compressedCode) {
		return types.ErrInvalid.Wrap("not a gzip")
	}
	wasmCode, err := ioutils.Uncompress(compressedCode, uint64(types.MaxWasmSize))
	if err != nil {
		return errorsmod.Wrap(types.ErrCreateFailed, err.Error())
	}

	// FIXME: check which codeIDs the checksum matches??
	_, err = k.wasmVM.Create(wasmCode)
	if err != nil {
		return errorsmod.Wrap(types.ErrCreateFailed, err.Error())
	}
	return nil
}

func finalizeV1(ctx sdk.Context, k *Keeper) error {
	// FIXME: ensure all codes have been uploaded?
	return k.InitializePinnedCodes(ctx)
}

func (ws *WasmSnapshotter) processAllItems(
	height uint64,
	payloadReader snapshot.ExtensionPayloadReader,
	cb func(sdk.Context, *Keeper, []byte) error,
	finalize func(sdk.Context, *Keeper) error,
) error {
	ctx := sdk.NewContext(ws.cms, tmproto.Header{Height: int64(height)}, false, log.NewNopLogger())
	for {
		payload, err := payloadReader()
		if err == io.EOF {
			break
		} else if err != nil {
			return err
		}

		if err := cb(ctx, ws.wasm, payload); err != nil {
			return errorsmod.Wrap(err, "processing snapshot item")
		}
	}

	return finalize(ctx, ws.wasm)
}
