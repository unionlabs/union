//go:build linux

package dmflakey

import (
	"errors"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
	"time"

	testutils "go.etcd.io/bbolt/tests/utils"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/sys/unix"
)

func TestMain(m *testing.M) {
	flag.Parse()
	testutils.RequiresRoot()
	os.Exit(m.Run())
}

func TestBasic(t *testing.T) {
	for _, fsType := range []FSType{FSTypeEXT4, FSTypeXFS} {
		t.Run(string(fsType), func(t *testing.T) {
			tmpDir := t.TempDir()

			flakey, err := InitFlakey("go-dmflakey", tmpDir, fsType, "")
			require.NoError(t, err, "init flakey")
			defer func() {
				assert.NoError(t, flakey.Teardown())
			}()

			target := filepath.Join(tmpDir, "root")
			require.NoError(t, os.MkdirAll(target, 0600))

			require.NoError(t, mount(target, flakey.DevicePath(), ""))
			defer func() {
				assert.NoError(t, unmount(target))
			}()

			file := filepath.Join(target, "test")
			assert.NoError(t, writeFile(file, []byte("hello, world"), 0600, true))

			assert.NoError(t, unmount(target))

			assert.NoError(t, flakey.Teardown())
		})
	}
}

func TestDropWritesExt4(t *testing.T) {
	flakey, root := initFlakey(t, FSTypeEXT4)

	// commit=1000 is to delay commit triggered by writeback thread
	require.NoError(t, mount(root, flakey.DevicePath(), "commit=1000"))

	// ensure testdir/f1 is synced.
	target := filepath.Join(root, "testdir")
	require.NoError(t, os.MkdirAll(target, 0600))

	f1 := filepath.Join(target, "f1")
	assert.NoError(t, writeFile(f1, []byte("hello, world from f1"), 0600, false))
	require.NoError(t, syncfs(f1))

	// testdir/f2 is created but without fsync
	f2 := filepath.Join(target, "f2")
	assert.NoError(t, writeFile(f2, []byte("hello, world from f2"), 0600, false))

	// simulate power failure
	assert.NoError(t, flakey.DropWrites())
	assert.NoError(t, unmount(root))
	assert.NoError(t, flakey.AllowWrites())
	require.NoError(t, mount(root, flakey.DevicePath(), ""))

	data, err := os.ReadFile(f1)
	assert.NoError(t, err)
	assert.Equal(t, "hello, world from f1", string(data))

	_, err = os.ReadFile(f2)
	assert.True(t, errors.Is(err, os.ErrNotExist))
}

func TestErrorWritesExt4(t *testing.T) {
	flakey, root := initFlakey(t, FSTypeEXT4)

	// commit=1000 is to delay commit triggered by writeback thread
	require.NoError(t, mount(root, flakey.DevicePath(), "commit=1000"))

	// inject IO failure on write
	assert.NoError(t, flakey.ErrorWrites())

	f1 := filepath.Join(root, "f1")
	err := writeFile(f1, []byte("hello, world during failpoint"), 0600, true)
	assert.ErrorContains(t, err, "input/output error")

	// resume
	assert.NoError(t, flakey.AllowWrites())
	err = writeFile(f1, []byte("hello, world"), 0600, true)
	assert.NoError(t, err)

	assert.NoError(t, unmount(root))
	require.NoError(t, mount(root, flakey.DevicePath(), ""))

	data, err := os.ReadFile(f1)
	assert.NoError(t, err)
	assert.Equal(t, "hello, world", string(data))
}

func initFlakey(t *testing.T, fsType FSType) (_ Flakey, root string) {
	tmpDir := t.TempDir()

	target := filepath.Join(tmpDir, "root")
	require.NoError(t, os.MkdirAll(target, 0600))

	flakey, err := InitFlakey("go-dmflakey", tmpDir, fsType, "")
	require.NoError(t, err, "init flakey")

	t.Cleanup(func() {
		assert.NoError(t, unmount(target))
		assert.NoError(t, flakey.Teardown())
	})
	return flakey, target
}

func writeFile(name string, data []byte, perm os.FileMode, sync bool) error {
	f, err := os.OpenFile(name, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, perm)
	if err != nil {
		return err
	}
	defer f.Close()

	if _, err = f.Write(data); err != nil {
		return err
	}

	if sync {
		return f.Sync()
	}
	return nil
}

func syncfs(file string) error {
	f, err := os.Open(file)
	if err != nil {
		return fmt.Errorf("failed to open %s: %w", file, err)
	}
	defer f.Close()

	_, _, errno := unix.Syscall(unix.SYS_SYNCFS, uintptr(f.Fd()), 0, 0)
	if errno != 0 {
		return errno
	}
	return nil
}

func mount(target string, devPath string, opt string) error {
	args := []string{"-o", opt, devPath, target}

	output, err := exec.Command("mount", args...).CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to mount (args: %v) (out: %s): %w",
			args, string(output), err)
	}
	return nil
}

func unmount(target string) error {
	for i := 0; i < 50; i++ {
		if err := unix.Unmount(target, 0); err != nil {
			switch err {
			case unix.EBUSY:
				time.Sleep(500 * time.Millisecond)
				continue
			case unix.EINVAL:
			default:
				return fmt.Errorf("failed to umount %s: %w", target, err)
			}
		}
		return nil
	}
	return unix.EBUSY
}
