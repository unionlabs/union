// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build aix || darwin || dragonfly || freebsd || linux || netbsd || openbsd || solaris

package unix_test

import (
	"bytes"
	"flag"
	"fmt"
	"io"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"runtime"
	"strconv"
	"sync"
	"syscall"
	"testing"
	"time"

	"golang.org/x/sys/unix"
)

// Tests that below functions, structures and constants are consistent
// on all Unix-like systems.
func _() {
	// program scheduling priority functions and constants
	var (
		_ func(int, int, int) error   = unix.Setpriority
		_ func(int, int) (int, error) = unix.Getpriority
	)
	const (
		_ int = unix.PRIO_USER
		_ int = unix.PRIO_PROCESS
		_ int = unix.PRIO_PGRP
	)

	// termios constants
	const (
		_ int = unix.TCIFLUSH
		_ int = unix.TCIOFLUSH
		_ int = unix.TCOFLUSH
	)

	// fcntl file locking structure and constants
	var (
		_ = unix.Flock_t{
			Type:   int16(0),
			Whence: int16(0),
			Start:  int64(0),
			Len:    int64(0),
			Pid:    int32(0),
		}
	)
	const (
		_ = unix.F_GETLK
		_ = unix.F_SETLK
		_ = unix.F_SETLKW
	)
}

func TestErrnoSignalName(t *testing.T) {
	testErrors := []struct {
		num  syscall.Errno
		name string
	}{
		{syscall.EPERM, "EPERM"},
		{syscall.EINVAL, "EINVAL"},
		{syscall.ENOENT, "ENOENT"},
	}

	for _, te := range testErrors {
		t.Run(fmt.Sprintf("%d/%s", te.num, te.name), func(t *testing.T) {
			e := unix.ErrnoName(te.num)
			if e != te.name {
				t.Errorf("ErrnoName(%d) returned %s, want %s", te.num, e, te.name)
			}
		})
	}

	testSignals := []struct {
		num  syscall.Signal
		name string
	}{
		{syscall.SIGHUP, "SIGHUP"},
		{syscall.SIGPIPE, "SIGPIPE"},
		{syscall.SIGSEGV, "SIGSEGV"},
	}

	for _, ts := range testSignals {
		t.Run(fmt.Sprintf("%d/%s", ts.num, ts.name), func(t *testing.T) {
			s := unix.SignalName(ts.num)
			if s != ts.name {
				t.Errorf("SignalName(%d) returned %s, want %s", ts.num, s, ts.name)
			}
		})
	}
}

func TestSignalNum(t *testing.T) {
	testSignals := []struct {
		name string
		want syscall.Signal
	}{
		{"SIGHUP", syscall.SIGHUP},
		{"SIGPIPE", syscall.SIGPIPE},
		{"SIGSEGV", syscall.SIGSEGV},
		{"NONEXISTS", 0},
	}
	for _, ts := range testSignals {
		t.Run(fmt.Sprintf("%s/%d", ts.name, ts.want), func(t *testing.T) {
			got := unix.SignalNum(ts.name)
			if got != ts.want {
				t.Errorf("SignalNum(%s) returned %d, want %d", ts.name, got, ts.want)
			}
		})

	}
}

func TestFcntlInt(t *testing.T) {
	t.Parallel()
	file, err := os.Create(filepath.Join(t.TempDir(), t.Name()))
	if err != nil {
		t.Fatal(err)
	}
	defer file.Close()
	f := file.Fd()
	flags, err := unix.FcntlInt(f, unix.F_GETFD, 0)
	if err != nil {
		t.Fatal(err)
	}
	if flags&unix.FD_CLOEXEC == 0 {
		t.Errorf("flags %#x do not include FD_CLOEXEC", flags)
	}
}

// TestFcntlFlock tests whether the file locking structure matches
// the calling convention of each kernel.
func TestFcntlFlock(t *testing.T) {
	name := filepath.Join(t.TempDir(), "TestFcntlFlock")
	fd, err := unix.Open(name, unix.O_CREAT|unix.O_RDWR|unix.O_CLOEXEC, 0)
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer unix.Close(fd)
	flock := unix.Flock_t{
		Type:  unix.F_RDLCK,
		Start: 0, Len: 0, Whence: 1,
	}
	if err := unix.FcntlFlock(uintptr(fd), unix.F_GETLK, &flock); err != nil {
		t.Fatalf("FcntlFlock failed: %v", err)
	}
}

