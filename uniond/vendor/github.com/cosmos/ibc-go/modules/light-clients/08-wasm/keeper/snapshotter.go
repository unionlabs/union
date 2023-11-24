package keeper

import (
	"encoding/hex"
	"io"

	errorsmod "cosmossdk.io/errors"
	snapshot "cosmossdk.io/store/snapshots/types"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

var _ snapshot.ExtensionSnapshotter = &WasmSnapshotter{}

// SnapshotFormat defines the default snapshot extension encoding format.
// SnapshotFormat 1 is gzipped wasm byte code for each item payload. No protobuf envelope, no metadata.
const SnapshotFormat = 1

// WasmSnapshotter implements the snapshot.ExtensionSnapshotter interface and is used to
// import and export state maintained within the wasmvm cache.
// NOTE: The following ExtensionSnapshotter has been adapted from CosmWasm's x/wasm:
// https://github.com/CosmWasm/wasmd/blob/v0.43.0/x/wasm/keeper/snapshotter.go
type WasmSnapshotter struct {
	cms    storetypes.MultiStore
	keeper *Keeper
}

// NewWasmSnapshotter creates and returns a new snapshot.ExtensionSnapshotter implementation for the 08-wasm module.
func NewWasmSnapshotter(cms storetypes.MultiStore, keeper *Keeper) snapshot.ExtensionSnapshotter {
	return &WasmSnapshotter{
		cms:    cms,
		keeper: keeper,
	}
}

// SnapshotName implements the snapshot.ExtensionSnapshotter interface.
// A unique name should be provided such that the implementation can be identified by the manager.
func (*WasmSnapshotter) SnapshotName() string {
	return types.ModuleName
}

// SnapshotFormat implements the snapshot.ExtensionSnapshotter interface.
// This is the default format used for encoding payloads when taking a snapshot.
func (*WasmSnapshotter) SnapshotFormat() uint32 {
	return SnapshotFormat
}

// SupportedFormats implements the snapshot.ExtensionSnapshotter interface.
// This defines a list of supported formats the snapshotter extension can restore from.
func (*WasmSnapshotter) SupportedFormats() []uint32 {
	return []uint32{SnapshotFormat}
}

// SnapshotExtension implements the snapshot.ExntensionSnapshotter interface.
// SnapshotExtension is used to write data payloads into the underlying protobuf stream from the 08-wasm module.
func (ws *WasmSnapshotter) SnapshotExtension(height uint64, payloadWriter snapshot.ExtensionPayloadWriter) error {
	cacheMS, err := ws.cms.CacheMultiStoreWithVersion(int64(height))
	if err != nil {
		return err
	}

	ctx := sdk.NewContext(cacheMS, cmtproto.Header{}, false, nil)

	checksums, err := types.GetAllChecksums(ctx)
	if err != nil {
		return err
	}

	for _, checksum := range checksums {
		wasmCode, err := ibcwasm.GetVM().GetCode(checksum)
		if err != nil {
			return err
		}

		compressedWasm, err := types.GzipIt(wasmCode)
		if err != nil {
			return err
		}

		if err = payloadWriter(compressedWasm); err != nil {
			return err
		}
	}

	return nil
}

// RestoreExtension implements the snapshot.ExtensionSnapshotter interface.
// RestoreExtension is used to read data from an existing extension state snapshot into the 08-wasm module.
// The payload reader returns io.EOF when it has reached the end of the extension state snapshot.
func (ws *WasmSnapshotter) RestoreExtension(height uint64, format uint32, payloadReader snapshot.ExtensionPayloadReader) error {
	if format == ws.SnapshotFormat() {
		return ws.processAllItems(height, payloadReader, restoreV1)
	}

	return errorsmod.Wrapf(snapshot.ErrUnknownFormat, "expected %d, got %d", ws.SnapshotFormat(), format)
}

func restoreV1(ctx sdk.Context, k *Keeper, compressedCode []byte) error {
	if !types.IsGzip(compressedCode) {
		return errorsmod.Wrap(types.ErrInvalidData, "expected wasm code is not gzip format")
	}

	wasmCode, err := types.Uncompress(compressedCode, types.MaxWasmByteSize())
	if err != nil {
		return errorsmod.Wrap(err, "failed to uncompress wasm code")
	}

	checksum, err := ibcwasm.GetVM().StoreCode(wasmCode)
	if err != nil {
		return errorsmod.Wrap(err, "failed to store wasm code")
	}

	if err := ibcwasm.GetVM().Pin(checksum); err != nil {
		return errorsmod.Wrapf(err, "failed to pin checksum: %s to in-memory cache", hex.EncodeToString(checksum))
	}

	return nil
}

func (ws *WasmSnapshotter) processAllItems(
	height uint64,
	payloadReader snapshot.ExtensionPayloadReader,
	cb func(sdk.Context, *Keeper, []byte) error,
) error {
	ctx := sdk.NewContext(ws.cms, cmtproto.Header{Height: int64(height)}, false, nil)
	for {
		payload, err := payloadReader()
		if err == io.EOF {
			break
		} else if err != nil {
			return err
		}

		if err := cb(ctx, ws.keeper, payload); err != nil {
			return errorsmod.Wrap(err, "failure processing snapshot item")
		}
	}

	return nil
}
