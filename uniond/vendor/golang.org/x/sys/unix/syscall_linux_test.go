// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build linux

package unix_test

import (
	"bufio"
	"bytes"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"strconv"
	"strings"
	"syscall"
	"testing"
	"time"
	"unsafe"

	"golang.org/x/sys/unix"
)

func TestIoctlGetEthtoolDrvinfo(t *testing.T) {
	if runtime.GOOS == "android" {
		t.Skip("ethtool driver info is not available on android, skipping test")
	}

	s, err := unix.Socket(unix.AF_INET, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatalf("failed to open socket: %v", err)
	}
	defer unix.Close(s)

	ifis, err := net.Interfaces()
	if err != nil {
		t.Fatalf("failed to get network interfaces: %v", err)
	}

	// Print the interface name and associated driver information for each
	// network interface supported by ethtool.
	for _, ifi := range ifis {
		drv, err := unix.IoctlGetEthtoolDrvinfo(s, ifi.Name)
		if err != nil {
			if err == unix.EOPNOTSUPP {
				continue
			}

			if err == unix.EBUSY {
				// See https://go.dev/issues/67350
				t.Logf("%s: ethtool driver busy, possible kernel bug", ifi.Name)
				continue
			}

			t.Fatalf("failed to get ethtool driver info for %q: %v", ifi.Name, err)
		}

		// Trim trailing NULLs.
		t.Logf("%s: %q", ifi.Name, string(bytes.TrimRight(drv.Driver[:], "\x00")))
	}
}

func TestIoctlGetInt(t *testing.T) {
	f, err := os.Open("/dev/random")
	if err != nil {
		t.Fatalf("failed to open device: %v", err)
	}
	defer f.Close()

	v, err := unix.IoctlGetInt(int(f.Fd()), unix.RNDGETENTCNT)
	if err != nil {
		t.Fatalf("failed to perform ioctl: %v", err)
	}

	t.Logf("%d bits of entropy available", v)
}

func TestIoctlRetInt(t *testing.T) {
	f, err := os.Open("/proc/self/ns/mnt")
	if err != nil {
		t.Skipf("skipping test, %v", err)
	}
	defer f.Close()

	v, err := unix.IoctlRetInt(int(f.Fd()), unix.NS_GET_NSTYPE)
	if err != nil {
		if err == unix.ENOTTY {
			t.Skipf("old kernel? (need Linux >= 4.11)")
		}
		t.Fatalf("failed to perform ioctl: %v", err)
	}
	if v != unix.CLONE_NEWNS {
		t.Fatalf("unexpected return from ioctl; expected %v, got %v", v, unix.CLONE_NEWNS)
	}
}

func TestIoctlGetRTCTime(t *testing.T) {
	f, err := os.Open("/dev/rtc0")
	if err != nil {
		t.Skipf("skipping test, %v", err)
	}
	defer f.Close()

	v, err := unix.IoctlGetRTCTime(int(f.Fd()))
	if err != nil {
		t.Fatalf("failed to perform ioctl: %v", err)
	}

	t.Logf("RTC time: %04d-%02d-%02d %02d:%02d:%02d", v.Year+1900, v.Mon+1, v.Mday, v.Hour, v.Min, v.Sec)
}

func TestIoctlGetRTCWkAlrm(t *testing.T) {
	f, err := os.Open("/dev/rtc0")
	if err != nil {
		t.Skipf("skipping test, %v", err)
	}
	defer f.Close()

	v, err := unix.IoctlGetRTCWkAlrm(int(f.Fd()))

	// Not all RTC drivers support wakeup alarms, and will return EINVAL in such cases.
	if err == unix.EINVAL {
		t.Skip("RTC_WKALM_RD ioctl not supported on this rtc, skipping test")
	}

	if err != nil {
		t.Fatalf("failed to perform ioctl: %v", err)
	}

	t.Logf("RTC wake alarm enabled '%d'; time: %04d-%02d-%02d %02d:%02d:%02d",
		v.Enabled, v.Time.Year+1900, v.Time.Mon+1, v.Time.Mday, v.Time.Hour, v.Time.Min, v.Time.Sec)
}

func TestIoctlIfreq(t *testing.T) {
	s, err := unix.Socket(unix.AF_INET, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatalf("failed to open socket: %v", err)
	}
	defer unix.Close(s)

	ifis, err := net.Interfaces()
	if err != nil {
		t.Fatalf("failed to get network interfaces: %v", err)
	}

	// Compare the network interface fetched from rtnetlink with the data from
	// the equivalent ioctl API.
	for _, ifi := range ifis {
		ifr, err := unix.NewIfreq(ifi.Name)
		if err != nil {
			t.Fatalf("failed to create ifreq for %q: %v", ifi.Name, err)
		}

		if err := unix.IoctlIfreq(s, unix.SIOCGIFINDEX, ifr); err != nil {
			t.Fatalf("failed to get interface index for %q: %v", ifi.Name, err)
		}

		if want, got := ifi.Index, int(ifr.Uint32()); want != got {
			t.Fatalf("unexpected interface index for %q: got: %d, want: %d",
				ifi.Name, got, want)
		}

		if want, got := ifi.Name, ifr.Name(); want != got {
			t.Fatalf("unexpected interface name for index %d: got: %q, want: %q",
				ifi.Index, got, want)
		}

		wantIP, ok := firstIPv4(t, &ifi)
		if err := unix.IoctlIfreq(s, unix.SIOCGIFADDR, ifr); err != nil {
			// Interface may have no assigned IPv4 address.
			if err != unix.EADDRNOTAVAIL {
				t.Fatalf("failed to get IPv4 address for %q: %v", ifi.Name, err)
			}

			// But if we found an address via rtnetlink, we should expect the
			// ioctl to return one.
			if ok {
				t.Fatalf("found IPv4 address %q for %q but ioctl returned none", wantIP, ifi.Name)
			}

			continue
		}

		// Found an address, compare it directly.
		addr, err := ifr.Inet4Addr()
		if err != nil {
			t.Fatalf("failed to get ifreq IPv4 address: %v", err)
		}

		if want, got := wantIP, addr; !want.Equal(got) {
			t.Fatalf("unexpected first IPv4 address for %q: got: %q, want: %q",
				ifi.Name, got, want)
		}
	}
}