// TestPassFD tests passing a file descriptor over a Unix socket.
//
// This test involved both a parent and child process. The parent
// process is invoked as a normal test, with "go test", which then
// runs the child process by running the current test binary with args
// "-test.run=^TestPassFD$" and an environment variable used to signal
// that the test should become the child process instead.
func TestPassFD(t *testing.T) {
	if runtime.GOOS == "ios" {
		t.Skip("cannot exec subprocess on iOS, skipping test")
	}

	if os.Getenv("GO_WANT_HELPER_PROCESS") == "1" {
		passFDChild()
		return
	}

	if runtime.GOOS == "aix" {
		// Unix network isn't properly working on AIX
		// 7.2 with Technical Level < 2
		out, err := exec.Command("oslevel", "-s").Output()
		if err != nil {
			t.Skipf("skipping on AIX because oslevel -s failed: %v", err)
		}

		if len(out) < len("7200-XX-ZZ-YYMM") { // AIX 7.2, Tech Level XX, Service Pack ZZ, date YYMM
			t.Skip("skipping on AIX because oslevel -s hasn't the right length")
		}
		aixVer := string(out[:4])
		tl, err := strconv.Atoi(string(out[5:7]))
		if err != nil {
			t.Skipf("skipping on AIX because oslevel -s output cannot be parsed: %v", err)
		}
		if aixVer < "7200" || (aixVer == "7200" && tl < 2) {
			t.Skip("skipped on AIX versions previous to 7.2 TL 2")
		}
	}

	fds, err := unix.Socketpair(unix.AF_LOCAL, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatalf("Socketpair: %v", err)
	}
	writeFile := os.NewFile(uintptr(fds[0]), "child-writes")
	readFile := os.NewFile(uintptr(fds[1]), "parent-reads")
	defer writeFile.Close()
	defer readFile.Close()

	exe, err := os.Executable()
	if err != nil {
		t.Fatal(err)
	}
	cmd := exec.Command(exe, "-test.run=^TestPassFD$", "--", t.TempDir())
	cmd.Env = []string{"GO_WANT_HELPER_PROCESS=1"}
	if lp := os.Getenv("LD_LIBRARY_PATH"); lp != "" {
		cmd.Env = append(cmd.Env, "LD_LIBRARY_PATH="+lp)
	}
	cmd.ExtraFiles = []*os.File{writeFile}

	out, err := cmd.CombinedOutput()
	if len(out) > 0 || err != nil {
		t.Fatalf("child process: %q, %v", out, err)
	}

	c, err := net.FileConn(readFile)
	if err != nil {
		t.Fatalf("FileConn: %v", err)
	}
	defer c.Close()

	uc, ok := c.(*net.UnixConn)
	if !ok {
		t.Fatalf("unexpected FileConn type; expected UnixConn, got %T", c)
	}

	buf := make([]byte, 32) // expect 1 byte
	oob := make([]byte, 32) // expect 24 bytes
	closeUnix := time.AfterFunc(5*time.Second, func() {
		t.Logf("timeout reading from unix socket")
		uc.Close()
	})
	_, oobn, _, _, err := uc.ReadMsgUnix(buf, oob)
	if err != nil {
		t.Fatalf("ReadMsgUnix: %v", err)
	}
	closeUnix.Stop()

	scms, err := unix.ParseSocketControlMessage(oob[:oobn])
	if err != nil {
		t.Fatalf("ParseSocketControlMessage: %v", err)
	}
	if len(scms) != 1 {
		t.Fatalf("expected 1 SocketControlMessage; got scms = %#v", scms)
	}
	scm := scms[0]
	gotFds, err := unix.ParseUnixRights(&scm)
	if err != nil {
		t.Fatalf("unix.ParseUnixRights: %v", err)
	}
	if len(gotFds) != 1 {
		t.Fatalf("wanted 1 fd; got %#v", gotFds)
	}

	f := os.NewFile(uintptr(gotFds[0]), "fd-from-child")
	defer f.Close()

	got, err := io.ReadAll(f)
	want := "Hello from child process!\n"
	if string(got) != want {
		t.Errorf("child process ReadAll: %q, %v; want %q", got, err, want)
	}
}

