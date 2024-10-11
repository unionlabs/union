//go:build linux

package dmflakey

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"strings"
	"time"

	"golang.org/x/sys/unix"
)

type featCfg struct {
	// SyncFS attempts to synchronize filesystem before inject failure.
	syncFS bool
	// interval is used to determine the up time for feature.
	//
	// For AllowWrites, it means that the device is available for `interval` seconds.
	// For Other features, the device exhibits unreliable behaviour for
	// `interval` seconds.
	interval time.Duration
}

// Default values.
const (
	// defaultImgSize is the default size for filesystem image.
	defaultImgSize int64 = 1024 * 1024 * 1024 * 10 // 10 GiB
	// defaultInterval is the default interval for the up time of feature.
	defaultInterval = 2 * time.Minute
)

// defaultFeatCfg is the default setting for flakey feature.
var defaultFeatCfg = featCfg{interval: defaultInterval}

// FeatOpt is used to configure failure feature.
type FeatOpt func(*featCfg)

// WithIntervalFeatOpt updates the up time for the feature.
func WithIntervalFeatOpt(interval time.Duration) FeatOpt {
	return func(cfg *featCfg) {
		cfg.interval = interval
	}
}

// WithSyncFSFeatOpt is to determine if the caller wants to synchronize
// filesystem before inject failure.
func WithSyncFSFeatOpt(syncFS bool) FeatOpt {
	return func(cfg *featCfg) {
		cfg.syncFS = syncFS
	}
}

// Flakey is to inject failure into device.
type Flakey interface {
	// DevicePath returns the flakey device path.
	DevicePath() string

	// Filesystem returns filesystem's type.
	Filesystem() FSType

	// AllowWrites allows write I/O.
	AllowWrites(opts ...FeatOpt) error

	// DropWrites drops all write I/O silently.
	DropWrites(opts ...FeatOpt) error

	// ErrorWrites drops all write I/O and returns error.
	ErrorWrites(opts ...FeatOpt) error

	// Teardown releases the flakey device.
	Teardown() error
}

// FSType represents the filesystem name.
type FSType string

// Supported filesystems.
const (
	FSTypeEXT4 FSType = "ext4"
	FSTypeXFS  FSType = "xfs"
)

// InitFlakey creates an filesystem on a loopback device and returns Flakey on it.
//
// The device-mapper device will be /dev/mapper/$flakeyDevice. And the filesystem
// image will be created at $dataStorePath/$flakeyDevice.img. By default, the
// device is available for 2 minutes and size is 10 GiB.
func InitFlakey(flakeyDevice, dataStorePath string, fsType FSType, mkfsOpt string) (_ Flakey, retErr error) {
	imgPath := filepath.Join(dataStorePath, fmt.Sprintf("%s.img", flakeyDevice))
	if err := createEmptyFSImage(imgPath, fsType, mkfsOpt); err != nil {
		return nil, err
	}
	defer func() {
		if retErr != nil {
			os.RemoveAll(imgPath)
		}
	}()

	loopDevice, err := attachToLoopDevice(imgPath)
	if err != nil {
		return nil, err
	}
	defer func() {
		if retErr != nil {
			_ = detachLoopDevice(loopDevice)
		}
	}()

	imgSize, err := getBlkSize(loopDevice)
	if err != nil {
		return nil, err
	}

	if err := newFlakeyDevice(flakeyDevice, loopDevice, defaultInterval); err != nil {
		return nil, err
	}

	return &flakey{
		fsType:  fsType,
		imgPath: imgPath,
		imgSize: imgSize,

		loopDevice:   loopDevice,
		flakeyDevice: flakeyDevice,
	}, nil
}

type flakey struct {
	fsType  FSType
	imgPath string
	imgSize int64

	loopDevice   string
	flakeyDevice string
}

// DevicePath returns the flakey device path.
func (f *flakey) DevicePath() string {
	return fmt.Sprintf("/dev/mapper/%s", f.flakeyDevice)
}

// Filesystem returns filesystem's type.
func (f *flakey) Filesystem() FSType {
	return f.fsType
}

// AllowWrites allows write I/O.
func (f *flakey) AllowWrites(opts ...FeatOpt) error {
	var o = defaultFeatCfg
	for _, opt := range opts {
		opt(&o)
	}

	// NOTE: Table parameters
	//
	// 0 imgSize flakey <dev path> <offset> <up interval> <down interval> [<num_features> [<feature arguments>]]
	//
	// Mandatory parameters:
	//
	//  <dev path>: Full pathname to the underlying block-device, or a "major:minor" device-number.
	//  <offset>: Starting sector within the device.
	//  <up interval>: Number of seconds device is available.
	//  <down interval>: Number of seconds device returns errors.
	//
	// Optional:
	//
	// If no feature parameters are present, during the periods of unreliability, all I/O returns errors.
	//
	// For AllowWrites, the device will handle data correctly in `interval` seconds.
	//
	// REF: https://docs.kernel.org/admin-guide/device-mapper/dm-flakey.html.
	table := fmt.Sprintf("0 %d flakey %s 0 %d 0",
		f.imgSize, f.loopDevice, int(o.interval.Seconds()))

	return reloadFlakeyDevice(f.flakeyDevice, o.syncFS, table)
}

