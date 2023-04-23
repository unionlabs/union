package tempfile

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

const (
	atomicWriteFilePrefix = "write-file-atomic-"
	// Maximum number of atomic write file conflicts before we start reseeding
	// (reduced from golang's default 10 due to using an increased randomness space)
	atomicWriteFileMaxNumConflicts = 5
	// Maximum number of attempts to make at writing the write file before giving up
	// (reduced from golang's default 10000 due to using an increased randomness space)
	atomicWriteFileMaxNumWriteAttempts = 1000
	// LCG constants from Donald Knuth MMIX
	// This LCG's has a period equal to 2**64
	lcgA = 6364136223846793005
	lcgC = 1442695040888963407
	// Create in case it doesn't exist and force kernel
	// flush, which still leaves the potential of lingering disk cache.
	// Never overwrites files
	atomicWriteFileFlag = os.O_WRONLY | os.O_CREATE | os.O_SYNC | os.O_TRUNC | os.O_EXCL
)

var (
	atomicWriteFileRand   uint64
	atomicWriteFileRandMu cmtsync.Mutex
)

func writeFileRandReseed() uint64 {
	// Scale the PID, to minimize the chance that two processes seeded at similar times
	// don't get the same seed. Note that PID typically ranges in [0, 2**15), but can be
	// up to 2**22 under certain configurations. We left bit-shift the PID by 20, so that
	// a PID difference of one corresponds to a time difference of 2048 seconds.
	// The important thing here is that now for a seed conflict, they would both have to be on
	// the correct nanosecond offset, and second-based offset, which is much less likely than
	// just a conflict with the correct nanosecond offset.
	return uint64(time.Now().UnixNano() + int64(os.Getpid()<<20))
}

// Use a fast thread safe LCG for atomic write file names.
// Returns a string corresponding to a 64 bit int.
// If it was a negative int, the leading number is a 0.
func randWriteFileSuffix() string {
	atomicWriteFileRandMu.Lock()
	r := atomicWriteFileRand
	if r == 0 {
		r = writeFileRandReseed()
	}

	// Update randomness according to lcg
	r = r*lcgA + lcgC

	atomicWriteFileRand = r
	atomicWriteFileRandMu.Unlock()
	// Can have a negative name, replace this in the following
	suffix := strconv.Itoa(int(r))
	if string(suffix[0]) == "-" {
		// Replace first "-" with "0". This is purely for UI clarity,
		// as otherwhise there would be two `-` in a row.
		suffix = strings.Replace(suffix, "-", "0", 1)
	}
	return suffix
}

// WriteFileAtomic creates a temporary file with data and provided perm and
// swaps it atomically with filename if successful.
func WriteFileAtomic(filename string, data []byte, perm os.FileMode) (err error) {
	// This implementation is inspired by the golang stdlibs method of creating
	// tempfiles. Notable differences are that we use different flags, a 64 bit LCG
	// and handle negatives differently.
	// The core reason we can't use golang's TempFile is that we must write
	// to the file synchronously, as we need this to persist to disk.
	// We also open it in write-only mode, to avoid concerns that arise with read.
	var (
		dir = filepath.Dir(filename)
		f   *os.File
	)

	nconflict := 0
	// Limit the number of attempts to create a file. Something is seriously
	// wrong if it didn't get created after 1000 attempts, and we don't want
	// an infinite loop
	i := 0
	for ; i < atomicWriteFileMaxNumWriteAttempts; i++ {
		name := filepath.Join(dir, atomicWriteFilePrefix+randWriteFileSuffix())
		f, err = os.OpenFile(name, atomicWriteFileFlag, perm)
		// If the file already exists, try a new file
		if os.IsExist(err) {
			// If the files exists too many times, start reseeding as we've
			// likely hit another instances seed.
			if nconflict++; nconflict > atomicWriteFileMaxNumConflicts {
				atomicWriteFileRandMu.Lock()
				atomicWriteFileRand = writeFileRandReseed()
				atomicWriteFileRandMu.Unlock()
			}
			continue
		} else if err != nil {
			return err
		}
		break
	}
	if i == atomicWriteFileMaxNumWriteAttempts {
		return fmt.Errorf("could not create atomic write file after %d attempts", i)
	}

	// Clean up in any case. Defer stacking order is last-in-first-out.
	defer os.Remove(f.Name())
	defer f.Close()

	if n, err := f.Write(data); err != nil {
		return err
	} else if n < len(data) {
		return io.ErrShortWrite
	}
	// Close the file before renaming it, otherwise it will cause "The process
	// cannot access the file because it is being used by another process." on windows.
	f.Close()

	return os.Rename(f.Name(), filename)
}