// passFDChild is the child process used by TestPassFD.
func passFDChild() {
	defer os.Exit(0)

	// Look for our fd. It should be fd 3, but we work around an fd leak
	// bug here (http://golang.org/issue/2603) to let it be elsewhere.
	var uc *net.UnixConn
	for fd := uintptr(3); fd <= 10; fd++ {
		f := os.NewFile(fd, "unix-conn")
		var ok bool
		netc, _ := net.FileConn(f)
		uc, ok = netc.(*net.UnixConn)
		if ok {
			break
		}
	}
	if uc == nil {
		fmt.Println("failed to find unix fd")
		return
	}

	// Make a file f to send to our parent process on uc.
	// We make it in tempDir, which our parent will clean up.
	flag.Parse()
	tempDir := flag.Arg(0)
	f, err := os.Create(filepath.Join(tempDir, "file"))
	if err != nil {
		fmt.Println(err)
		return
	}

	f.Write([]byte("Hello from child process!\n"))
	f.Seek(0, 0)

	rights := unix.UnixRights(int(f.Fd()))
	dummyByte := []byte("x")
	n, oobn, err := uc.WriteMsgUnix(dummyByte, rights, nil)
	if err != nil {
		fmt.Printf("WriteMsgUnix: %v", err)
		return
	}
	if n != 1 || oobn != len(rights) {
		fmt.Printf("WriteMsgUnix = %d, %d; want 1, %d", n, oobn, len(rights))
		return
	}
}

// TestUnixRightsRoundtrip tests that UnixRights, ParseSocketControlMessage, ParseOneSocketControlMessage,
// and ParseUnixRights are able to successfully round-trip lists of file descriptors.
func TestUnixRightsRoundtrip(t *testing.T) {
	testCases := [...][][]int{
		{{42}},
		{{1, 2}},
		{{3, 4, 5}},
		{{}},
		{{1, 2}, {3, 4, 5}, {}, {7}},
	}
	for _, testCase := range testCases {
		b := []byte{}
		var n int
		for _, fds := range testCase {
			// Last assignment to n wins
			n = len(b) + unix.CmsgLen(4*len(fds))
			b = append(b, unix.UnixRights(fds...)...)
		}
		// Truncate b
		b = b[:n]

		scms, err := unix.ParseSocketControlMessage(b)
		if err != nil {
			t.Fatalf("ParseSocketControlMessage: %v", err)
		}
		if len(scms) != len(testCase) {
			t.Fatalf("expected %v SocketControlMessage; got scms = %#v", len(testCase), scms)
		}

		var c int
		for len(b) > 0 {
			hdr, data, remainder, err := unix.ParseOneSocketControlMessage(b)
			if err != nil {
				t.Fatalf("ParseOneSocketControlMessage: %v", err)
			}
			if scms[c].Header != hdr || !bytes.Equal(scms[c].Data, data) {
				t.Fatal("expected SocketControlMessage header and data to match")
			}
			b = remainder
			c++
		}
		if c != len(scms) {
			t.Fatalf("expected %d SocketControlMessages; got %d", len(scms), c)
		}

		for i, scm := range scms {
			gotFds, err := unix.ParseUnixRights(&scm)
			if err != nil {
				t.Fatalf("ParseUnixRights: %v", err)
			}
			wantFds := testCase[i]
			if len(gotFds) != len(wantFds) {
				t.Fatalf("expected %v fds, got %#v", len(wantFds), gotFds)
			}
			for j, fd := range gotFds {
				if fd != wantFds[j] {
					t.Fatalf("expected fd %v, got %v", wantFds[j], fd)
				}
			}
		}
	}
}

func TestRlimit(t *testing.T) {
	var rlimit, zero unix.Rlimit
	err := unix.Getrlimit(unix.RLIMIT_NOFILE, &rlimit)
	if err != nil {
		t.Fatalf("Getrlimit: save failed: %v", err)
	}
	if zero == rlimit {
		t.Fatalf("Getrlimit: save failed: got zero value %#v", rlimit)
	}
	set := rlimit
	set.Cur = set.Max - 1
	if (runtime.GOOS == "darwin" || runtime.GOOS == "ios") && set.Cur > 4096 {
		// rlim_min for RLIMIT_NOFILE should be equal to
		// or lower than kern.maxfilesperproc, which on
		// some machines are 4096. See #40564.
		set.Cur = 4096
	}
	err = unix.Setrlimit(unix.RLIMIT_NOFILE, &set)
	if err != nil {
		t.Fatalf("Setrlimit: set failed: %#v %v", set, err)
	}
	var get unix.Rlimit
	err = unix.Getrlimit(unix.RLIMIT_NOFILE, &get)
	if err != nil {
		t.Fatalf("Getrlimit: get failed: %v", err)
	}
	set = rlimit
	set.Cur = set.Max - 1
	if (runtime.GOOS == "darwin" || runtime.GOOS == "ios") && set.Cur > 4096 {
		set.Cur = 4096
	}
	if set != get {
		// Seems like Darwin requires some privilege to
		// increase the soft limit of rlimit sandbox, though
		// Setrlimit never reports an error.
		switch runtime.GOOS {
		case "darwin", "ios":
		default:
			t.Fatalf("Rlimit: change failed: wanted %#v got %#v", set, get)
		}
	}
	err = unix.Setrlimit(unix.RLIMIT_NOFILE, &rlimit)
	if err != nil {
		t.Fatalf("Setrlimit: restore failed: %#v %v", rlimit, err)
	}

	// make sure RLIM_INFINITY can be assigned to Rlimit members
	_ = unix.Rlimit{
		Cur: unix.RLIM_INFINITY,
		Max: unix.RLIM_INFINITY,
	}
}