// firstIPv4 reports whether the interface has an IPv4 address assigned,
// returning the first discovered address.
func firstIPv4(t *testing.T, ifi *net.Interface) (net.IP, bool) {
	t.Helper()

	addrs, err := ifi.Addrs()
	if err != nil {
		t.Fatalf("failed to get interface %q addresses: %v", ifi.Name, err)
	}

	for _, a := range addrs {
		// Only want valid IPv4 addresses.
		ipn, ok := a.(*net.IPNet)
		if !ok || ipn.IP.To4() == nil {
			continue
		}

		return ipn.IP, true
	}

	return nil, false
}

func TestPidfd(t *testing.T) {
	// Start a child process which will sleep for 1 hour; longer than the 10
	// minute default Go test timeout.
	cmd := exec.Command("sleep", "1h")
	if err := cmd.Start(); err != nil {
		t.Fatalf("failed to exec sleep: %v", err)
	}

	fd, err := unix.PidfdOpen(cmd.Process.Pid, 0)
	if err != nil {
		// GOARCH arm/arm64 and GOOS android builders do not support pidfds.
		if errors.Is(err, unix.ENOSYS) {
			t.Skipf("skipping, pidfd_open is not implemented: %v", err)
		}

		t.Fatalf("failed to open child pidfd: %v", err)
	}
	defer unix.Close(fd)

	// Child is running but not terminated.
	if err := unix.Waitid(unix.P_PIDFD, fd, nil, unix.WEXITED|unix.WNOHANG, nil); err != nil {
		if errors.Is(err, unix.EINVAL) {
			t.Skip("skipping due to waitid EINVAL, see https://go.dev/issues/52014")
		}

		t.Fatalf("failed to check for child exit: %v", err)
	}

	const want = unix.SIGHUP
	if err := unix.PidfdSendSignal(fd, want, nil, 0); err != nil {
		t.Fatalf("failed to signal child process: %v", err)
	}

	// Now verify that the child process received the expected signal.
	var eerr *exec.ExitError
	if err := cmd.Wait(); !errors.As(err, &eerr) {
		t.Fatalf("child process terminated but did not return an exit error: %v", err)
	}

	if err := unix.Waitid(unix.P_PIDFD, fd, nil, unix.WEXITED, nil); !errors.Is(err, unix.ECHILD) {
		t.Fatalf("expected ECHILD for final waitid, but got: %v", err)
	}

	ws, ok := eerr.Sys().(syscall.WaitStatus)
	if !ok {
		t.Fatalf("expected syscall.WaitStatus value, but got: %#T", eerr.Sys())
	}

	if got := ws.Signal(); got != want {
		t.Fatalf("unexpected child exit signal, got: %s, want: %s", got, want)
	}
}

func TestPpoll(t *testing.T) {
	if runtime.GOOS == "android" {
		t.Skip("mkfifo syscall is not available on android, skipping test")
	}

	chtmpdir(t)
	f := mktmpfifo(t)

	const timeout = 100 * time.Millisecond

	ok := make(chan bool, 1)
	go func() {
		select {
		case <-time.After(10 * timeout):
			t.Errorf("Ppoll: failed to timeout after %d", 10*timeout)
		case <-ok:
		}
	}()

	fds := []unix.PollFd{{Fd: int32(f.Fd()), Events: unix.POLLIN}}
	timeoutTs := unix.NsecToTimespec(int64(timeout))
	n, err := unix.Ppoll(fds, &timeoutTs, nil)
	ok <- true
	if err != nil {
		t.Errorf("Ppoll: unexpected error: %v", err)
		return
	}
	if n != 0 {
		t.Errorf("Ppoll: wrong number of events: got %v, expected %v", n, 0)
		return
	}
}

func TestTime(t *testing.T) {
	var ut unix.Time_t
	ut2, err := unix.Time(&ut)
	if err != nil {
		t.Fatalf("Time: %v", err)
	}
	if ut != ut2 {
		t.Errorf("Time: return value %v should be equal to argument %v", ut2, ut)
	}

	var now time.Time

	for i := 0; i < 10; i++ {
		ut, err = unix.Time(nil)
		if err != nil {
			t.Fatalf("Time: %v", err)
		}

		now = time.Now()
		diff := int64(ut) - now.Unix()
		if -1 <= diff && diff <= 1 {
			return
		}
	}

	t.Errorf("Time: return value %v should be nearly equal to time.Now().Unix() %v±1", ut, now.Unix())
}

