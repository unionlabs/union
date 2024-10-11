//go:build windows

package fs

import (
	"errors"
	"os"
	"strings"

	"golang.org/x/sys/windows"

	"github.com/Microsoft/go-winio/internal/fs"
)

// ResolvePath returns the final path to a file or directory represented, resolving symlinks,
// handling mount points, etc.
// The resolution works by using the Windows API GetFinalPathNameByHandle, which takes a
// handle and returns the final path to that file.
//
// It is intended to address short-comings of [filepath.EvalSymlinks], which does not work
// well on Windows.
func ResolvePath(path string) (string, error) {
	// We are not able to use builtin Go functionality for opening a directory path:
	//   - os.Open on a directory returns a os.File where Fd() is a search handle from FindFirstFile.
	//   - syscall.Open does not provide a way to specify FILE_FLAG_BACKUP_SEMANTICS, which is needed to
	//     open a directory.
	//
	// We could use os.Open if the path is a file, but it's easier to just use the same code for both.
	// Therefore, we call windows.CreateFile directly.
	h, err := fs.CreateFile(
		path,
		fs.FILE_ANY_ACCESS, // access
		fs.FILE_SHARE_READ|fs.FILE_SHARE_WRITE|fs.FILE_SHARE_DELETE,
		nil, // security attributes
		fs.OPEN_EXISTING,
		fs.FILE_FLAG_BACKUP_SEMANTICS, // Needed to open a directory handle.
		fs.NullHandle,                 // template file
	)
	if err != nil {
		return "", &os.PathError{
			Op:   "CreateFile",
			Path: path,
			Err:  err,
		}
	}
	defer windows.CloseHandle(h) //nolint:errcheck

	// We use the Windows API GetFinalPathNameByHandle to handle path resolution. GetFinalPathNameByHandle
	// returns a resolved path name for a file or directory. The returned path can be in several different
	// formats, based on the flags passed. There are several goals behind the design here:
	// - Do as little manual path manipulation as possible. Since Windows path formatting can be quite
	//   complex, we try to just let the Windows APIs handle that for us.
	// - Retain as much compatibility with existing Go path functions as we can. In particular, we try to
	//   ensure paths returned from resolvePath can be passed to EvalSymlinks.
	//
	// First, we query for the VOLUME_NAME_GUID path of the file. This will return a path in the form
	// "\\?\Volume{8a25748f-cf34-4ac6-9ee2-c89400e886db}\dir\file.txt". If the path is a UNC share
	// (e.g. "\\server\share\dir\file.txt"), then the VOLUME_NAME_GUID query will fail with ERROR_PATH_NOT_FOUND.
	// In this case, we will next try a VOLUME_NAME_DOS query. This query will return a path for a UNC share
	// in the form "\\?\UNC\server\share\dir\file.txt". This path will work with most functions, but EvalSymlinks
	// fails on it. Therefore, we rewrite the path to the form "\\server\share\dir\file.txt" before returning it.
	// This path rewrite may not be valid in all cases (see the notes in the next paragraph), but those should
	// be very rare edge cases, and this case wouldn't have worked with EvalSymlinks anyways.
	//
	// The "\\?\" prefix indicates that no path parsing or normalization should be performed by Windows.
	// Instead the path is passed directly to the object manager. The lack of parsing means that "." and ".." are
	// interpreted literally and "\"" must be used as a path separator. Additionally, because normalization is
	// not done, certain paths can only be represented in this format. For instance, "\\?\C:\foo." (with a trailing .)
	// cannot be written as "C:\foo.", because path normalization will remove the trailing ".".
	//
	// FILE_NAME_NORMALIZED can fail on some UNC paths based on access restrictions.
	// Attempt to query with FILE_NAME_NORMALIZED, and then fall back on FILE_NAME_OPENED if access is denied.
	//
	// Querying for VOLUME_NAME_DOS first instead of VOLUME_NAME_GUID would yield a "nicer looking" path in some cases.
	// For instance, it could return "\\?\C:\dir\file.txt" instead of "\\?\Volume{8a25748f-cf34-4ac6-9ee2-c89400e886db}\dir\file.txt".
	// However, we query for VOLUME_NAME_GUID first for two reasons:
	// - The volume GUID path is more stable. A volume's mount point can change when it is remounted, but its
	//   volume GUID should not change.
	// - If the volume is mounted at a non-drive letter path (e.g. mounted to "C:\mnt"), then VOLUME_NAME_DOS
	//   will return the mount path. EvalSymlinks fails on a path like this due to a bug.
	//
	// References:
	// - GetFinalPathNameByHandle: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlea
	// - Naming Files, Paths, and Namespaces: https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file
	// - Naming a Volume: https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-volume

	normalize := true
	guid := true
	rPath := ""
	for i := 1; i <= 4; i++ { // maximum of 4 different cases to try
		var flags fs.GetFinalPathFlag
		if normalize {
			flags |= fs.FILE_NAME_NORMALIZED // nop; for clarity
		} else {
			flags |= fs.FILE_NAME_OPENED
		}

		if guid {
			flags |= fs.VOLUME_NAME_GUID
		} else {
			flags |= fs.VOLUME_NAME_DOS // nop; for clarity
		}

		rPath, err = fs.GetFinalPathNameByHandle(h, flags)
		switch {
		case guid && errors.Is(err, windows.ERROR_PATH_NOT_FOUND):
			// ERROR_PATH_NOT_FOUND is returned from the VOLUME_NAME_GUID query if the path is a
			// network share (UNC path). In this case, query for the DOS name instead.
			guid = false
			continue
		case normalize && errors.Is(err, windows.ERROR_ACCESS_DENIED):
			// normalization failed when accessing individual components along path for SMB share
			normalize = false
			continue
		default:
		}
		break
	}

	if err == nil && strings.HasPrefix(rPath, `\\?\UNC\`) {
		// Convert \\?\UNC\server\share -> \\server\share. The \\?\UNC syntax does not work with
		// some Go filepath functions such as EvalSymlinks. In the future if other components
		// move away from EvalSymlinks and use GetFinalPathNameByHandle instead, we could remove
		// this path munging.
		rPath = `\\` + rPath[len(`\\?\UNC\`):]
	}
	return rPath, err
}