func TestSeekFailure(t *testing.T) {
	_, err := unix.Seek(-1, 0, 0)
	if err == nil {
		t.Fatalf("Seek(-1, 0, 0) did not fail")
	}
	str := err.Error() // used to crash on Linux
	t.Logf("Seek: %v", str)
	if str == "" {
		t.Fatalf("Seek(-1, 0, 0) return error with empty message")
	}
}

func TestSetsockoptString(t *testing.T) {
	// should not panic on empty string, see issue #31277
	err := unix.SetsockoptString(-1, 0, 0, "")
	if err == nil {
		t.Fatalf("SetsockoptString: did not fail")
	}
}

func TestDup(t *testing.T) {
	file, err := os.Create(filepath.Join(t.TempDir(), t.Name()))
	if err != nil {
		t.Fatal(err)
	}
	defer file.Close()
	f := int(file.Fd())

	newFd, err := unix.Dup(f)
	if err != nil {
		t.Fatalf("Dup: %v", err)
	}

	// Create and reserve a file descriptor.
	// Dup2 automatically closes it before reusing it.
	nullFile, err := os.Open("/dev/null")
	if err != nil {
		t.Fatal(err)
	}
	dupFd := int(file.Fd())
	err = unix.Dup2(newFd, dupFd)
	if err != nil {
		t.Fatalf("Dup2: %v", err)
	}
	// Keep the dummy file open long enough to not be closed in
	// its finalizer.
	runtime.KeepAlive(nullFile)

	b1 := []byte("Test123")
	b2 := make([]byte, 7)
	_, err = unix.Write(dupFd, b1)
	if err != nil {
		t.Fatalf("Write to dup2 fd failed: %v", err)
	}
	_, err = unix.Seek(f, 0, 0)
	if err != nil {
		t.Fatalf("Seek failed: %v", err)
	}
	_, err = unix.Read(f, b2)
	if err != nil {
		t.Fatalf("Read back failed: %v", err)
	}
	if string(b1) != string(b2) {
		t.Errorf("Dup: stdout write not in file, expected %v, got %v", string(b1), string(b2))
	}
}

func TestPoll(t *testing.T) {
	if runtime.GOOS == "android" || runtime.GOOS == "ios" {
		t.Skip("mkfifo syscall is not available on android and iOS, skipping test")
	}

	chtmpdir(t)
	f := mktmpfifo(t)

	const timeout = 100

	ok := make(chan bool, 1)
	go func() {
		select {
		case <-time.After(10 * timeout * time.Millisecond):
			t.Errorf("Poll: failed to timeout after %d milliseconds", 10*timeout)
		case <-ok:
		}
	}()

	for {
		fds := []unix.PollFd{{Fd: int32(f.Fd()), Events: unix.POLLIN}}
		n, err := unix.Poll(fds, timeout)
		ok <- true
		if err == unix.EINTR {
			t.Logf("Poll interrupted")
			continue
		} else if err != nil {
			t.Errorf("Poll: unexpected error: %v", err)
			return
		}
		if n != 0 {
			t.Errorf("Poll: wrong number of events: got %v, expected %v", n, 0)

			// Identify which event(s) caused Poll to return.
			// We can't trivially use a table here because Revents
			// isn't the same type on all systems.
			if fds[0].Revents&unix.POLLIN != 0 {
				t.Log("found POLLIN event")
			}
			if fds[0].Revents&unix.POLLPRI != 0 {
				t.Log("found POLLPRI event")
			}
			if fds[0].Revents&unix.POLLOUT != 0 {
				t.Log("found POLLOUT event")
			}
			if fds[0].Revents&unix.POLLERR != 0 {
				t.Log("found POLLERR event")
			}
			if fds[0].Revents&unix.POLLHUP != 0 {
				t.Log("found POLLHUP event")
			}
			if fds[0].Revents&unix.POLLNVAL != 0 {
				t.Log("found POLLNVAL event")
			}
		}
		break
	}
}