func TestUtime(t *testing.T) {
	chtmpdir(t)

	touch(t, "file1")

	buf := &unix.Utimbuf{
		Modtime: 12345,
	}

	err := unix.Utime("file1", buf)
	if err != nil {
		t.Fatalf("Utime: %v", err)
	}

	fi, err := os.Stat("file1")
	if err != nil {
		t.Fatal(err)
	}

	if fi.ModTime().Unix() != 12345 {
		t.Errorf("Utime: failed to change modtime: expected %v, got %v", 12345, fi.ModTime().Unix())
	}
}

func TestRlimitAs(t *testing.T) {
	// disable GC during to avoid flaky test
	defer debug.SetGCPercent(debug.SetGCPercent(-1))

	var rlim unix.Rlimit
	err := unix.Getrlimit(unix.RLIMIT_AS, &rlim)
	if err != nil {
		t.Fatalf("Getrlimit: %v", err)
	}
	var zero unix.Rlimit
	if zero == rlim {
		t.Fatalf("Getrlimit: got zero value %#v", rlim)
	}
	set := rlim
	set.Cur = uint64(unix.Getpagesize())
	err = unix.Setrlimit(unix.RLIMIT_AS, &set)
	if err != nil {
		t.Fatalf("Setrlimit: set failed: %#v %v", set, err)
	}

	// RLIMIT_AS was set to the page size, so mmap()'ing twice the page size
	// should fail. See 'man 2 getrlimit'.
	_, err = unix.Mmap(-1, 0, 2*unix.Getpagesize(), unix.PROT_NONE, unix.MAP_ANON|unix.MAP_PRIVATE)
	if err == nil {
		t.Fatal("Mmap: unexpectedly succeeded after setting RLIMIT_AS")
	}

	err = unix.Setrlimit(unix.RLIMIT_AS, &rlim)
	if err != nil {
		t.Fatalf("Setrlimit: restore failed: %#v %v", rlim, err)
	}

	b, err := unix.Mmap(-1, 0, 2*unix.Getpagesize(), unix.PROT_NONE, unix.MAP_ANON|unix.MAP_PRIVATE)
	if err != nil {
		t.Fatalf("Mmap: %v", err)
	}
	err = unix.Munmap(b)
	if err != nil {
		t.Fatalf("Munmap: %v", err)
	}
}

func TestPselect(t *testing.T) {
	for {
		n, err := unix.Pselect(0, nil, nil, nil, &unix.Timespec{Sec: 0, Nsec: 0}, nil)
		if err == unix.EINTR {
			t.Logf("Pselect interrupted")
			continue
		} else if err != nil {
			t.Fatalf("Pselect: %v", err)
		}
		if n != 0 {
			t.Fatalf("Pselect: got %v ready file descriptors, expected 0", n)
		}
		break
	}

	dur := 2500 * time.Microsecond
	var took time.Duration
	for {
		// On some platforms (e.g. Linux), the passed-in timespec is
		// updated by pselect(2). Make sure to reset to the full
		// duration in case of an EINTR.
		ts := unix.NsecToTimespec(int64(dur))
		start := time.Now()
		n, err := unix.Pselect(0, nil, nil, nil, &ts, nil)
		took = time.Since(start)
		if err == unix.EINTR {
			t.Logf("Pselect interrupted after %v", took)
			continue
		} else if err != nil {
			t.Fatalf("Pselect: %v", err)
		}
		if n != 0 {
			t.Fatalf("Pselect: got %v ready file descriptors, expected 0", n)
		}
		break
	}

	// On some builder the actual timeout might also be slightly less than the requested.
	// Add an acceptable margin to avoid flaky tests.
	if took < dur*2/3 {
		t.Errorf("Pselect: got %v timeout, expected at least %v", took, dur)
	}
}

func TestPselectWithSigmask(t *testing.T) {
	var sigmask unix.Sigset_t
	sigmask.Val[0] |= 1 << (uint(unix.SIGUSR1) - 1)
	for {
		n, err := unix.Pselect(0, nil, nil, nil, &unix.Timespec{Sec: 0, Nsec: 0}, &sigmask)
		if err == unix.EINTR {
			t.Logf("Pselect interrupted")
			continue
		} else if err != nil {
			t.Fatalf("Pselect: %v", err)
		}
		if n != 0 {
			t.Fatalf("Pselect: got %v ready file descriptors, expected 0", n)
		}
		break
	}
}

