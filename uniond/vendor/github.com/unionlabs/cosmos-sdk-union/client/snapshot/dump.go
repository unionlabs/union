package snapshot

import (
	"archive/tar"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"os"
	"strconv"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/server"
)

// DumpArchiveCmd returns a command to dump the snapshot as portable archive format
func DumpArchiveCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "dump <height> <format>",
		Short: "Dump the snapshot as portable archive format",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			viper := client.GetViperFromCmd(cmd)
			snapshotStore, err := server.GetSnapshotStore(viper)
			if err != nil {
				return err
			}

			output, err := cmd.Flags().GetString("output")
			if err != nil {
				return err
			}

			height, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}
			format, err := strconv.ParseUint(args[1], 10, 32)
			if err != nil {
				return err
			}

			if output == "" {
				output = fmt.Sprintf("%d-%d.tar.gz", height, format)
			}

			snapshot, err := snapshotStore.Get(height, uint32(format))
			if err != nil {
				return err
			}

			if snapshot == nil {
				return errors.New("snapshot doesn't exist")
			}

			bz, err := snapshot.Marshal()
			if err != nil {
				return err
			}

			fp, err := os.Create(output)
			if err != nil {
				return err
			}
			defer fp.Close()

			// since the chunk files are already compressed, we just use fastest compression here
			gzipWriter, err := gzip.NewWriterLevel(fp, gzip.BestSpeed)
			if err != nil {
				return err
			}
			tarWriter := tar.NewWriter(gzipWriter)
			if err := tarWriter.WriteHeader(&tar.Header{
				Name: SnapshotFileName,
				Mode: 0o644,
				Size: int64(len(bz)),
			}); err != nil {
				return fmt.Errorf("failed to write snapshot header to tar: %w", err)
			}
			if _, err := tarWriter.Write(bz); err != nil {
				return fmt.Errorf("failed to write snapshot to tar: %w", err)
			}

			for i := uint32(0); i < snapshot.Chunks; i++ {
				path := snapshotStore.PathChunk(height, uint32(format), i)
				tarName := strconv.FormatUint(uint64(i), 10)
				if err := processChunk(tarWriter, path, tarName); err != nil {
					return err
				}
			}

			if err := tarWriter.Close(); err != nil {
				return fmt.Errorf("failed to close tar writer: %w", err)
			}

			if err := gzipWriter.Close(); err != nil {
				return fmt.Errorf("failed to close gzip writer: %w", err)
			}

			return fp.Close()
		},
	}

	cmd.Flags().StringP("output", "o", "", "output file")

	return cmd
}

func processChunk(tarWriter *tar.Writer, path, tarName string) error {
	file, err := os.Open(path)
	if err != nil {
		return fmt.Errorf("failed to open chunk file %s: %w", path, err)
	}
	defer file.Close()

	st, err := file.Stat()
	if err != nil {
		return fmt.Errorf("failed to stat chunk file %s: %w", path, err)
	}

	if err := tarWriter.WriteHeader(&tar.Header{
		Name: tarName,
		Mode: 0o644,
		Size: st.Size(),
	}); err != nil {
		return fmt.Errorf("failed to write chunk header to tar: %w", err)
	}

	if _, err := io.Copy(tarWriter, file); err != nil {
		return fmt.Errorf("failed to write chunk to tar: %w", err)
	}

	return nil
}