func TestSelect(t *testing.T) {
	for {
		n, err := unix.Select(0, nil, nil, nil, &unix.Timeval{Sec: 0, Usec: 0})
		if err == unix.EINTR {
			t.Logf("Select interrupted")
			continue
		} else if err != nil {
			t.Fatalf("Select: %v", err)
		}
		if n != 0 {
			t.Fatalf("Select: got %v ready file descriptors, expected 0", n)
		}
		break
	}

	dur := 250 * time.Millisecond
	var took time.Duration
	for {
		// On some platforms (e.g. Linux), the passed-in timeval is
		// updated by select(2). Make sure to reset to the full duration
		// in case of an EINTR.
		tv := unix.NsecToTimeval(int64(dur))
		start := time.Now()
		n, err := unix.Select(0, nil, nil, nil, &tv)
		took = time.Since(start)
		if err == unix.EINTR {
			t.Logf("Select interrupted after %v", took)
			continue
		} else if err != nil {
			t.Fatalf("Select: %v", err)
		}
		if n != 0 {
			t.Fatalf("Select: got %v ready file descriptors, expected 0", n)
		}
		break
	}

	// On some platforms (e.g. NetBSD) the actual timeout might be arbitrarily
	// less than requested. However, Linux in particular promises to only return
	// early if a file descriptor becomes ready (not applicable here), or the call
	// is interrupted by a signal handler (explicitly retried in the loop above),
	// or the timeout expires.
	if took < dur {
		if runtime.GOOS == "linux" {
			t.Errorf("Select: slept for %v, expected %v", took, dur)
		} else {
			t.Logf("Select: slept for %v, requested %v", took, dur)
		}
	}

	rr, ww, err := os.Pipe()
	if err != nil {
		t.Fatal(err)
	}
	defer rr.Close()
	defer ww.Close()

	if _, err := ww.Write([]byte("HELLO GOPHER")); err != nil {
		t.Fatal(err)
	}

	rFdSet := &unix.FdSet{}
	fd := int(rr.Fd())
	rFdSet.Set(fd)

	for {
		n, err := unix.Select(fd+1, rFdSet, nil, nil, nil)
		if err == unix.EINTR {
			t.Log("Select interrupted")
			continue
		} else if err != nil {
			t.Fatalf("Select: %v", err)
		}
		if n != 1 {
			t.Fatalf("Select: got %v ready file descriptors, expected 1", n)
		}
		break
	}
}

func TestGetwd(t *testing.T) {
	fd, err := os.Open(".")
	if err != nil {
		t.Fatalf("Open .: %s", err)
	}
	defer fd.Close()
	// Directory list for test. Do not worry if any are symlinks or do not
	// exist on some common unix desktop environments. That will be checked.
	dirs := []string{"/", "/usr/bin", "/etc", "/var", "/opt"}
	switch runtime.GOOS {
	case "android":
		dirs = []string{"/", "/system/bin"}
	case "ios":
		dirs = []string{t.TempDir(), t.TempDir()}
	}
	oldwd := os.Getenv("PWD")
	for _, d := range dirs {
		// Check whether d exists, is a dir and that d's path does not contain a symlink
		fi, err := os.Stat(d)
		if err != nil || !fi.IsDir() {
			t.Logf("Test dir %s stat error (%v) or not a directory, skipping", d, err)
			continue
		}
		check, err := filepath.EvalSymlinks(d)
		if err != nil || check != d {
			t.Logf("Test dir %s (%s) is symlink or other error (%v), skipping", d, check, err)
			continue
		}
		err = os.Chdir(d)
		if err != nil {
			t.Fatalf("Chdir: %v", err)
		}
		pwd, err := unix.Getwd()
		if err != nil {
			t.Fatalf("Getwd in %s: %s", d, err)
		}
		os.Setenv("PWD", oldwd)
		err = fd.Chdir()
		if err != nil {
			// We changed the current directory and cannot go back.
			// Don't let the tests continue; they'll scribble
			// all over some other directory.
			fmt.Fprintf(os.Stderr, "fchdir back to dot failed: %s\n", err)
			os.Exit(1)
		}
		if pwd != d {
			t.Fatalf("Getwd returned %q want %q", pwd, d)
		}
	}
}