func TestSchedSetaffinity(t *testing.T) {
	var newMask unix.CPUSet
	newMask.Zero()
	if newMask.Count() != 0 {
		t.Errorf("CpuZero: didn't zero CPU set: %v", newMask)
	}
	cpu := 1
	newMask.Set(cpu)
	if newMask.Count() != 1 || !newMask.IsSet(cpu) {
		t.Errorf("CpuSet: didn't set CPU %d in set: %v", cpu, newMask)
	}
	cpu = 5
	newMask.Set(cpu)
	if newMask.Count() != 2 || !newMask.IsSet(cpu) {
		t.Errorf("CpuSet: didn't set CPU %d in set: %v", cpu, newMask)
	}
	newMask.Clear(cpu)
	if newMask.Count() != 1 || newMask.IsSet(cpu) {
		t.Errorf("CpuClr: didn't clear CPU %d in set: %v", cpu, newMask)
	}

	runtime.LockOSThread()
	defer runtime.UnlockOSThread()

	var oldMask unix.CPUSet
	err := unix.SchedGetaffinity(0, &oldMask)
	if err != nil {
		t.Fatalf("SchedGetaffinity: %v", err)
	}

	if runtime.NumCPU() < 2 {
		t.Skip("skipping setaffinity tests on single CPU system")
	}
	if runtime.GOOS == "android" {
		t.Skip("skipping setaffinity tests on android")
	}

	// On a system like ppc64x where some cores can be disabled using ppc64_cpu,
	// setaffinity should only be called with enabled cores. The valid cores
	// are found from the oldMask, but if none are found then the setaffinity
	// tests are skipped. Issue #27875.
	cpu = 1
	if !oldMask.IsSet(cpu) {
		newMask.Zero()
		for i := 0; i < len(oldMask); i++ {
			if oldMask.IsSet(i) {
				newMask.Set(i)
				break
			}
		}
		if newMask.Count() == 0 {
			t.Skip("skipping setaffinity tests if CPU not available")
		}
	}

	err = unix.SchedSetaffinity(0, &newMask)
	if err != nil {
		t.Fatalf("SchedSetaffinity: %v", err)
	}

	var gotMask unix.CPUSet
	err = unix.SchedGetaffinity(0, &gotMask)
	if err != nil {
		t.Fatalf("SchedGetaffinity: %v", err)
	}

	if gotMask != newMask {
		t.Errorf("SchedSetaffinity: returned affinity mask does not match set affinity mask")
	}

	// Restore old mask so it doesn't affect successive tests
	err = unix.SchedSetaffinity(0, &oldMask)
	if err != nil {
		t.Fatalf("SchedSetaffinity: %v", err)
	}
}

func TestStatx(t *testing.T) {
	var stx unix.Statx_t
	err := unix.Statx(unix.AT_FDCWD, ".", 0, 0, &stx)
	if err == unix.ENOSYS || err == unix.EPERM {
		t.Skip("statx syscall is not available, skipping test")
	} else if err != nil {
		t.Fatalf("Statx: %v", err)
	}

	chtmpdir(t)
	touch(t, "file1")

	var st unix.Stat_t
	err = unix.Stat("file1", &st)
	if err != nil {
		t.Fatalf("Stat: %v", err)
	}

	flags := unix.AT_STATX_SYNC_AS_STAT
	err = unix.Statx(unix.AT_FDCWD, "file1", flags, unix.STATX_ALL, &stx)
	if err != nil {
		t.Fatalf("Statx: %v", err)
	}

	if uint32(stx.Mode) != st.Mode {
		t.Errorf("Statx: returned stat mode does not match Stat")
	}

	ctime := unix.StatxTimestamp{Sec: int64(st.Ctim.Sec), Nsec: uint32(st.Ctim.Nsec)}
	mtime := unix.StatxTimestamp{Sec: int64(st.Mtim.Sec), Nsec: uint32(st.Mtim.Nsec)}

	if stx.Ctime != ctime {
		t.Errorf("Statx: returned stat ctime does not match Stat")
	}
	if stx.Mtime != mtime {
		t.Errorf("Statx: returned stat mtime does not match Stat")
	}

	err = os.Symlink("file1", "symlink1")
	if err != nil {
		t.Fatal(err)
	}

	err = unix.Lstat("symlink1", &st)
	if err != nil {
		t.Fatalf("Lstat: %v", err)
	}

	err = unix.Statx(unix.AT_FDCWD, "symlink1", flags, unix.STATX_BASIC_STATS, &stx)
	if err != nil {
		t.Fatalf("Statx: %v", err)
	}

	// follow symlink, expect a regulat file
	if stx.Mode&unix.S_IFREG == 0 {
		t.Errorf("Statx: didn't follow symlink")
	}

	err = unix.Statx(unix.AT_FDCWD, "symlink1", flags|unix.AT_SYMLINK_NOFOLLOW, unix.STATX_ALL, &stx)
	if err != nil {
		t.Fatalf("Statx: %v", err)
	}

	// follow symlink, expect a symlink
	if stx.Mode&unix.S_IFLNK == 0 {
		t.Errorf("Statx: unexpectedly followed symlink")
	}
	if uint32(stx.Mode) != st.Mode {
		t.Errorf("Statx: returned stat mode does not match Lstat")
	}

	ctime = unix.StatxTimestamp{Sec: int64(st.Ctim.Sec), Nsec: uint32(st.Ctim.Nsec)}
	mtime = unix.StatxTimestamp{Sec: int64(st.Mtim.Sec), Nsec: uint32(st.Mtim.Nsec)}

	if stx.Ctime != ctime {
		t.Errorf("Statx: returned stat ctime does not match Lstat")
	}
	if stx.Mtime != mtime {
		t.Errorf("Statx: returned stat mtime does not match Lstat")
	}
}

