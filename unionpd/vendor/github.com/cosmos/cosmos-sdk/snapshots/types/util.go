package types

import (
	protoio "github.com/cosmos/gogoproto/io"
)

// WriteExtensionPayload writes an extension payload for current extension snapshotter.
func WriteExtensionPayload(protoWriter protoio.Writer, payload []byte) error {
	return protoWriter.WriteMsg(&SnapshotItem{
		Item: &SnapshotItem_ExtensionPayload{
			ExtensionPayload: &SnapshotExtensionPayload{
				Payload: payload,
			},
		},
	})
}
