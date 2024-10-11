package fs

import (
	"errors"
	"path/filepath"

	"golang.org/x/sys/windows"

	"github.com/Microsoft/go-winio/internal/stringbuffer"
)

var (
	// ErrInvalidPath is returned when the location of a file path doesn't begin with a driver letter.
	ErrInvalidPath = errors.New("the path provided to GetFileSystemType must start with a drive letter")
)

// GetFileSystemType obtains the type of a file system through GetVolumeInformation.
//
// https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumeinformationw
func GetFileSystemType(path string) (fsType string, err error) {
	drive := filepath.VolumeName(path)
	if len(drive) != 2 {
		return "", ErrInvalidPath
	}

	buf := stringbuffer.NewWString()
	defer buf.Free()

	drive += `\`
	err = windows.GetVolumeInformation(windows.StringToUTF16Ptr(drive), nil, 0, nil, nil, nil, buf.Pointer(), buf.Cap())
	return buf.String(), err
}
