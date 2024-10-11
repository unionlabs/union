//go:build linux

package dmflakey

import (
	"errors"
	"fmt"
	"os"
	"time"

	"golang.org/x/sys/unix"
)

const (
	loopControlDevice = "/dev/loop-control"
	loopDevicePattern = "/dev/loop%d"

	maxRetryToAttach = 50
)

// attachToLoopDevice associates free loop device with backing file.
//
// There might have race condition. It needs to retry when it runs into EBUSY.
//
// REF: https://man7.org/linux/man-pages/man4/loop.4.html
func attachToLoopDevice(backingFile string) (string, error) {
	backingFd, err := os.OpenFile(backingFile, os.O_RDWR, 0)
	if err != nil {
		return "", fmt.Errorf("failed to open loop device's backing file %s: %w",
			backingFile, err)
	}
	defer backingFd.Close()

	for i := 0; i < maxRetryToAttach; i++ {
		loop, err := getFreeLoopDevice()
		if err != nil {
			return "", fmt.Errorf("failed to get free loop device: %w", err)
		}

		err = func() error {
			loopFd, err := os.OpenFile(loop, os.O_RDWR, 0)
			if err != nil {
				return err
			}
			defer loopFd.Close()

			return unix.IoctlSetInt(int(loopFd.Fd()),
				unix.LOOP_SET_FD, int(backingFd.Fd()))
		}()
		if err != nil {
			if errors.Is(err, unix.EBUSY) {
				time.Sleep(500 * time.Millisecond)
				continue
			}
			return "", err
		}
		return loop, nil
	}
	return "", fmt.Errorf("failed to associate free loop device with backing file %s after retry %v",
		backingFile, maxRetryToAttach)
}

// detachLoopDevice disassociates the loop device from any backing file.
//
// REF: https://man7.org/linux/man-pages/man4/loop.4.html
func detachLoopDevice(loopDevice string) error {
	loopFd, err := os.Open(loopDevice)
	if err != nil {
		return fmt.Errorf("failed to open loop %s: %w", loopDevice, err)
	}
	defer loopFd.Close()

	return unix.IoctlSetInt(int(loopFd.Fd()), unix.LOOP_CLR_FD, 0)
}

// getFreeLoopDevice allocates or finds a free loop device for use.
//
// REF: https://man7.org/linux/man-pages/man4/loop.4.html
func getFreeLoopDevice() (string, error) {
	control, err := os.OpenFile(loopControlDevice, os.O_RDWR, 0)
	if err != nil {
		return "", fmt.Errorf("failed to open %s: %w", loopControlDevice, err)
	}

	idx, err := unix.IoctlRetInt(int(control.Fd()), unix.LOOP_CTL_GET_FREE)
	control.Close()
	if err != nil {
		return "", fmt.Errorf("failed to get free loop device number: %w", err)
	}
	return fmt.Sprintf(loopDevicePattern, idx), nil
}