// stringsFromByteSlice converts a sequence of attributes to a []string.
// On Linux, each entry is a NULL-terminated string.
func stringsFromByteSlice(buf []byte) []string {
	var result []string
	off := 0
	for i, b := range buf {
		if b == 0 {
			result = append(result, string(buf[off:i]))
			off = i + 1
		}
	}
	return result
}

func TestFaccessat(t *testing.T) {
	chtmpdir(t)
	touch(t, "file1")

	err := unix.Faccessat(unix.AT_FDCWD, "file1", unix.R_OK, 0)
	if err != nil {
		t.Errorf("Faccessat: unexpected error: %v", err)
	}

	err = unix.Faccessat(unix.AT_FDCWD, "file1", unix.R_OK, 2)
	if err != unix.EINVAL {
		t.Errorf("Faccessat: unexpected error: %v, want EINVAL", err)
	}

	err = unix.Faccessat(unix.AT_FDCWD, "file1", unix.R_OK, unix.AT_EACCESS)
	if err != nil {
		t.Errorf("Faccessat: unexpected error: %v", err)
	}

	err = os.Symlink("file1", "symlink1")
	if err != nil {
		t.Fatal(err)
	}

	err = unix.Faccessat(unix.AT_FDCWD, "symlink1", unix.R_OK, unix.AT_SYMLINK_NOFOLLOW)
	if err != nil {
		t.Errorf("Faccessat SYMLINK_NOFOLLOW: unexpected error %v", err)
	}

	// We can't really test AT_SYMLINK_NOFOLLOW, because there
	// doesn't seem to be any way to change the mode of a symlink.
	// We don't test AT_EACCESS because such tests are only
	// meaningful if run as root.

	err = unix.Fchmodat(unix.AT_FDCWD, "file1", 0, 0)
	if err != nil {
		t.Errorf("Fchmodat: unexpected error %v", err)
	}

	err = unix.Faccessat(unix.AT_FDCWD, "file1", unix.F_OK, unix.AT_SYMLINK_NOFOLLOW)
	if err != nil {
		t.Errorf("Faccessat: unexpected error: %v", err)
	}

	err = unix.Faccessat(unix.AT_FDCWD, "file1", unix.R_OK, unix.AT_SYMLINK_NOFOLLOW)
	if err != unix.EACCES {
		if unix.Getuid() != 0 {
			t.Errorf("Faccessat: unexpected error: %v, want EACCES", err)
		}
	}
}

func TestSyncFileRange(t *testing.T) {
	file, err := os.Create(filepath.Join(t.TempDir(), t.Name()))
	if err != nil {
		t.Fatal(err)
	}
	defer file.Close()

	err = unix.SyncFileRange(int(file.Fd()), 0, 0, 0)
	if err == unix.ENOSYS || err == unix.EPERM {
		t.Skip("sync_file_range syscall is not available, skipping test")
	} else if err != nil {
		t.Fatalf("SyncFileRange: %v", err)
	}

	// invalid flags
	flags := 0xf00
	err = unix.SyncFileRange(int(file.Fd()), 0, 0, flags)
	if err != unix.EINVAL {
		t.Fatalf("SyncFileRange: unexpected error: %v, want EINVAL", err)
	}
}

func TestClockNanosleep(t *testing.T) {
	delay := 50 * time.Millisecond

	// Relative timespec.
	start := time.Now()
	rel := unix.NsecToTimespec(delay.Nanoseconds())
	remain := unix.Timespec{}
	for {
		err := unix.ClockNanosleep(unix.CLOCK_MONOTONIC, 0, &rel, &remain)
		if err == unix.ENOSYS || err == unix.EPERM {
			t.Skip("clock_nanosleep syscall is not available, skipping test")
		} else if err == unix.EINTR {
			t.Logf("ClockNanosleep interrupted after %v", time.Since(start))
			rel = remain
			continue
		} else if err != nil {
			t.Errorf("ClockNanosleep(CLOCK_MONOTONIC, 0, %#v, nil) = %v", &rel, err)
		} else if slept := time.Since(start); slept < delay {
			t.Errorf("ClockNanosleep(CLOCK_MONOTONIC, 0, %#v, nil) slept only %v", &rel, slept)
		}
		break
	}

	// Absolute timespec.
	for {
		start = time.Now()
		until := start.Add(delay)
		abs := unix.NsecToTimespec(until.UnixNano())
		err := unix.ClockNanosleep(unix.CLOCK_REALTIME, unix.TIMER_ABSTIME, &abs, nil)
		if err == unix.EINTR {
			t.Logf("ClockNanosleep interrupted after %v", time.Since(start))
			continue
		} else if err != nil {
			t.Errorf("ClockNanosleep(CLOCK_REALTIME, TIMER_ABSTIME, %#v (=%v), nil) = %v", &abs, until, err)
		} else {
			// We asked for CLOCK_REALTIME, but we have no way to know whether it
			// jumped backward after ClockNanosleep returned. Compare both ways,
			// and only fail if both the monotonic and wall clocks agree that
			// the elapsed sleep was too short.
			//
			// This can still theoretically fail spuriously: if the clock jumps
			// forward during ClockNanosleep and then backward again before we can
			// call time.Now, then we could end up with a time that is too short on
			// both the monotonic scale (because of the forward jump) and the
			// real-time scale (because of the backward jump. However, it seems
			// unlikely that two such contrary jumps will ever occur in the time it
			// takes to execute this test.
			if now := time.Now(); now.Before(until) && now.Round(0).Before(until) {
				t.Errorf("ClockNanosleep(CLOCK_REALTIME, TIMER_ABSTIME, %#v (=%v), nil) slept only until %v", &abs, until, now)
			}
		}
		break
	}

	// Invalid clock. clock_nanosleep(2) says EINVAL, but it’s actually EOPNOTSUPP.
	err := unix.ClockNanosleep(unix.CLOCK_THREAD_CPUTIME_ID, 0, &rel, nil)
	if err != unix.EINVAL && err != unix.EOPNOTSUPP {
		t.Errorf("ClockNanosleep(CLOCK_THREAD_CPUTIME_ID, 0, %#v, nil) = %v, want EINVAL or EOPNOTSUPP", &rel, err)
	}
}