// DropWrites drops all write I/O silently.
func (f *flakey) DropWrites(opts ...FeatOpt) error {
	var o = defaultFeatCfg
	for _, opt := range opts {
		opt(&o)
	}

	// NOTE: Table parameters
	//
	// 0 imgSize flakey <dev path> <offset> <up interval> <down interval> [<num_features> [<feature arguments>]]
	//
	// Mandatory parameters:
	//
	//  <dev path>: Full pathname to the underlying block-device, or a "major:minor" device-number.
	//  <offset>: Starting sector within the device.
	//  <up interval>: Number of seconds device is available.
	//  <down interval>: Number of seconds device returns errors.
	//
	// Optional:
	//
	// <num_features>: How many arguments (length of <feature_arguments>)
	//
	// For DropWrites,
	//
	// num_features: 1 (there is only one argument)
	// feature_arguments: drop_writes
	//
	// The Device will drop all the writes into disk in `interval` seconds.
	// Read I/O is handled correctly.
	//
	// For example, the application calls fsync, all the dirty pages will
	// be flushed into disk ideally. But during DropWrites, device will
	// ignore all the data and return successfully. It can be used to
	// simulate data-loss after power failure.
	//
	// REF: https://docs.kernel.org/admin-guide/device-mapper/dm-flakey.html.
	table := fmt.Sprintf("0 %d flakey %s 0 0 %d 1 drop_writes",
		f.imgSize, f.loopDevice, int(o.interval.Seconds()))

	return reloadFlakeyDevice(f.flakeyDevice, o.syncFS, table)
}

// ErrorWrites drops all write I/O and returns error.
func (f *flakey) ErrorWrites(opts ...FeatOpt) error {
	var o = defaultFeatCfg
	for _, opt := range opts {
		opt(&o)
	}

	// NOTE: Table parameters
	//
	// 0 imgSize flakey <dev path> <offset> <up interval> <down interval> [<num_features> [<feature arguments>]]
	//
	// Mandatory parameters:
	//
	//  <dev path>: Full pathname to the underlying block-device, or a "major:minor" device-number.
	//  <offset>: Starting sector within the device.
	//  <up interval>: Number of seconds device is available.
	//  <down interval>: Number of seconds device returns errors.
	//
	// Optional:
	//
	// <num_features>: How many arguments (length of <feature_arguments>)
	//
	// For ErrorWrites,
	//
	// num_features: 1 (there is only one argument)
	// feature_arguments: error_writes
	//
	// The Device will drop all the writes into disk in `interval` seconds
	// and return failure to caller. Read I/O is handled correctly.
	//
	// REF: https://docs.kernel.org/admin-guide/device-mapper/dm-flakey.html.
	table := fmt.Sprintf("0 %d flakey %s 0 0 %d 1 error_writes",
		f.imgSize, f.loopDevice, int(o.interval.Seconds()))

	return reloadFlakeyDevice(f.flakeyDevice, o.syncFS, table)
}

// Teardown releases the flakey device.
func (f *flakey) Teardown() error {
	if err := deleteFlakeyDevice(f.flakeyDevice); err != nil {
		if !strings.Contains(err.Error(), "No such device or address") {
			return err
		}
	}
	if err := detachLoopDevice(f.loopDevice); err != nil {
		if !errors.Is(err, unix.ENXIO) {
			return err
		}
	}
	return os.RemoveAll(f.imgPath)
}

// createEmptyFSImage creates empty filesystem on dataStorePath folder with
// default size - 10 GiB.
func createEmptyFSImage(imgPath string, fsType FSType, mkfsOpt string) error {
	if err := validateFSType(fsType); err != nil {
		return err
	}

	mkfs, err := exec.LookPath(fmt.Sprintf("mkfs.%s", fsType))
	if err != nil {
		return fmt.Errorf("failed to ensure mkfs.%s: %w", fsType, err)
	}

	if _, err := os.Stat(imgPath); err == nil {
		return fmt.Errorf("failed to create image because %s already exists", imgPath)
	}

	if err := os.MkdirAll(path.Dir(imgPath), 0600); err != nil {
		return fmt.Errorf("failed to ensure parent directory %s: %w", path.Dir(imgPath), err)
	}

	f, err := os.Create(imgPath)
	if err != nil {
		return fmt.Errorf("failed to create image %s: %w", imgPath, err)
	}

	if err = func() error {
		defer f.Close()

		return f.Truncate(defaultImgSize)
	}(); err != nil {
		return fmt.Errorf("failed to truncate image %s with %v bytes: %w",
			imgPath, defaultImgSize, err)
	}

	args := []string{imgPath}
	if mkfsOpt != "" {
		splitArgs := strings.Split(mkfsOpt, " ")
		args = append(splitArgs, imgPath)
	}

	output, err := exec.Command(mkfs, args...).CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to mkfs on %s (%s %v) (out: %s): %w",
			imgPath, mkfs, args, string(output), err)
	}
	return nil
}

// validateFSType validates the fs type input.
func validateFSType(fsType FSType) error {
	switch fsType {
	case FSTypeEXT4, FSTypeXFS:
		return nil
	default:
		return fmt.Errorf("unsupported filesystem %s", fsType)
	}
}