func compareStat_t(t *testing.T, otherStat string, st1, st2 *unix.Stat_t) {
	if st2.Dev != st1.Dev {
		t.Errorf("%s/Fstatat: got dev %v, expected %v", otherStat, st2.Dev, st1.Dev)
	}
	if st2.Ino != st1.Ino {
		t.Errorf("%s/Fstatat: got ino %v, expected %v", otherStat, st2.Ino, st1.Ino)
	}
	if st2.Mode != st1.Mode {
		t.Errorf("%s/Fstatat: got mode %v, expected %v", otherStat, st2.Mode, st1.Mode)
	}
	if st2.Uid != st1.Uid {
		t.Errorf("%s/Fstatat: got uid %v, expected %v", otherStat, st2.Uid, st1.Uid)
	}
	if st2.Gid != st1.Gid {
		t.Errorf("%s/Fstatat: got gid %v, expected %v", otherStat, st2.Gid, st1.Gid)
	}
	if st2.Size != st1.Size {
		t.Errorf("%s/Fstatat: got size %v, expected %v", otherStat, st2.Size, st1.Size)
	}
}

func TestFstatat(t *testing.T) {
	chtmpdir(t)

	touch(t, "file1")

	var st1 unix.Stat_t
	err := unix.Stat("file1", &st1)
	if err != nil {
		t.Fatalf("Stat: %v", err)
	}

	var st2 unix.Stat_t
	err = unix.Fstatat(unix.AT_FDCWD, "file1", &st2, 0)
	if err != nil {
		t.Fatalf("Fstatat: %v", err)
	}

	compareStat_t(t, "Stat", &st1, &st2)

	err = os.Symlink("file1", "symlink1")
	if err != nil {
		t.Fatal(err)
	}

	err = unix.Lstat("symlink1", &st1)
	if err != nil {
		t.Fatalf("Lstat: %v", err)
	}

	err = unix.Fstatat(unix.AT_FDCWD, "symlink1", &st2, unix.AT_SYMLINK_NOFOLLOW)
	if err != nil {
		t.Fatalf("Fstatat: %v", err)
	}

	compareStat_t(t, "Lstat", &st1, &st2)
}

func TestFchmodat(t *testing.T) {
	chtmpdir(t)

	touch(t, "file1")
	err := os.Symlink("file1", "symlink1")
	if err != nil {
		t.Fatal(err)
	}

	mode := os.FileMode(0444)
	err = unix.Fchmodat(unix.AT_FDCWD, "symlink1", uint32(mode), 0)
	if err != nil {
		t.Fatalf("Fchmodat: unexpected error: %v", err)
	}

	fi, err := os.Stat("file1")
	if err != nil {
		t.Fatal(err)
	}

	if fi.Mode() != mode {
		t.Errorf("Fchmodat: failed to change file mode: expected %v, got %v", mode, fi.Mode())
	}

	mode = os.FileMode(0644)
	didChmodSymlink := true
	err = unix.Fchmodat(unix.AT_FDCWD, "symlink1", uint32(mode), unix.AT_SYMLINK_NOFOLLOW)
	if err != nil {
		if (runtime.GOOS == "android" || runtime.GOOS == "linux" ||
			runtime.GOOS == "solaris" || runtime.GOOS == "illumos") && err == unix.EOPNOTSUPP {
			// Linux and Illumos don't support flags != 0
			didChmodSymlink = false
		} else {
			t.Fatalf("Fchmodat: unexpected error: %v", err)
		}
	}

	if !didChmodSymlink {
		// Didn't change mode of the symlink. On Linux, the permissions
		// of a symbolic link are always 0777 according to symlink(7)
		mode = os.FileMode(0777)
	}

	var st unix.Stat_t
	err = unix.Lstat("symlink1", &st)
	if err != nil {
		t.Fatal(err)
	}

	got := os.FileMode(st.Mode & 0777)
	if got != mode {
		t.Errorf("Fchmodat: failed to change symlink mode: expected %v, got %v", mode, got)
	}
}

func TestMkdev(t *testing.T) {
	major := uint32(42)
	minor := uint32(7)
	dev := unix.Mkdev(major, minor)

	if unix.Major(dev) != major {
		t.Errorf("Major(%#x) == %d, want %d", dev, unix.Major(dev), major)
	}
	if unix.Minor(dev) != minor {
		t.Errorf("Minor(%#x) == %d, want %d", dev, unix.Minor(dev), minor)
	}
}