func TestOpenByHandleAt(t *testing.T) {
	skipIfNotSupported := func(t *testing.T, name string, err error) {
		if err == unix.EPERM {
			t.Skipf("skipping %s test without CAP_DAC_READ_SEARCH", name)
		}
		if err == unix.ENOSYS {
			t.Skipf("%s system call not available", name)
		}
		if err == unix.EOPNOTSUPP {
			t.Skipf("%s not supported on this filesystem", name)
		}
	}

	h, mountID, err := unix.NameToHandleAt(unix.AT_FDCWD, "syscall_linux_test.go", 0)
	if err != nil {
		skipIfNotSupported(t, "name_to_handle_at", err)
		t.Fatalf("NameToHandleAt: %v", err)
	}
	t.Logf("mountID: %v, handle: size=%d, type=%d, bytes=%q", mountID,
		h.Size(), h.Type(), h.Bytes())
	mount, err := openMountByID(mountID)
	if err != nil {
		t.Fatalf("openMountByID: %v", err)
	}
	defer mount.Close()

	for _, clone := range []bool{false, true} {
		t.Run("clone="+strconv.FormatBool(clone), func(t *testing.T) {
			if clone {
				h = unix.NewFileHandle(h.Type(), h.Bytes())
			}
			fd, err := unix.OpenByHandleAt(int(mount.Fd()), h, unix.O_RDONLY)
			skipIfNotSupported(t, "open_by_handle_at", err)
			if err != nil {
				t.Fatalf("OpenByHandleAt: %v", err)
			}
			t.Logf("opened fd %v", fd)
			f := os.NewFile(uintptr(fd), "")
			defer f.Close()

			slurp, err := io.ReadAll(f)
			if err != nil {
				t.Fatal(err)
			}
			const substr = "Some substring for a test."
			if !strings.Contains(string(slurp), substr) {
				t.Errorf("didn't find substring %q in opened file; read %d bytes", substr, len(slurp))
			}
		})
	}
}

func openMountByID(mountID int) (f *os.File, err error) {
	mi, err := os.Open("/proc/self/mountinfo")
	if err != nil {
		return nil, err
	}
	defer mi.Close()
	bs := bufio.NewScanner(mi)
	wantPrefix := []byte(fmt.Sprintf("%v ", mountID))
	for bs.Scan() {
		if !bytes.HasPrefix(bs.Bytes(), wantPrefix) {
			continue
		}
		fields := strings.Fields(bs.Text())
		dev := fields[4]
		return os.Open(dev)
	}
	if err := bs.Err(); err != nil {
		return nil, err
	}
	return nil, errors.New("mountID not found")
}

func TestEpoll(t *testing.T) {
	efd, err := unix.EpollCreate1(unix.EPOLL_CLOEXEC)
	if err != nil {
		t.Fatalf("EpollCreate1: %v", err)
	}
	defer unix.Close(efd)

	r, w, err := os.Pipe()
	if err != nil {
		t.Fatal(err)
	}
	defer r.Close()
	defer w.Close()

	fd := int(r.Fd())
	ev := unix.EpollEvent{Events: unix.EPOLLIN, Fd: int32(fd)}

	err = unix.EpollCtl(efd, unix.EPOLL_CTL_ADD, fd, &ev)
	if err != nil {
		t.Fatalf("EpollCtl: %v", err)
	}

	if _, err := w.Write([]byte("HELLO GOPHER")); err != nil {
		t.Fatal(err)
	}

	events := make([]unix.EpollEvent, 128)
	n, err := unix.EpollWait(efd, events, 1)
	if err != nil {
		t.Fatalf("EpollWait: %v", err)
	}

	if n != 1 {
		t.Errorf("EpollWait: wrong number of events: got %v, expected 1", n)
	}

	got := int(events[0].Fd)
	if got != fd {
		t.Errorf("EpollWait: wrong Fd in event: got %v, expected %v", got, fd)
	}
}

