package getter

import (
	"context"
	"fmt"
	"io"
	"os"
)

// readerFunc is syntactic sugar for read interface.
type readerFunc func(p []byte) (n int, err error)

func (rf readerFunc) Read(p []byte) (n int, err error) { return rf(p) }

// Copy is a io.Copy cancellable by context
func Copy(ctx context.Context, dst io.Writer, src io.Reader) (int64, error) {
	// Copy will call the Reader and Writer interface multiple time, in order
	// to copy by chunk (avoiding loading the whole file in memory).
	return io.Copy(dst, readerFunc(func(p []byte) (int, error) {

		select {
		case <-ctx.Done():
			// context has been canceled
			// stop process and propagate "context canceled" error
			return 0, ctx.Err()
		default:
			// otherwise just run default io.Reader implementation
			return src.Read(p)
		}
	}))
}

// copyReader copies from an io.Reader into a file, using umask to create the dst file
func copyReader(dst string, src io.Reader, fmode, umask os.FileMode, fileSizeLimit int64) error {
	dstF, err := os.OpenFile(dst, os.O_RDWR|os.O_CREATE|os.O_TRUNC, fmode)
	if err != nil {
		return err
	}
	defer dstF.Close()

	if fileSizeLimit > 0 {
		src = io.LimitReader(src, fileSizeLimit)
	}

	_, err = io.Copy(dstF, src)
	if err != nil {
		return err
	}

	// Explicitly chmod; the process umask is unconditionally applied otherwise.
	// We'll mask the mode with our own umask, but that may be different than
	// the process umask
	return os.Chmod(dst, mode(fmode, umask))
}

// copyFile copies a file in chunks from src path to dst path, using umask to create the dst file
func copyFile(ctx context.Context, dst, src string, disableSymlinks bool, fmode, umask os.FileMode) (int64, error) {
	if disableSymlinks {
		fileInfo, err := os.Lstat(src)
		if err != nil {
			return 0, fmt.Errorf("failed to check copy file source for symlinks: %w", err)
		}
		if fileInfo.Mode()&os.ModeSymlink == os.ModeSymlink {
			return 0, ErrSymlinkCopy
		}
	}

	srcF, err := os.Open(src)
	if err != nil {
		return 0, err
	}
	defer srcF.Close()

	dstF, err := os.OpenFile(dst, os.O_RDWR|os.O_CREATE|os.O_TRUNC, fmode)
	if err != nil {
		return 0, err
	}
	defer dstF.Close()

	count, err := Copy(ctx, dstF, srcF)
	if err != nil {
		return 0, err
	}

	// Explicitly chmod; the process umask is unconditionally applied otherwise.
	// We'll mask the mode with our own umask, but that may be different than
	// the process umask
	err = os.Chmod(dst, mode(fmode, umask))
	return count, err
}
