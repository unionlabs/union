package params

import (
	"os"
	"runtime"
	"time"
)

// IoConfig defines the shared io parameters.
type IoConfig struct {
	ReadWritePermissions        os.FileMode
	ReadWriteExecutePermissions os.FileMode
	BoltTimeout                 time.Duration
}

var defaultIoConfig = &IoConfig{
	ReadWritePermissions:        0600,            //-rw------- Read and Write permissions for user
	ReadWriteExecutePermissions: 0700,            //-rwx------ Read Write and Execute (traverse) permissions for user
	BoltTimeout:                 1 * time.Second, // 1 second for the bolt DB to timeout.
}

var defaultWindowsIoConfig = &IoConfig{
	ReadWritePermissions:        0666,
	ReadWriteExecutePermissions: 0777,
	BoltTimeout:                 1 * time.Second,
}

// BeaconIoConfig returns the current io config for
// the beacon chain.
func BeaconIoConfig() *IoConfig {
	if runtime.GOOS == "windows" {
		return defaultWindowsIoConfig
	}
	return defaultIoConfig
}