func TestPrctlRetInt(t *testing.T) {
	skipc := make(chan bool, 1)
	skip := func() {
		skipc <- true
		runtime.Goexit()
	}

	go func() {
		// This test uses prctl to modify the calling thread, so run it on its own
		// throwaway thread and do not unlock it when the goroutine exits.
		runtime.LockOSThread()
		defer close(skipc)

		err := unix.Prctl(unix.PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
		if err != nil {
			t.Logf("Prctl: %v, skipping test", err)
			skip()
		}

		v, err := unix.PrctlRetInt(unix.PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0)
		if err != nil {
			t.Errorf("failed to perform prctl: %v", err)
		}
		if v != 1 {
			t.Errorf("unexpected return from prctl; got %v, expected %v", v, 1)
		}
	}()

	if <-skipc {
		t.SkipNow()
	}
}

func TestTimerfd(t *testing.T) {
	var now unix.Timespec
	if err := unix.ClockGettime(unix.CLOCK_REALTIME, &now); err != nil {
		t.Fatalf("ClockGettime: %v", err)
	}

	tfd, err := unix.TimerfdCreate(unix.CLOCK_REALTIME, 0)
	if err == unix.ENOSYS {
		t.Skip("timerfd_create system call not implemented")
	} else if err != nil {
		t.Fatalf("TimerfdCreate: %v", err)
	}
	defer unix.Close(tfd)

	var timeSpec unix.ItimerSpec
	if err := unix.TimerfdGettime(tfd, &timeSpec); err != nil {
		t.Fatalf("TimerfdGettime: %v", err)
	}

	if timeSpec.Value.Nsec != 0 || timeSpec.Value.Sec != 0 {
		t.Fatalf("TimerfdGettime: timer is already set, but shouldn't be")
	}

	timeSpec = unix.ItimerSpec{
		Interval: unix.NsecToTimespec(int64(time.Millisecond)),
		Value:    now,
	}

	if err := unix.TimerfdSettime(tfd, unix.TFD_TIMER_ABSTIME, &timeSpec, nil); err != nil {
		t.Fatalf("TimerfdSettime: %v", err)
	}

	const totalTicks = 10
	const bufferLength = 8

	buffer := make([]byte, bufferLength)

	var count uint64 = 0
	for count < totalTicks {
		n, err := unix.Read(tfd, buffer)
		if err != nil {
			t.Fatalf("Timerfd: %v", err)
		} else if n != bufferLength {
			t.Fatalf("Timerfd: got %d bytes from timerfd, expected %d bytes", n, bufferLength)
		}

		count += *(*uint64)(unsafe.Pointer(&buffer))
	}
}

func TestOpenat2(t *testing.T) {
	how := &unix.OpenHow{
		Flags: unix.O_RDONLY,
	}
	fd, err := unix.Openat2(unix.AT_FDCWD, ".", how)
	if err != nil {
		if err == unix.ENOSYS || err == unix.EPERM {
			t.Skipf("openat2: %v (old kernel? need Linux >= 5.6)", err)
		}
		t.Fatalf("openat2: %v", err)
	}
	if err := unix.Close(fd); err != nil {
		t.Fatalf("close: %v", err)
	}

	// prepare
	subdir := filepath.Join(t.TempDir(), "dir")
	if err := os.Mkdir(subdir, 0755); err != nil {
		t.Fatal(err)
	}
	symlink := filepath.Join(subdir, "symlink")
	if err := os.Symlink("../", symlink); err != nil {
		t.Fatal(err)
	}

	dirfd, err := unix.Open(subdir, unix.O_RDONLY, 0)
	if err != nil {
		t.Fatalf("open(%q): %v", subdir, err)
	}
	defer unix.Close(dirfd)

	// openat2 with no extra flags -- should succeed
	fd, err = unix.Openat2(dirfd, "symlink", how)
	if err != nil {
		t.Errorf("Openat2 should succeed, got %v", err)
	}
	if err := unix.Close(fd); err != nil {
		t.Fatalf("close: %v", err)
	}

	// open with RESOLVE_BENEATH, should result in EXDEV
	how.Resolve = unix.RESOLVE_BENEATH
	fd, err = unix.Openat2(dirfd, "symlink", how)
	if err == nil {
		if err := unix.Close(fd); err != nil {
			t.Fatalf("close: %v", err)
		}
	}
	if err != unix.EXDEV {
		t.Errorf("Openat2 should fail with EXDEV, got %v", err)
	}
}

func TestIoctlFileDedupeRange(t *testing.T) {
	dir := t.TempDir()
	f1, err := os.Create(filepath.Join(dir, "f1"))
	if err != nil {
		t.Fatal(err)
	}
	defer f1.Close()

	// Test deduplication with two blocks of zeros
	data := make([]byte, 4096)

	for i := 0; i < 2; i += 1 {
		_, err = f1.Write(data)
		if err != nil {
			t.Fatal(err)
		}
	}

	f2, err := os.Create(filepath.Join(dir, "f2"))
	if err != nil {
		t.Fatal(err)
	}
	defer f2.Close()

	for i := 0; i < 2; i += 1 {
		// Make the 2nd block different
		if i == 1 {
			data[1] = 1
		}

		_, err = f2.Write(data)
		if err != nil {
			t.Fatal(err)
		}
	}

	dedupe := unix.FileDedupeRange{
		Src_offset: uint64(0),
		Src_length: uint64(4096),
		Info: []unix.FileDedupeRangeInfo{
			unix.FileDedupeRangeInfo{
				Dest_fd:     int64(f2.Fd()),
				Dest_offset: uint64(0),
			},
			unix.FileDedupeRangeInfo{
				Dest_fd:     int64(f2.Fd()),
				Dest_offset: uint64(4096),
			},
		}}

	err = unix.IoctlFileDedupeRange(int(f1.Fd()), &dedupe)
	if err == unix.EOPNOTSUPP || err == unix.EINVAL || err == unix.ENOTTY {
		t.Skip("deduplication not supported on this filesystem")
	} else if err != nil {
		t.Fatal(err)
	}

	// The first Info should be equal
	if dedupe.Info[0].Status < 0 {
		errno := unix.Errno(-dedupe.Info[0].Status)
		if errno == unix.EINVAL || errno == unix.EOPNOTSUPP {
			t.Skip("deduplication not supported on this filesystem")
		}
		t.Errorf("Unexpected error in FileDedupeRange: %s", unix.ErrnoName(errno))
	} else if dedupe.Info[0].Status == unix.FILE_DEDUPE_RANGE_DIFFERS {
		t.Errorf("Unexpected different bytes in FileDedupeRange")
	}
	if dedupe.Info[0].Bytes_deduped != 4096 {
		t.Errorf("Unexpected amount of bytes deduped %v != %v",
			dedupe.Info[0].Bytes_deduped, 4096)
	}

	// The second Info should be different
	if dedupe.Info[1].Status < 0 {
		errno := unix.Errno(-dedupe.Info[1].Status)
		if errno == unix.EINVAL || errno == unix.EOPNOTSUPP {
			t.Skip("deduplication not supported on this filesystem")
		}
		t.Errorf("Unexpected error in FileDedupeRange: %s", unix.ErrnoName(errno))
	} else if dedupe.Info[1].Status == unix.FILE_DEDUPE_RANGE_SAME {
		t.Errorf("Unexpected equal bytes in FileDedupeRange")
	}
	if dedupe.Info[1].Bytes_deduped != 0 {
		t.Errorf("Unexpected amount of bytes deduped %v != %v",
			dedupe.Info[1].Bytes_deduped, 0)
	}
}

// TestPwritevOffsets tests golang.org/issues/57291 where
// offs2lohi was shifting by the size of long in bytes, not bits.
func TestPwritevOffsets(t *testing.T) {
	path := filepath.Join(t.TempDir(), "x.txt")

	f, err := os.Create(path)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { f.Close() })

	const (
		off = 20
	)
	b := [][]byte{{byte(0)}}
	n, err := unix.Pwritev(int(f.Fd()), b, off)
	if err != nil {
		t.Fatal(err)
	}
	if n != len(b) {
		t.Fatalf("expected to write %d, wrote %d", len(b), n)
	}

	info, err := f.Stat()
	if err != nil {
		t.Fatal(err)
	}
	want := off + int64(len(b))
	if info.Size() != want {
		t.Fatalf("expected size to be %d, got %d", want, info.Size())
	}
}

