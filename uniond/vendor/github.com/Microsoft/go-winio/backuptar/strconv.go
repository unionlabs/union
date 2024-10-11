//go:build windows

package backuptar

import (
	"archive/tar"
	"fmt"
	"strconv"
	"strings"
	"time"
)

// Functions copied from https://github.com/golang/go/blob/master/src/archive/tar/strconv.go
// as we need to manage the LIBARCHIVE.creationtime PAXRecord manually.
// Idea taken from containerd which did the same thing.

// parsePAXTime takes a string of the form %d.%d as described in the PAX
// specification. Note that this implementation allows for negative timestamps,
// which is allowed for by the PAX specification, but not always portable.
func parsePAXTime(s string) (time.Time, error) {
	const maxNanoSecondDigits = 9

	// Split string into seconds and sub-seconds parts.
	ss, sn := s, ""
	if pos := strings.IndexByte(s, '.'); pos >= 0 {
		ss, sn = s[:pos], s[pos+1:]
	}

	// Parse the seconds.
	secs, err := strconv.ParseInt(ss, 10, 64)
	if err != nil {
		return time.Time{}, tar.ErrHeader
	}
	if len(sn) == 0 {
		return time.Unix(secs, 0), nil // No sub-second values
	}

	// Parse the nanoseconds.
	if strings.Trim(sn, "0123456789") != "" {
		return time.Time{}, tar.ErrHeader
	}
	if len(sn) < maxNanoSecondDigits {
		sn += strings.Repeat("0", maxNanoSecondDigits-len(sn)) // Right pad
	} else {
		sn = sn[:maxNanoSecondDigits] // Right truncate
	}
	nsecs, _ := strconv.ParseInt(sn, 10, 64) // Must succeed
	if len(ss) > 0 && ss[0] == '-' {
		return time.Unix(secs, -1*nsecs), nil // Negative correction
	}
	return time.Unix(secs, nsecs), nil
}

// formatPAXTime converts ts into a time of the form %d.%d as described in the
// PAX specification. This function is capable of negative timestamps.
func formatPAXTime(ts time.Time) (s string) {
	secs, nsecs := ts.Unix(), ts.Nanosecond()
	if nsecs == 0 {
		return strconv.FormatInt(secs, 10)
	}

	// If seconds is negative, then perform correction.
	sign := ""
	if secs < 0 {
		sign = "-"             // Remember sign
		secs = -(secs + 1)     // Add a second to secs
		nsecs = -(nsecs - 1e9) // Take that second away from nsecs
	}
	return strings.TrimRight(fmt.Sprintf("%s%d.%09d", sign, secs, nsecs), "0")
}