func TestPipe(t *testing.T) {
	const s = "hello"
	var pipes [2]int
	err := unix.Pipe(pipes[:])
	if err != nil {
		t.Fatalf("pipe: %v", err)
	}
	r := pipes[0]
	w := pipes[1]
	go func() {
		n, err := unix.Write(w, []byte(s))
		if err != nil {
			t.Errorf("bad write: %v", err)
			return
		}
		if n != len(s) {
			t.Errorf("bad write count: %d", n)
			return
		}
		err = unix.Close(w)
		if err != nil {
			t.Errorf("bad close: %v", err)
			return
		}
	}()
	var buf [10 + len(s)]byte
	n, err := unix.Read(r, buf[:])
	if err != nil {
		t.Fatalf("bad read: %v", err)
	}
	if n != len(s) {
		t.Fatalf("bad read count: %d", n)
	}
	if string(buf[:n]) != s {
		t.Fatalf("bad contents: %s", string(buf[:n]))
	}
	err = unix.Close(r)
	if err != nil {
		t.Fatalf("bad close: %v", err)
	}
}

func TestRenameat(t *testing.T) {
	chtmpdir(t)

	from, to := "renamefrom", "renameto"

	touch(t, from)

	err := unix.Renameat(unix.AT_FDCWD, from, unix.AT_FDCWD, to)
	if err != nil {
		t.Fatalf("Renameat: unexpected error: %v", err)
	}

	_, err = os.Stat(to)
	if err != nil {
		t.Error(err)
	}

	_, err = os.Stat(from)
	if err == nil {
		t.Errorf("Renameat: stat of renamed file %q unexpectedly succeeded", from)
	}
}

func TestUtimesNanoAt(t *testing.T) {
	chtmpdir(t)

	symlink := "symlink1"
	os.Remove(symlink)
	err := os.Symlink("nonexisting", symlink)
	if err != nil {
		t.Fatal(err)
	}

	// Some filesystems only support microsecond resolution. Account for
	// that in Nsec.
	ts := []unix.Timespec{
		{Sec: 1111, Nsec: 2000},
		{Sec: 3333, Nsec: 4000},
	}
	err = unix.UtimesNanoAt(unix.AT_FDCWD, symlink, ts, unix.AT_SYMLINK_NOFOLLOW)
	if err != nil {
		t.Fatalf("UtimesNanoAt: %v", err)
	}

	var st unix.Stat_t
	err = unix.Lstat(symlink, &st)
	if err != nil {
		t.Fatalf("Lstat: %v", err)
	}

	// Only check Mtim, Atim might not be supported by the underlying filesystem
	expected := ts[1]
	if st.Mtim.Nsec == 0 {
		// Some filesystems only support 1-second time stamp resolution
		// and will always set Nsec to 0.
		expected.Nsec = 0
	}
	if st.Mtim != expected {
		t.Errorf("UtimesNanoAt: wrong mtime: got %v, expected %v", st.Mtim, expected)
	}
}

