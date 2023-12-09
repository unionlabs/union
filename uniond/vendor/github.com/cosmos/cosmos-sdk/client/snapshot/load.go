package snapshot

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"fmt"
	"io"
	"os"
	"reflect"
	"strconv"

	"github.com/spf13/cobra"

	snapshottypes "cosmossdk.io/store/snapshots/types"

	"github.com/cosmos/cosmos-sdk/server"
)

const SnapshotFileName = "_snapshot"

// LoadArchiveCmd load a portable archive format snapshot into snapshot store
func LoadArchiveCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "load <archive-file>",
		Short: "Load a snapshot archive file (.tar.gz) into snapshot store",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := server.GetServerContextFromCmd(cmd)
			snapshotStore, err := server.GetSnapshotStore(ctx.Viper)
			if err != nil {
				return err
			}

			path := args[0]
			fp, err := os.Open(path)
			if err != nil {
				return fmt.Errorf("failed to open archive file: %w", err)
			}
			reader, err := gzip.NewReader(fp)
			if err != nil {
				return fmt.Errorf("failed to create gzip reader: %w", err)
			}

			var snapshot snapshottypes.Snapshot
			tr := tar.NewReader(reader)
			if err != nil {
				return fmt.Errorf("failed to create tar reader: %w", err)
			}

			hdr, err := tr.Next()
			if err != nil {
				return fmt.Errorf("failed to read snapshot file header: %w", err)
			}
			if hdr.Name != SnapshotFileName {
				return fmt.Errorf("invalid archive, expect file: snapshot, got: %s", hdr.Name)
			}
			bz, err := io.ReadAll(tr)
			if err != nil {
				return fmt.Errorf("failed to read snapshot file: %w", err)
			}
			if err := snapshot.Unmarshal(bz); err != nil {
				return fmt.Errorf("failed to unmarshal snapshot: %w", err)
			}

			// make sure the channel is unbuffered, because the tar reader can't do concurrency
			chunks := make(chan io.ReadCloser)
			quitChan := make(chan *snapshottypes.Snapshot)
			go func() {
				defer close(quitChan)

				savedSnapshot, err := snapshotStore.Save(snapshot.Height, snapshot.Format, chunks)
				if err != nil {
					cmd.Println("failed to save snapshot", err)
					return
				}
				quitChan <- savedSnapshot
			}()

			for i := uint32(0); i < snapshot.Chunks; i++ {
				hdr, err = tr.Next()
				if err != nil {
					if err == io.EOF {
						break
					}
					return err
				}

				if hdr.Name != strconv.FormatInt(int64(i), 10) {
					return fmt.Errorf("invalid archive, expect file: %d, got: %s", i, hdr.Name)
				}

				bz, err := io.ReadAll(tr)
				if err != nil {
					return fmt.Errorf("failed to read chunk file: %w", err)
				}
				chunks <- io.NopCloser(bytes.NewReader(bz))
			}
			close(chunks)

			savedSnapshot := <-quitChan
			if savedSnapshot == nil {
				return fmt.Errorf("failed to save snapshot")
			}

			if !reflect.DeepEqual(&snapshot, savedSnapshot) {
				_ = snapshotStore.Delete(snapshot.Height, snapshot.Format)
				return fmt.Errorf("invalid archive, the saved snapshot is not equal to the original one")
			}

			return nil
		},
	}
}
