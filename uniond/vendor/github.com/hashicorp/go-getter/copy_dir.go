package getter

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// mode returns the file mode masked by the umask
func mode(mode, umask os.FileMode) os.FileMode {
	return mode & ^umask
}

// copyDir copies the src directory contents into dst. Both directories
// should already exist.
//
// If ignoreDot is set to true, then dot-prefixed files/folders are ignored.
func copyDir(ctx context.Context, dst string, src string, ignoreDot bool, disableSymlinks bool, umask os.FileMode) error {
	// We can safely evaluate the symlinks here, even if disabled, because they
	// will be checked before actual use in walkFn and copyFile
	var err error
	src, err = filepath.EvalSymlinks(src)
	if err != nil {
		return err
	}

	walkFn := func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if disableSymlinks {
			fileInfo, err := os.Lstat(path)
			if err != nil {
				return fmt.Errorf("failed to check copy file source for symlinks: %w", err)
			}
			if fileInfo.Mode()&os.ModeSymlink == os.ModeSymlink {
				return ErrSymlinkCopy
			}
			// if info.Mode()&os.ModeSymlink == os.ModeSymlink {
			// 	return ErrSymlinkCopy
			// }
		}

		if path == src {
			return nil
		}

		if ignoreDot && strings.HasPrefix(filepath.Base(path), ".") {
			// Skip any dot files
			if info.IsDir() {
				return filepath.SkipDir
			} else {
				return nil
			}
		}

		// The "path" has the src prefixed to it. We need to join our
		// destination with the path without the src on it.
		dstPath := filepath.Join(dst, path[len(src):])

		// If we have a directory, make that subdirectory, then continue
		// the walk.
		if info.IsDir() {
			if path == filepath.Join(src, dst) {
				// dst is in src; don't walk it.
				return nil
			}

			if err := os.MkdirAll(dstPath, mode(0755, umask)); err != nil {
				return err
			}

			return nil
		}

		// If we have a file, copy the contents.
		_, err = copyFile(ctx, dstPath, path, disableSymlinks, info.Mode(), umask)
		return err
	}

	return filepath.Walk(src, walkFn)
}