func TestSend(t *testing.T) {
	ec := make(chan error, 2)
	ts := []byte("HELLO GOPHER")

	fds, err := unix.Socketpair(unix.AF_LOCAL, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatalf("Socketpair: %v", err)
	}
	defer unix.Close(fds[0])
	defer unix.Close(fds[1])

	go func() {
		data := make([]byte, len(ts))

		_, _, err := unix.Recvfrom(fds[1], data, 0)
		if err != nil {
			ec <- err
		}
		if !bytes.Equal(ts, data) {
			ec <- fmt.Errorf("data sent != data received. Received %q", data)
		}
		ec <- nil
	}()
	err = unix.Send(fds[0], ts, 0)
	if err != nil {
		ec <- err
	}

	select {
	case err = <-ec:
		if err != nil {
			t.Fatalf("Send: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("Send: nothing received after 2 seconds")
	}
}

func TestSendmsgBuffers(t *testing.T) {
	fds, err := unix.Socketpair(unix.AF_LOCAL, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatal(err)
	}
	defer unix.Close(fds[0])
	defer unix.Close(fds[1])

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		bufs := [][]byte{
			make([]byte, 5),
			nil,
			make([]byte, 5),
		}
		n, oobn, recvflags, _, err := unix.RecvmsgBuffers(fds[1], bufs, nil, 0)
		if err != nil {
			t.Error(err)
			return
		}
		if n != 10 {
			t.Errorf("got %d bytes, want 10", n)
		}
		if oobn != 0 {
			t.Errorf("got %d OOB bytes, want 0", oobn)
		}
		if recvflags != 0 {
			t.Errorf("got flags %#x, want %#x", recvflags, 0)
		}
		want := [][]byte{
			[]byte("01234"),
			nil,
			[]byte("56789"),
		}
		if !reflect.DeepEqual(bufs, want) {
			t.Errorf("got data %q, want %q", bufs, want)
		}
	}()

	defer wg.Wait()

	bufs := [][]byte{
		[]byte("012"),
		[]byte("34"),
		nil,
		[]byte("5678"),
		[]byte("9"),
	}
	n, err := unix.SendmsgBuffers(fds[0], bufs, nil, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	if n != 10 {
		t.Errorf("sent %d bytes, want 10", n)
	}
}

// Issue 56384.
func TestRecvmsgControl(t *testing.T) {
	switch runtime.GOOS {
	case "solaris", "illumos":
		// Test fails on Solaris, saying
		// "got 0 control messages, want 1".
		// Not sure why; Solaris recvmsg man page says
		// "For processes on the same host, recvmsg() can be
		// used to receive a file descriptor from another
		// process, but it cannot receive ancillary data."
		t.Skipf("skipping on %s", runtime.GOOS)
	}

	fds, err := unix.Socketpair(unix.AF_UNIX, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatal(err)
	}
	defer unix.Close(fds[0])
	defer unix.Close(fds[1])

	const payload = "hello"

	// Start a goroutine that sends a control message followed by
	// a payload on fds[1].
	go func() {
		f, err := os.Create(filepath.Join(t.TempDir(), "file"))
		if err != nil {
			t.Error(err)
			return
		}
		defer f.Close()

		rc, err := f.SyscallConn()
		if err != nil {
			t.Error(err)
			return
		}
		var rights []byte
		err = rc.Control(func(fd uintptr) {
			rights = unix.UnixRights(int(fd))
		})
		if err != nil {
			t.Error(err)
			return
		}

		_, err = unix.SendmsgN(fds[1], nil, rights, nil, 0)
		if err != nil {
			t.Error(err)
		}
		if _, err := unix.Write(fds[1], []byte(payload)); err != nil {
			t.Error(err)
		}
	}()

	// Read the control message sent by the goroutine.  The
	// goroutine writes to fds[1], we read from fds[0].

	cbuf := make([]byte, unix.CmsgSpace(4))
	_, cn, _, _, err := unix.Recvmsg(fds[0], nil, cbuf, 0)
	if err != nil {
		t.Fatal(err)
	}
	cbuf = cbuf[:cn]

	// Read the payload sent by the goroutine.

	buf := make([]byte, len(payload))
	n, err := unix.Read(fds[0], buf)
	if err != nil {
		t.Fatal(err)
	}
	buf = buf[:n]
	if payload != string(buf) {
		t.Errorf("read payload %q, want %q", buf, payload)
	}

	// Check the control message.

	cmsgs, err := unix.ParseSocketControlMessage(cbuf)
	if err != nil {
		t.Fatal(err)
	}
	if len(cmsgs) != 1 {
		t.Fatalf("got %d control messages, want 1", len(cmsgs))
	}
	cfds, err := unix.ParseUnixRights(&cmsgs[0])
	if err != nil {
		t.Fatal(err)
	}
	if len(cfds) != 1 {
		t.Fatalf("got %d fds, want 1", len(cfds))
	}
	defer unix.Close(cfds[0])
}

// mktmpfifo creates a temporary FIFO and sets up a cleanup function.
func mktmpfifo(t *testing.T) *os.File {
	t.Helper()
	err := unix.Mkfifo("fifo", 0666)
	if err != nil {
		t.Fatalf("mktmpfifo: failed to create FIFO: %v", err)
	}

	f, err := os.OpenFile("fifo", os.O_RDWR, 0666)
	if err != nil {
		os.Remove("fifo")
		t.Fatal(err)
	}
	t.Cleanup(func() {
		f.Close()
		os.Remove("fifo")
	})

	return f
}

// utilities taken from os/os_test.go

func touch(t *testing.T, name string) {
	t.Helper()
	f, err := os.Create(name)
	if err != nil {
		t.Fatal(err)
	}
	if err := f.Close(); err != nil {
		t.Fatal(err)
	}
}

// chtmpdir changes the working directory to a new temporary directory and
// sets up a cleanup function. Used when PWD is read-only.
func chtmpdir(t *testing.T) {
	t.Helper()
	oldwd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	if err := os.Chdir(t.TempDir()); err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := os.Chdir(oldwd); err != nil {
			t.Fatal(err)
		}
	})
}