func TestReadvAllocate(t *testing.T) {
	f, err := os.Create(filepath.Join(t.TempDir(), "test"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { f.Close() })

	test := func(name string, fn func(fd int)) {
		n := int(testing.AllocsPerRun(100, func() {
			fn(int(f.Fd()))
		}))
		if n != 0 {
			t.Errorf("%q got %d allocations, want 0", name, n)
		}
	}

	iovs := make([][]byte, 8)
	for i := range iovs {
		iovs[i] = []byte{'A'}
	}

	test("Writev", func(fd int) {
		unix.Writev(fd, iovs)
	})
	test("Pwritev", func(fd int) {
		unix.Pwritev(fd, iovs, 0)
	})
	test("Pwritev2", func(fd int) {
		unix.Pwritev2(fd, iovs, 0, 0)
	})
	test("Readv", func(fd int) {
		unix.Readv(fd, iovs)
	})
	test("Preadv", func(fd int) {
		unix.Preadv(fd, iovs, 0)
	})
	test("Preadv2", func(fd int) {
		unix.Preadv2(fd, iovs, 0, 0)
	})
}

func TestSockaddrALG(t *testing.T) {
	// Open a socket to perform SHA1 hashing.
	fd, err := unix.Socket(unix.AF_ALG, unix.SOCK_SEQPACKET, 0)
	if err != nil {
		t.Skip("socket(AF_ALG):", err)
	}
	defer unix.Close(fd)
	addr := &unix.SockaddrALG{Type: "hash", Name: "sha1"}
	if err := unix.Bind(fd, addr); err != nil {
		t.Fatal("bind:", err)
	}
	// Need to call accept(2) with the second and third arguments as 0,
	// which is not possible via unix.Accept, thus the use of unix.Syscall.
	hashfd, _, errno := unix.Syscall6(unix.SYS_ACCEPT4, uintptr(fd), 0, 0, 0, 0, 0)
	if errno != 0 {
		t.Fatal("accept:", errno)
	}

	hash := os.NewFile(hashfd, "sha1")
	defer hash.Close()

	// Hash an input string and read the results.
	const (
		input = "Hello, world."
		exp   = "2ae01472317d1935a84797ec1983ae243fc6aa28"
	)
	if _, err := hash.WriteString(input); err != nil {
		t.Fatal(err)
	}
	b := make([]byte, 20)
	if _, err := hash.Read(b); err != nil {
		t.Fatal(err)
	}
	got := hex.EncodeToString(b)
	if got != exp {
		t.Fatalf("got: %q, want: %q", got, exp)
	}
}
