//go:build linux

package dmflakey

import (
	"fmt"
	"os"
	"os/exec"
	"time"
	"unsafe"

	"golang.org/x/sys/unix"
)

// newFlakeyDevice creates flakey device.
//
// REF: https://docs.kernel.org/admin-guide/device-mapper/dm-flakey.html
func newFlakeyDevice(flakeyDevice, loopDevice string, interval time.Duration) error {
	loopSize, err := getBlkSize(loopDevice)
	if err != nil {
		return fmt.Errorf("failed to get the size of the loop device %s: %w", loopDevice, err)
	}

	// The flakey device will be available in interval.Seconds().
	table := fmt.Sprintf("0 %d flakey %s 0 %d 0",
		loopSize, loopDevice, int(interval.Seconds()))

	args := []string{"create", flakeyDevice, "--table", table}

	output, err := exec.Command("dmsetup", args...).CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to create flakey device %s with table %s (out: %s): %w",
			flakeyDevice, table, string(output), err)
	}
	return nil
}

// reloadFlakeyDevice reloads the flakey device with feature table.
func reloadFlakeyDevice(flakeyDevice string, syncFS bool, table string) (retErr error) {
	args := []string{"suspend", "--nolockfs", flakeyDevice}
	if syncFS {
		args[1] = flakeyDevice
		args = args[:len(args)-1]
	}

	output, err := exec.Command("dmsetup", args...).CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to suspend flakey device %s (out: %s): %w",
			flakeyDevice, string(output), err)
	}

	defer func() {
		output, derr := exec.Command("dmsetup", "resume", flakeyDevice).CombinedOutput()
		if derr != nil {
			derr = fmt.Errorf("failed to resume flakey device %s (out: %s): %w",
				flakeyDevice, string(output), derr)
		}

		if retErr == nil {
			retErr = derr
		}
	}()

	output, err = exec.Command("dmsetup", "load", flakeyDevice, "--table", table).CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to reload flakey device %s with table (%s) (out: %s): %w",
			flakeyDevice, table, string(output), err)
	}
	return nil
}

// removeFlakeyDevice removes flakey device.
func deleteFlakeyDevice(flakeyDevice string) error {
	output, err := exec.Command("dmsetup", "remove", flakeyDevice).CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to remove flakey device %s (out: %s): %w",
			flakeyDevice, string(output), err)
	}
	return nil
}

// getBlkSize64 gets device size in bytes (BLKGETSIZE64).
//
// REF: https://man7.org/linux/man-pages/man8/blockdev.8.html
func getBlkSize64(device string) (int64, error) {
	deviceFd, err := os.Open(device)
	if err != nil {
		return 0, fmt.Errorf("failed to open device %s: %w", device, err)
	}
	defer deviceFd.Close()

	var size int64
	if _, _, err := unix.Syscall(unix.SYS_IOCTL, deviceFd.Fd(), unix.BLKGETSIZE64, uintptr(unsafe.Pointer(&size))); err != 0 {
		return 0, fmt.Errorf("failed to get block size: %w", err)
	}
	return size, nil
}

// getBlkSize gets size in 512-byte sectors (BLKGETSIZE64 / 512).
//
// REF: https://man7.org/linux/man-pages/man8/blockdev.8.html
func getBlkSize(device string) (int64, error) {
	size, err := getBlkSize64(device)
	return size / 512, err
}
