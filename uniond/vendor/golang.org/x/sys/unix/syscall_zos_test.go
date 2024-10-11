// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build zos && s390x

package unix_test

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"regexp"
	"runtime"
	"strconv"
	"strings"
	"syscall"
	"testing"
	"time"
	"unsafe"

	"golang.org/x/sys/unix"
)

func TestLibVec(t *testing.T) {
	ret := unix.GetZosLibVec()
	if ret == 0 {
		t.Fatalf("initLibVec failed %v\n", ret)
	}
}

func TestFprintf(t *testing.T) {
	const expected = 61
	ret, _, _ := unix.CallLeFuncWithErr(unix.GetZosLibVec()+unix.SYS___FPRINTF_A<<4,
		unix.ZosStdioFilep(2), uintptr(unsafe.Pointer(
			reflect.ValueOf([]byte("TEST DATA please ignore, fprintf stderr data %d %d %d %d\x0a\x00")).Pointer())),
		111, 222, 333, 444)
	if ret != expected {
		t.Fatalf("expected bytes written %d , receive %d\n", expected, int(ret))
	}
}

func TestErrnos(t *testing.T) {
	ret, err1, err2 := unix.CallLeFuncWithErr(unix.GetZosLibVec()+unix.SYS___OPEN_A<<4,
		uintptr(unsafe.Pointer(&(([]byte("/dev/nothing" + "\x00"))[0]))), 0, 0)
	if ret != 0xffffffffffffffff || err2 != 0x79 || err1 != 0x562003f {
		t.Fatalf("Expect ret ffffffffffffffff err2 79 err1 562003f, received  ret %x err2 %x err1 %x\n", ret, err2, err1)
	}
}

func TestErrnoString(t *testing.T) {
	var (
		ws  unix.WaitStatus
		rus unix.Rusage
	)
	_, err := unix.Wait4(-1, &ws, unix.WNOHANG, &rus)
	if syscall.Errno(int32(err.(syscall.Errno))) != unix.ECHILD {
		t.Fatalf("err != unix.ECHILD")
	}
}

func BypassTestOnUntil(sys string, date string) bool {
	t0, er0 := time.Parse(time.RFC3339, date)
	if er0 != nil {
		fmt.Printf("Bad date-time spec %s\n", date)
		return false
	}
	if time.Now().After(t0) {
		return false
	}
	host1, er1 := os.Hostname()
	hostname := strings.Split(host1, ".")[0]

	if er1 == nil && strings.EqualFold(hostname, sys) {
		pc, file, line, ok := runtime.Caller(1)
		if ok {
			name := runtime.FuncForPC(pc).Name()
			fmt.Fprintf(os.Stderr, "TODO: Test bypassed on %s %v:%v %v\n", hostname, file, line, name)
			return true
		}
	}
	return false

}

var euid = unix.Geteuid()

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

func zosLeVersion() (version, release uint32) {
	p1 := (*(*uintptr)(unsafe.Pointer(uintptr(1208)))) >> 32
	p1 = *(*uintptr)(unsafe.Pointer(uintptr(p1 + 88)))
	p1 = *(*uintptr)(unsafe.Pointer(uintptr(p1 + 8)))
	p1 = *(*uintptr)(unsafe.Pointer(uintptr(p1 + 984)))
	vrm := *(*uint32)(unsafe.Pointer(p1 + 80))
	version = (vrm & 0x00ff0000) >> 16
	release = (vrm & 0x0000ff00) >> 8
	return
}

func TestErrnoSignalName(t *testing.T) {
	testErrors := []struct {
		num  syscall.Errno
		name string
	}{
		{syscall.EPERM, "EDC5139I"},
		{syscall.EINVAL, "EDC5121I"},
		{syscall.ENOENT, "EDC5129I"},
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
	file, err := os.Create(filepath.Join(t.TempDir(), "TestFnctlInt"))
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
	name := filepath.Join(os.TempDir(), "TestFcntlFlock")
	fd, err := unix.Open(name, unix.O_CREAT|unix.O_RDWR|unix.O_CLOEXEC, 0)
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer unix.Unlink(name)
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
	if os.Getenv("GO_WANT_HELPER_PROCESS") == "1" {
		passFDChild()
		return
	}

	tempDir := t.TempDir()

	fds, err := unix.Socketpair(unix.AF_LOCAL, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatalf("Socketpair: %v", err)
	}
	defer unix.Close(fds[0])
	defer unix.Close(fds[1])
	writeFile := os.NewFile(uintptr(fds[0]), "child-writes")
	readFile := os.NewFile(uintptr(fds[1]), "parent-reads")
	defer writeFile.Close()
	defer readFile.Close()

	cmd := exec.Command(os.Args[0], "-test.run=^TestPassFD$", "--", tempDir)
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
	f, err := os.CreateTemp(tempDir, "")
	if err != nil {
		fmt.Printf("TempFile: %v", err)
		return
	}
	defer f.Close()

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

// TestUnixRightsRoundtrip tests that UnixRights, ParseSocketControlMessage,
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

func TestPrlimit(t *testing.T) {
	var rlimit, get, set, zero unix.Rlimit
	// Save initial settings
	err := unix.Prlimit(0, unix.RLIMIT_NOFILE, nil, &rlimit)
	if err != nil {
		t.Fatalf("Prlimit: save failed: %v", err)
	}
	if zero == rlimit {
		t.Fatalf("Prlimit: save failed: got zero value %#v", rlimit)
	}
	set = rlimit
	set.Cur = set.Max - 1
	// Set to one below max
	err = unix.Prlimit(0, unix.RLIMIT_NOFILE, &set, nil)
	if err != nil {
		t.Fatalf("Prlimit: set failed: %#v %v", set, err)
	}
	// Get and restore to original
	err = unix.Prlimit(0, unix.RLIMIT_NOFILE, &rlimit, &get)
	if err != nil {
		t.Fatalf("Prlimit: get and restore failed: %v", err)
	}
	if set != get {
		t.Fatalf("Rlimit: change failed: wanted %#v got %#v", set, get)
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
	if set != get {
		t.Fatalf("Rlimit: change failed: wanted %#v got %#v", set, get)
	}
	err = unix.Setrlimit(unix.RLIMIT_NOFILE, &rlimit)
	if err != nil {
		t.Fatalf("Setrlimit: restore failed: %#v %v", rlimit, err)
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
	file, err := os.Create(filepath.Join(t.TempDir(), "TestDup"))
	if err != nil {
		t.Fatalf("Tempfile failed: %v", err)
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
	defer nullFile.Close()

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

func TestGetwd(t *testing.T) {
	fd, err := os.Open(".")
	if err != nil {
		t.Fatalf("Open .: %s", err)
	}
	defer fd.Close()
	// Directory list for test. Do not worry if any are symlinks or do not
	// exist on some common unix desktop environments. That will be checked.
	dirs := []string{"/", "/usr/bin", "/etc", "/var", "/opt"}
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
func TestZosFdToPath(t *testing.T) {
	f, err := os.OpenFile("/tmp", os.O_RDONLY, 0755)
	if err != nil {
		t.Fatalf("Openfile %v", err)
	}
	defer f.Close()
	fd := f.Fd()

	var res string
	res, err = unix.ZosFdToPath(int(fd))
	if err != nil {
		t.Fatalf("ZosFdToPath %v", err)
	}
	chk := regexp.MustCompile(`^.*/([^\/]+)`).FindStringSubmatch(res)
	lastpath := chk[len(chk)-1]
	if lastpath != "tmp" {
		t.Fatalf("original %s last part of path \"%s\" received, expected \"tmp\" \n", res, lastpath)
	}
}

// mktmpfifo creates a temporary FIFO and provides a cleanup function.
func mktmpfifo(t *testing.T) (*os.File, func()) {
	err := unix.Mkfifo("fifo", 0666)
	if err != nil {
		t.Fatalf("mktmpfifo: failed to create FIFO: %v", err)
	}

	f, err := os.OpenFile("fifo", os.O_RDWR, 0666)
	if err != nil {
		os.Remove("fifo")
		t.Fatalf("mktmpfifo: failed to open FIFO: %v", err)
	}

	return f, func() {
		f.Close()
		os.Remove("fifo")
	}
}

// utilities taken from os/os_test.go

func touch(t *testing.T, name string) {
	f, err := os.Create(name)
	if err != nil {
		t.Fatal(err)
	}
	if err := f.Close(); err != nil {
		t.Fatal(err)
	}
}

// chtmpdir changes the working directory to a new temporary directory and
// provides a cleanup function. Used when PWD is read-only.
func chtmpdir(t *testing.T) func() {
	oldwd, err := os.Getwd()
	if err != nil {
		t.Fatalf("chtmpdir: %v", err)
	}
	d, err := os.MkdirTemp("", "test")
	if err != nil {
		t.Fatalf("chtmpdir: %v", err)
	}
	if err := os.Chdir(d); err != nil {
		t.Fatalf("chtmpdir: %v", err)
	}
	return func() {
		if err := os.Chdir(oldwd); err != nil {
			t.Fatalf("chtmpdir: %v", err)
		}
		os.RemoveAll(d)
	}
}

func TestLegacyMountUnmount(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}
	b2s := func(arr []byte) string {
		var str string
		for i := 0; i < len(arr); i++ {
			if arr[i] == 0 {
				str = string(arr[:i])
				break
			}
		}
		return str
	}
	// use an available fs
	var buffer struct {
		header unix.W_Mnth
		fsinfo [64]unix.W_Mntent
	}
	fs_count, err := unix.W_Getmntent_A((*byte)(unsafe.Pointer(&buffer)), int(unsafe.Sizeof(buffer)))
	if err != nil {
		t.Fatalf("W_Getmntent_A returns with error: %s", err.Error())
	} else if fs_count == 0 {
		t.Fatalf("W_Getmntent_A returns no entries")
	}
	var fs string
	var fstype string
	var mountpoint string
	var available bool = false
	for i := 0; i < fs_count; i++ {
		err = unix.Unmount(b2s(buffer.fsinfo[i].Mountpoint[:]), unix.MTM_RDWR)
		if err != nil {
			// Unmount and Mount require elevated privilege
			// If test is run without such permission, skip test
			if err == unix.EPERM {
				t.Logf("Permission denied for Unmount. Skipping test (Errno2:  %X)", unix.Errno2())
				return
			} else if err == unix.EBUSY {
				continue
			} else {
				t.Fatalf("Unmount returns with error: %s", err.Error())
			}
		} else {
			available = true
			fs = b2s(buffer.fsinfo[i].Fsname[:])
			fstype = b2s(buffer.fsinfo[i].Fstname[:])
			mountpoint = b2s(buffer.fsinfo[i].Mountpoint[:])
			t.Logf("using file system = %s; fstype = %s and mountpoint = %s\n", fs, fstype, mountpoint)
			break
		}
	}
	if !available {
		t.Fatalf("No filesystem available")
	}
	// test unmount
	buffer.header = unix.W_Mnth{}
	fs_count, err = unix.W_Getmntent_A((*byte)(unsafe.Pointer(&buffer)), int(unsafe.Sizeof(buffer)))
	if err != nil {
		t.Fatalf("W_Getmntent_A returns with error: %s", err.Error())
	}
	for i := 0; i < fs_count; i++ {
		if b2s(buffer.fsinfo[i].Fsname[:]) == fs {
			t.Fatalf("File system found after unmount")
		}
	}
	// test mount
	err = unix.Mount(fs, mountpoint, fstype, unix.MTM_RDWR, "")
	if err != nil {
		t.Fatalf("Mount returns with error: %s", err.Error())
	}
	buffer.header = unix.W_Mnth{}
	fs_count, err = unix.W_Getmntent_A((*byte)(unsafe.Pointer(&buffer)), int(unsafe.Sizeof(buffer)))
	if err != nil {
		t.Fatalf("W_Getmntent_A returns with error: %s", err.Error())
	}
	fs_mounted := false
	for i := 0; i < fs_count; i++ {
		if b2s(buffer.fsinfo[i].Fsname[:]) == fs && b2s(buffer.fsinfo[i].Mountpoint[:]) == mountpoint {
			fs_mounted = true
		}
	}
	if !fs_mounted {
		t.Fatalf("%s not mounted after Mount()", fs)
	}
}

func TestChroot(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}
	// create temp dir and tempfile 1
	tempDir := t.TempDir()
	f, err := os.CreateTemp(tempDir, "chroot_test_file")
	if err != nil {
		t.Fatalf("TempFile: %s", err.Error())
	}
	defer f.Close()

	// chroot temp dir
	err = unix.Chroot(tempDir)
	// Chroot requires elevated privilege
	// If test is run without such permission, skip test
	if err == unix.EPERM {
		t.Logf("Denied permission for Chroot. Skipping test (Errno2:  %X)", unix.Errno2())
		return
	} else if err != nil {
		t.Fatalf("Chroot: %s", err.Error())
	}
	// check if tempDir contains test file
	files, err := os.ReadDir("/")
	if err != nil {
		t.Fatalf("ReadDir: %s", err.Error())
	}
	found := false
	for _, file := range files {
		if file.Name() == filepath.Base(f.Name()) {
			found = true
			break
		}
	}
	if !found {
		t.Fatalf("Temp file not found in temp dir")
	}
}

func TestFlock(t *testing.T) {
	if v, r := zosLeVersion(); v <= 2 && r <= 4 {
		t.Skipf("New flock can't be used in %d.%d < 2.5. Run TestLegacyFlock", v, r)
	}

	const (
		SUCCESS = iota
		BLOCKED
	)

	if os.Getenv("TEST_FLOCK_HELPER") == "1" {
		defer os.Exit(0)
		if len(os.Args) != 4 {
			log.Fatal("bad argument")
		}
		mode, err := strconv.Atoi(os.Args[2])
		if err != nil {
			log.Fatalf("%s is invalid: %s", os.Args[2], err)
		}
		filename := os.Args[3]
		f, err := os.OpenFile(filename, os.O_RDWR, 0755)
		if err != nil {
			log.Fatalf("%s", err.Error())
		}
		defer f.Close()

		go func() {
			// timeout
			time.Sleep(5 * time.Second)
			unix.Flock(int(f.Fd()), unix.LOCK_UN)
			fmt.Print(BLOCKED)
			os.Exit(1)
		}()

		err = unix.Flock(int(f.Fd()), mode)
		if err == unix.EWOULDBLOCK {
			fmt.Print(int(unix.EWOULDBLOCK))
			os.Exit(1)
		}
		if err != nil {
			log.Fatal(err)
		}
		defer unix.Flock(int(f.Fd()), unix.LOCK_UN)
		fmt.Print(0)
		return
	}

	f, err := os.Create(filepath.Join(t.TempDir(), "flock_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %s\n", err)
	}
	defer f.Close()
	fd := int(f.Fd())

	testCases := []struct {
		name     string
		fd       int
		p1modes  []int
		p2mode   int
		expected syscall.Errno
	}{
		{"Invalid fd", -1, []int{unix.LOCK_SH}, unix.LOCK_SH, unix.EBADF},
		{"Invalid mode", fd, []int{unix.LOCK_EX | unix.LOCK_SH}, unix.LOCK_SH, unix.EINVAL},
		{"EX-EX", fd, []int{unix.LOCK_SH, unix.LOCK_EX}, unix.LOCK_EX, BLOCKED},
		{"EX-SH", fd, []int{unix.LOCK_SH, unix.LOCK_EX}, unix.LOCK_SH, BLOCKED},
		{"SH-EX", fd, []int{unix.LOCK_EX, unix.LOCK_SH}, unix.LOCK_EX, BLOCKED},
		{"SH-SH", fd, []int{unix.LOCK_EX, unix.LOCK_SH}, unix.LOCK_SH, SUCCESS},
		{"EX-EXNB", fd, []int{unix.LOCK_SH, unix.LOCK_EX}, unix.LOCK_EX | unix.LOCK_NB, unix.EWOULDBLOCK},
		{"EX-SHNB", fd, []int{unix.LOCK_SH, unix.LOCK_EX}, unix.LOCK_SH | unix.LOCK_NB, unix.EWOULDBLOCK},
		{"SH-SHNB", fd, []int{unix.LOCK_EX, unix.LOCK_SH}, unix.LOCK_EX | unix.LOCK_NB, unix.EWOULDBLOCK},
		{"SH-SHNB", fd, []int{unix.LOCK_EX, unix.LOCK_SH}, unix.LOCK_SH | unix.LOCK_NB, SUCCESS},
	}
	// testcase:
	for _, c := range testCases {
		t.Run(c.name, func(t *testing.T) {
			for _, mode := range c.p1modes {
				err = unix.Flock(c.fd, mode)
				if err == c.expected {
					return
				}
				if err != nil {
					t.Fatalf("failed to acquire Flock with mode(%d): %s\n", mode, err)
				}
			}

			p2status := BLOCKED
			done := make(chan bool)
			execP2 := func(isBlock bool) {
				cmd := exec.Command(os.Args[0], "-test.run=^TestFlock$", strconv.Itoa(c.p2mode), f.Name())
				cmd.Env = append(os.Environ(), "TEST_FLOCK_HELPER=1")
				out, _ := cmd.CombinedOutput()
				if p2status, err = strconv.Atoi(string(out)); err != nil {
					log.Fatalf("p2status is not valid: %s\n", err)
				}
				if isBlock {
					done <- true
				}
			}

			if c.expected == BLOCKED {
				go execP2(true)
				<-done
			} else {
				execP2(false)
			}

			if p2status != int(c.expected) {
				unix.Flock(c.fd, unix.LOCK_UN)
				t.Fatalf("expected %d, actual %d\n", c.expected, p2status)
			}
			unix.Flock(c.fd, unix.LOCK_UN)
		})

	}
}

func TestLegacyFlock(t *testing.T) {
	if v, r := zosLeVersion(); v > 2 || (v == 2 && r > 4) {
		t.Skipf("Legacy flock can't be used in %d.%d > 2.4. Run TestFlock", v, r)
	}
	if os.Getenv("GO_WANT_HELPER_PROCESS") == "1" {
		defer os.Exit(0)
		if len(os.Args) != 3 {
			fmt.Printf("bad argument")
			return
		}
		fn := os.Args[2]
		f, err := os.OpenFile(fn, os.O_RDWR, 0755)
		if err != nil {
			fmt.Printf("%s", err.Error())
			return
		}
		err = unix.Flock(int(f.Fd()), unix.LOCK_EX|unix.LOCK_NB)
		// if the lock we are trying should be locked, ignore EAGAIN error
		// otherwise, report all errors
		if err != nil && err != unix.EAGAIN {
			fmt.Printf("%s", err.Error())
		}
	} else {
		// create tempfile 1
		f, err := os.Create(filepath.Join(t.TempDir(), "flock_test_file"))
		if err != nil {
			t.Fatalf("TempFile: %s", err.Error())
		}
		defer f.Close()

		fd := int(f.Fd())

		/* Test Case 1
		 * Try acquiring an occupied lock from another process
		 */
		err = unix.Flock(fd, unix.LOCK_EX)
		if err != nil {
			t.Fatalf("Flock: %s", err.Error())
		}
		cmd := exec.Command(os.Args[0], "-test.run=TestLegacyFlock", f.Name())
		cmd.Env = append(os.Environ(), "GO_WANT_HELPER_PROCESS=1")
		out, err := cmd.CombinedOutput()
		if len(out) > 0 || err != nil {
			t.Fatalf("child process: %q, %v", out, err)
		}
		err = unix.Flock(fd, unix.LOCK_UN)
		if err != nil {
			t.Fatalf("Flock: %s", err.Error())
		}

		/* Test Case 2
		 * Try locking with Flock and FcntlFlock for same file
		 */
		err = unix.Flock(fd, unix.LOCK_EX)
		if err != nil {
			t.Fatalf("Flock: %s", err.Error())
		}
		flock := unix.Flock_t{
			Type:   int16(unix.F_WRLCK),
			Whence: int16(0),
			Start:  int64(0),
			Len:    int64(0),
			Pid:    int32(unix.Getppid()),
		}
		err = unix.FcntlFlock(f.Fd(), unix.F_SETLK, &flock)
		if err != nil {
			t.Fatalf("FcntlFlock: %s", err.Error())
		}
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

	// On some BSDs the actual timeout might also be slightly less than the requested.
	// Add an acceptable margin to avoid flaky tests.
	if took < dur*2/3 {
		t.Errorf("Select: got %v timeout, expected at least %v", took, dur)
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

func TestUnixCredentials(t *testing.T) {
	var ucred syscall.Ucred
	if os.Getuid() != 0 {
		ucred.Pid = int32(os.Getpid())
		ucred.Uid = 0
		ucred.Gid = 0
	}

	ucred.Pid = int32(os.Getpid())
	ucred.Uid = uint32(os.Getuid())
	ucred.Gid = uint32(os.Getgid())
	oob := syscall.UnixCredentials(&ucred)

	// On SOCK_STREAM, this is internally going to send a dummy byte
	scm, err := syscall.ParseSocketControlMessage(oob)
	if err != nil {
		t.Fatalf("ParseSocketControlMessage: %v", err)
	}
	newUcred, err := syscall.ParseUnixCredentials(&scm[0])
	if err != nil {
		t.Fatalf("ParseUnixCredentials: %v", err)
	}
	if *newUcred != ucred {
		t.Fatalf("ParseUnixCredentials = %+v, want %+v", newUcred, ucred)
	}
}

func TestFutimes(t *testing.T) {
	// Create temp dir and file
	f, err := os.Create(filepath.Join(t.TempDir(), "futimes_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %s", err.Error())
	}
	defer f.Close()

	fd := int(f.Fd())

	// Set mod time to newTime
	newTime := time.Date(2001, time.Month(2), 15, 7, 7, 7, 0, time.UTC)
	err = unix.Futimes(
		fd,
		[]unix.Timeval{
			unix.Timeval{newTime.Unix(), 0},
			unix.Timeval{newTime.Unix(), 0},
		})
	if err != nil {
		t.Fatalf("TestFutimes: %v", err)
	}

	// Compare mod time
	stats, err := f.Stat()
	if err != nil {
		t.Fatalf("Stat: %v", err)
	}

	modTime := stats.ModTime()
	if modTime.UTC() != newTime {
		t.Fatalf("TestFutimes: modTime = %v, want %v", modTime.UTC(), newTime)
	}
}

func TestLutimes(t *testing.T) {
	// Create temp dir and file
	tempDir := t.TempDir()
	f, err := os.CreateTemp(tempDir, "lutimes_test_file")
	if err != nil {
		t.Fatalf("TempFile: %s", err.Error())
	}
	defer f.Close()

	symlinkPath := tempDir + "/test_symlink"
	err = os.Symlink(f.Name(), symlinkPath)
	if err != nil {
		t.Fatalf("Symlink: %v", err)
	}

	// Set mod time to newTime
	newTime := time.Date(2001, time.Month(2), 15, 7, 7, 7, 0, time.UTC)
	err = unix.Lutimes(
		symlinkPath,
		[]unix.Timeval{
			unix.Timeval{newTime.Unix(), 0},
			unix.Timeval{newTime.Unix(), 0},
		})
	if err != nil {
		t.Fatalf("TestLutimes: %v", err)
	}

	// Compare mod time
	stats, err := os.Lstat(symlinkPath)
	if err != nil {
		t.Fatalf("Lstat: %v", err)
	}

	modTime := stats.ModTime()
	if modTime.UTC() != newTime {
		t.Fatalf("TestLutimes: modTime = %v, want %v", modTime.UTC(), newTime)
	}

}

func TestDirfd(t *testing.T) {
	// Create temporary directory
	tempDir := t.TempDir()

	f, err := os.CreateTemp(tempDir, "dirfd_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer f.Close()

	// Open temp dir and get stream
	dirStream, err := unix.Opendir(tempDir)
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	defer unix.Closedir(dirStream)

	// Get fd from stream
	dirFd, err := unix.Dirfd(dirStream)
	if err != nil {
		t.Fatalf("Dirfd: %v", err)
	}
	if dirFd < 0 {
		t.Fatalf("Dirfd: fd < 0, (fd = %v)", dirFd)
	}

	oldwd, err := os.Getwd()
	if err != nil {
		t.Fatalf("Getwd: %v", err)
	}
	defer os.Chdir(oldwd)

	// Change dir to fd and get path
	err = unix.Fchdir(dirFd)
	if err != nil {
		t.Fatalf("Fchdir: %v", err)
	}
	path, err := os.Getwd()
	if err != nil {
		t.Fatalf("Getwd: %v", err)
	}

	pathInfo, err := os.Lstat(path)
	if err != nil {
		t.Fatalf("os.Stat: %v", err)
	}
	tempDirInfo2, err := os.Lstat(tempDir)
	if err != nil {
		t.Fatalf("os.Stat: %v", err)
	}
	// Perform Test
	if !os.SameFile(pathInfo, tempDirInfo2) {
		t.Fatalf("Dirfd: expected working directory to be %v, actually: %v", tempDir, path)
	}
}

func TestEpollCreate(t *testing.T) {
	if BypassTestOnUntil("zoscan59", "2024-04-01T12:45:21.123Z") {
		t.Skip("skipping on zoscan59 until 2024-04-01")
	}
	efd, err := unix.EpollCreate(1)
	if err != nil {
		t.Fatalf("EpollCreate: %v", err)
	}
	defer unix.Close(efd)
}

func TestEpollCreate1(t *testing.T) {
	if BypassTestOnUntil("zoscan59", "2024-04-01T12:45:21.123Z") {
		t.Skip("skipping on zoscan59 until 2024-04-01")
	}
	efd, err := unix.EpollCreate1(0)
	if err != nil {
		t.Fatalf("EpollCreate1: %v", err)
	}
	unix.Close(efd)
}

func TestEpoll(t *testing.T) {
	if BypassTestOnUntil("zoscan59", "2024-04-01T12:45:21.123Z") {
		t.Skip("skipping on zoscan59 until 2024-04-01")
	}
	efd, err := unix.EpollCreate1(0) // no CLOEXEC equivalent on z/OS
	if err != nil {
		t.Fatalf("EpollCreate1: %v", err)
	}
	// no need to defer a close on efd, as it's not a real file descriptor on zos

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

	if events[0].Events&unix.EPOLLIN == 0 {
		t.Errorf("Expected EPOLLIN flag to be set, got %b", events[0].Events)
	}

	x := 0
	n, err = unix.EpollPwait(efd, events, 1, &x)
	if err != nil {
		t.Fatalf("EpollPwait: %v", err)
	}
}

func TestEventfd(t *testing.T) {
	fd, err := unix.Eventfd(0, 0)
	if err != nil {
		t.Fatalf("Eventfd: %v", err)
	}
	if fd <= 2 {
		t.Fatalf("Eventfd: fd <= 2, got: %d", fd)
	}
}

func TestEventfdSemaphore(t *testing.T) {
	efd, err := unix.Eventfd(1, unix.EFD_SEMAPHORE|unix.EFD_NONBLOCK|unix.EFD_CLOEXEC)
	if err != nil {
		t.Fatalf("Eventfd: %v", err)
	}

	writeBytes := make([]byte, 8)
	writeBytes[7] = 0x4
	n, err := unix.Write(efd, writeBytes)
	if err != nil {
		t.Fatalf("Write: %v", err)
	}
	if n != 8 {
		t.Fatalf("Write: only wrote %d bytes, wanted 8", n)
	}

	for i := 0; i < 5; i++ {
		readBytes := make([]byte, 8)
		n, err := unix.Read(efd, readBytes)
		if err != nil {
			t.Fatalf("Read: %v", err)
		}
		if n != 8 {
			t.Fatalf("Read: only read %d bytes, wanted 8", n)
		}
	}
	readBytes := make([]byte, 8)
	n, err = unix.Read(efd, readBytes)
	if err == nil || err.Error() != "EDC5112I Resource temporarily unavailable." {
		t.Fatalf("Read: expected error \"EDC5112I Resource temporarily unavailable.\", got %v", err)
	}
	if n != -1 {
		t.Fatalf("Read: expected error code -1, got %d", n)
	}
	if readBytes[7] != 0 {
		t.Fatalf("Read: expected return of 0, got %d", readBytes[7])
	}
}

func TestStatfs(t *testing.T) {
	// Create temporary directory
	tempDir := t.TempDir()

	var stat unix.Statfs_t
	if err := unix.Statfs(tempDir, &stat); err != nil {
		t.Fatalf("Stafs: %v", err)
	}

	if stat.Files == 0 {
		t.Fatalf("Statfs: expected files > 0")
	}
}

func TestStatfsProc(t *testing.T) {
	// Create temporary directory

	if _, err := os.Stat("/proc/self"); errors.Is(err, os.ErrNotExist) {
		t.Skip("/proc/self is not exist skipping the test")
	}

	var stat unix.Statfs_t
	if err := unix.Statfs("/proc/self/ns", &stat); err != nil {
		t.Fatalf("Stafs: %v", err)
	}

	if stat.Type != unix.PROC_SUPER_MAGIC {
		t.Fatalf("Statfs: expected files > 0")
	}
}

func TestFstatfs(t *testing.T) {
	// Create temporary directory
	file, err := os.Create(filepath.Join(t.TempDir(), "fstatfs_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer file.Close()

	fd := int(file.Fd())

	var stat unix.Statfs_t
	if err = unix.Fstatfs(fd, &stat); err != nil {
		t.Fatalf("Stafs: %v", err)
	}

	if stat.Files == 0 {
		t.Fatalf("Statfs: expected files > 0")
	}
}

func TestFdatasync(t *testing.T) {
	t.Skip("FAIL: Known failure, would hang if not skipped")
	// Create temporary directory
	file, err := os.Create(filepath.Join(t.TempDir(), "fdatasync_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	file.Close()

	fd := int(file.Fd())

	var stat1 unix.Stat_t
	if err = unix.Fstat(fd, &stat1); err != nil {
		t.Fatalf("Fstat: %v", err)
	}

	time.Sleep(1 * time.Second)
	if _, err := unix.Write(fd, []byte("Test string")); err != nil {
		t.Fatalf("Write: %v", err)
	}

	var stat2 unix.Stat_t
	if err = unix.Fstat(fd, &stat2); err != nil {
		t.Fatalf("Fstat: %v", err)
	}

	time.Sleep(1 * time.Second)
	if err = unix.Fdatasync(fd); err != nil {
		t.Fatalf("Fdatasync: %v", err)
	}

	var stat3 unix.Stat_t
	if err = unix.Fstat(fd, &stat3); err != nil {
		t.Fatalf("Fstat: %v", err)
	}

	if stat2.Mtim != stat3.Mtim {
		t.Fatalf("Fdatasync: Modify times do not match. Wanted %v, got %v", stat2.Mtim, stat3.Mtim)
	}
}

func TestReadDirent(t *testing.T) {
	// Create temporary directory and files
	tempDir := t.TempDir()

	f1, err := os.CreateTemp(tempDir, "ReadDirent_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer f1.Close()
	f2, err := os.CreateTemp(tempDir, "ReadDirent_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer f2.Close()

	tempSubDir, err := os.MkdirTemp(tempDir, "ReadDirent_SubDir")
	if err != nil {
		t.Fatalf("TempDir: %v", err)
	}
	f3, err := os.CreateTemp(tempSubDir, "ReadDirent_subDir_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer f3.Close()

	// Get fd of tempDir
	dir, err := os.Open(tempDir)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer dir.Close()

	fd := int(dir.Fd())

	// Run Getdirentries
	buf := make([]byte, 2048)
	n, err := unix.ReadDirent(fd, buf)
	if err != nil {
		t.Fatalf("ReadDirent: %v", err)
	}
	if n == 0 {
		t.Fatalf("ReadDirent: 0 bytes read")
	}

	names := make([]string, 0)
	consumed, count, _ := unix.ParseDirent(buf, 100, names)
	if consumed == 0 {
		t.Fatalf("ParseDirent: consumed 0 bytes")
	}
	if count != 3 {
		t.Fatalf("ParseDirent: only recorded %d entries, expected 3", count)
	}
}

func TestPipe2(t *testing.T) {
	var p [2]int
	err := unix.Pipe2(p[:], unix.O_CLOEXEC)
	if err != nil {
		t.Fatalf("Pipe2: %v", err)
	}

	r, w := int(p[0]), int(p[1])

	n1, err := unix.Write(w, []byte("Testing pipe2!"))
	if err != nil {
		t.Fatalf("Write: %v", err)
	}

	buf := make([]byte, 256)
	n2, err := unix.Read(r, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}

	if n1 != n2 {
		t.Fatalf("Pipe2: bytes read != bytes written. Wrote %d, read %d", n1, n2)
	}
}

func TestInotify(t *testing.T) {
	// Driver func to try and capture all possible events
	t.Run("IN_ACCESS", inotify_access)
	t.Run("IN_ATTRIB", inotify_attrib)
	t.Run("IN_CLOSE_WRITE", inotify_close_write)
	t.Run("IN_CLOSE_NOWRITE", inotify_close_nowrite)
	t.Run("IN_CREATE", inotify_create)
	t.Run("IN_DELETE", inotify_delete)
	t.Run("IN_DELETE_SELF", inotify_delete_self)
	t.Run("IN_MODIFY", inotify_modify)
	t.Run("IN_MOVE_SELF", inotify_move_self)
	t.Run("IN_MOVED_FROM", inotify_moved_from)
	t.Run("IN_MOVED_TO", inotify_moved_to)
	t.Run("IN_OPEN", inotify_open)
}

func inotify_access(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.Create(filepath.Join(tempDir, "inotify_access_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempFile.Name(), unix.IN_ACCESS)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	n, err := tempFile.Write([]byte("Writing before reading"))
	if err != nil {
		t.Fatalf("Write: %v", err)
	}
	if n <= 0 {
		t.Fatalf("Did not write any data")
	}
	tempFile.Seek(0, 0)

	buf := make([]byte, 64)
	n, err = tempFile.Read(buf)
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n <= 0 {
		t.Fatalf("Did not read any data")
	}

	// Expect event
	buf = make([]byte, unix.SizeofInotifyEvent)
	n, err = unix.Read(infd, buf[:])
	if n == -1 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_attrib(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.Create(filepath.Join(tempDir, "inotify_attrib_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}
	defer unix.Close(infd)

	wd, err := unix.InotifyAddWatch(infd, tempFile.Name(), unix.IN_ATTRIB)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	if err = tempFile.Chmod(0777); err != nil {
		t.Fatalf("Chmod: %v", err)
	}

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent)
	n, err := unix.Read(infd, buf[:])
	if n == -1 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_close_write(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.Create(filepath.Join(tempDir, "inotify_close_write_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	// File closed in test later

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempFile.Name(), unix.IN_CLOSE_WRITE)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	_, err = tempFile.Write([]byte("Writing before closing"))
	if err != nil {
		t.Fatalf("Write: %v", err)
	}
	tempFile.Close()

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent)
	n, err := unix.Read(infd, buf[:])
	if n == -1 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_close_nowrite(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir, "inotify_close_nowrite_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	// File closed later in test

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempDir, unix.IN_CLOSE_NOWRITE)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	d, err := os.Open(tempDir)
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	tempFile.Close()
	d.Close()

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent*4)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}

	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_create(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempDir, unix.IN_CREATE)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	f, err := os.CreateTemp(tempDir, "inotify_create_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	f.Close()

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent*4)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_delete(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir, "inotify_delete_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	// File closed later in test

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempDir, unix.IN_DELETE)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	name := tempFile.Name()
	tempFile.Close()
	if err = os.Remove(name); err != nil {
		t.Fatalf("Remove: %v", err)
	}

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent*4)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_delete_self(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir, "inotify_delete_self_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	// File closed later in test

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	_, err = unix.InotifyAddWatch(infd, tempDir, unix.IN_DELETE_SELF)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	tempFile.Close()
	if err = os.RemoveAll(tempDir); err != nil {
		t.Fatalf("RemoveAll: %v", err)
	}

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}
}

func inotify_modify(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir, "inotify_modify_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempFile.Name(), unix.IN_MODIFY)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	_, err = tempFile.Write([]byte("Writing before closing"))
	if err != nil {
		t.Fatalf("Write: %v", err)
	}

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_move_self(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir, "inotify_move_self_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempFile.Name(), unix.IN_MOVE_SELF)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	err = os.Rename(tempFile.Name(), tempFile.Name()+"2")
	if err != nil {
		t.Fatalf("Rename: %v", err)
	}

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_moved_from(t *testing.T) {
	// Create temporary files
	tempDir1 := t.TempDir()
	tempDir2 := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir1, "inotify_moved_from_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempDir1, unix.IN_MOVED_FROM)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	filename := strings.TrimPrefix(tempFile.Name(), tempDir1)
	err = os.Rename(tempDir1+filename,
		tempDir2+filename)
	if err != nil {
		t.Fatalf("Rename: %v", err)
	}

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent*4)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}

	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_moved_to(t *testing.T) {
	// Create temporary files
	tempDir1 := t.TempDir()
	tempDir2 := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir1, "inotify_moved_to_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempDir2, unix.IN_MOVED_TO)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	filename := strings.TrimPrefix(tempFile.Name(), tempDir1)
	err = os.Rename(tempDir1+filename,
		tempDir2+filename)
	if err != nil {
		t.Fatalf("Rename: %v", err)
	}

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent*4)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}

	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func inotify_open(t *testing.T) {
	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.CreateTemp(tempDir, "inotify_open_test_file")
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	// File closed later in test

	// Setup iNotify
	infd, err := unix.InotifyInit1(unix.IN_CLOEXEC | unix.IN_NONBLOCK)
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempFile.Name(), unix.IN_OPEN)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	name := tempFile.Name()
	tempFile.Close()
	f, err := os.Open(name)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer f.Close()

	// Expect event
	buf := make([]byte, unix.SizeofInotifyEvent)
	n, err := unix.Read(infd, buf[:])
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n == 0 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func TestSockNonblock(t *testing.T) {
	ch1 := make(chan int)
	go func() {
		select {
		case <-ch1:
		}

		client, err := unix.Socket(unix.AF_INET, unix.SOCK_STREAM|unix.SOCK_CLOEXEC, 0)
		if err != nil {
			t.Fatalf("Socket: %v", err)
		}
		defer unix.Close(client)

		clientSA := unix.SockaddrInet4{Port: 33333, Addr: [4]byte{127, 0, 0, 1}}
		err = unix.Connect(client, &clientSA)
		if err != nil {
			t.Fatalf("Connect: %v", err)
		}

		select {
		case <-ch1:
		}
	}()

	server, err := unix.Socket(unix.AF_INET, unix.SOCK_STREAM|unix.SOCK_CLOEXEC, 0)
	if err != nil {
		t.Fatalf("Socket: %v", err)
	}
	defer unix.Close(server)

	serverSA := unix.SockaddrInet4{Port: 33333, Addr: [4]byte{}}

	err = unix.SetsockoptInt(server, unix.SOL_SOCKET, unix.SO_REUSEADDR, 1)
	if err != nil {
		t.Fatalf("SetsockoptInt: %v", err)
	}

	err = unix.Bind(server, &serverSA)
	if err != nil {
		t.Fatalf("Bind: %v", err)
	}

	err = unix.Listen(server, 3)
	if err != nil {
		t.Fatalf("Listen: %v", err)
	}

	ch1 <- 1

	accept, _, err := unix.Accept4(server, unix.SOCK_NONBLOCK|unix.SOCK_CLOEXEC)
	if err != nil {
		t.Fatalf("Accept: %v", err)
	}

	buf := make([]byte, 16)
	_, err = unix.Read(accept, buf)
	if err.Error() != "EDC8102I Operation would block." {
		t.Fatalf("Read: Expected error \"EDC8102I Operation would block.\", but got \"%v\"", err)
	}

	ch1 <- 1
}

func TestSockIPTTL(t *testing.T) {
	server, err := unix.Socket(unix.AF_INET, unix.SOCK_STREAM|unix.SOCK_CLOEXEC, 0)
	if err != nil {
		t.Fatalf("Socket: %v", err)
	}

	ttl, err := unix.GetsockoptInt(server, unix.IPPROTO_IP, unix.IP_TTL)
	if err != nil {
		t.Fatalf("GetsockoptInt: %v", err)
	}

	if ttl != 64 {
		t.Fatalf("Expected TTL value of 64, got %v", ttl)
	}

	err = unix.SetsockoptInt(server, unix.IPPROTO_IP, unix.IP_TTL, 65)
	if err != nil {
		t.Fatalf("SetsockoptInt: %v", err)
	}

	ttl, err = unix.GetsockoptInt(server, unix.IPPROTO_IP, unix.IP_TTL)
	if err != nil {
		t.Fatalf("GetsockoptInt: %v", err)
	}

	if ttl != 65 {
		t.Fatalf("Expected TTL value of 65, got %v", ttl)
	}
}

func TestSethostname(t *testing.T) {
	name, err := os.Hostname()
	if err != nil {
		t.Fatalf("Failed to get hostname: %v", err)
	}

	err = unix.Sethostname([]byte(name))
	if !strings.Contains(err.Error(), unix.ENOTSUP.Error()) {
		t.Fatalf("Sethostname: Expected error \"EDC5247I Operation not supported.\", but got \"%v\"", err)
	}
}

func TestGetrandom(t *testing.T) {
	buf := make([]byte, 16)
	n, err := unix.Getrandom(buf, unix.GRND_NONBLOCK|unix.GRND_RANDOM)
	if err != nil {
		t.Fatalf("Getrandom: %v", err)
	}
	if n != 16 {
		t.Fatalf("Expected to read %d bytes. Actually read %d", 16, n)
	}

	sum := 0
	for _, v := range buf {
		sum += int(v)
	}
	if sum == 0 {
		t.Fatalf("Getrandom: no random values retrieved")
	}
}

func TestTermios(t *testing.T) {
	const ioctlReadTermios = unix.TCGETS
	const ioctlWriteTermios = unix.TCSETS

	// Get address of controlling terminal
	tty, err := unix.Ctermid()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		t.Skip("No terminal")
	}

	// Open controlling terminal
	f, err := unix.Open(tty, 3, 0755)
	if err != nil {
		t.Skipf("Skipping because opening /dev/tty failed: %v", err)
	}
	defer unix.Close(f)

	// Test IoctlGetTermios
	termios, err := unix.IoctlGetTermios(f, ioctlReadTermios)

	// Save old terminal settings to restore
	oldTermios := *termios
	if err != nil {
		t.Fatalf("IoctlGetTermios: %v", err)
	}

	// This attempts to replicate the behaviour documented for cfmakeraw in
	// the termios(3) manpage.
	termios.Iflag &^= unix.IGNBRK | unix.BRKINT | unix.PARMRK | unix.ISTRIP | unix.INLCR | unix.IGNCR | unix.ICRNL | unix.IXON
	termios.Oflag &^= unix.OPOST
	termios.Lflag &^= unix.ECHO | unix.ECHONL | unix.ICANON | unix.ISIG | unix.IEXTEN
	termios.Cflag &^= unix.CSIZE | unix.PARENB
	termios.Cflag |= unix.CS8
	termios.Cc[unix.VMIN] = 1
	termios.Cc[unix.VTIME] = 0

	// Test IoctlSetTermios
	if err := unix.IoctlSetTermios(f, ioctlWriteTermios, termios); err != nil {
		t.Fatalf("IoctlSetTermios: %v", err)
	}

	// Restore
	if err := unix.IoctlSetTermios(f, ioctlWriteTermios, &oldTermios); err != nil {
		t.Fatalf("IoctlSetTermios: %v", err)
	}
}

func TestDup3(t *testing.T) {
	data := []byte("Test String")

	// Create temporary files
	tempDir := t.TempDir()
	tempFile, err := os.Create(filepath.Join(tempDir, "dup3_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Duplicate fd
	fd1, err := unix.Open(tempFile.Name(), unix.O_RDWR|unix.O_CLOEXEC, 777)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer unix.Close(fd1)

	fd2 := 11
	if err := unix.Dup3(fd1, fd2, unix.O_CLOEXEC); err != nil {
		t.Fatalf("Dup3: %v", err)
	}
	defer unix.Close(fd2)

	// Write
	n, err := unix.Write(fd2, data)
	if err != nil {
		t.Fatalf("Write: %v", err)
	}
	if n != len(data) {
		t.Fatalf("Write: only wrote %d bytes, expected %d", n, len(data))
	}

	// Read
	buf := make([]byte, 16)
	n, err = tempFile.Read(buf)
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n != len(data) {
		t.Fatalf("Read: only read %d bytes, expected %d", n, len(data))
	}

	// Compare
	for i := 0; i < len(data); i++ {
		if buf[i] != data[i] {
			t.Fatalf("Dup3: data read did not match data written")
		}
	}
}

func TestWait4(t *testing.T) {
	if v, r := zosLeVersion(); v <= 2 && r <= 4 {
		t.Skipf("New wait4 can't be used in %d.%d < 2.5", v, r)
	}
	if os.Getenv("TEST_WAIT4_HELPER") == "1" {
		exitCode, err := strconv.Atoi(os.Args[2])
		if err != nil {
			fmt.Printf("exit code cannot be parsed: %s\n", err)
		}
		defer os.Exit(exitCode)
		for i := 0; i < 50000000; i++ {
		}
		return
	}

	const (
		childPid = -2
		core     = 0x80
		exited   = 0x00
		stopped  = 0x7F
		shift    = 8
	)
	pgid, err := unix.Getpgid(os.Getpid())
	if err != nil {
		t.Fatal(err)
	}
	testCases := []struct {
		name     string
		exitCode int
		pid      int
		options  int
		signals  []syscall.Signal
		wpid     int
		err      error
		ws       unix.WaitStatus
	}{
		{"Child's pgid", 0, -pgid, 0, []syscall.Signal{}, childPid, nil, exited},
		{"Any", 0, -1, 0, []syscall.Signal{}, childPid, nil, exited},
		{"Pid zero", 0, 0, 0, []syscall.Signal{}, childPid, nil, exited},
		{"Child's pid", 0, childPid, 0, []syscall.Signal{}, childPid, nil, exited},
		{"Exited with 2", 2, childPid, 0, []syscall.Signal{}, childPid, nil, unix.WaitStatus((2 << shift) | exited)},
		{"No hang", 0, childPid, unix.WNOHANG, []syscall.Signal{}, 0, nil, exited},
		{"No child", 0, os.Getpid(), 0, []syscall.Signal{}, -1, unix.ECHILD, exited},
		{"Inval", 0, -1, -1, []syscall.Signal{}, -1, unix.EINVAL, exited},
		{"Killed", 0, childPid, 0, []syscall.Signal{unix.SIGKILL}, childPid, nil, unix.WaitStatus(unix.SIGKILL)},
		{"Interrupted", 0, childPid, 0, []syscall.Signal{unix.SIGINT}, childPid, nil, unix.WaitStatus(unix.SIGINT)},
		{"Stopped", 0, childPid, unix.WUNTRACED, []syscall.Signal{unix.SIGSTOP}, childPid, nil, unix.WaitStatus((unix.SIGSTOP << shift) | stopped)},
		{"Core dump", 0, childPid, unix.WUNTRACED, []syscall.Signal{unix.SIGTRAP}, childPid, nil, unix.WaitStatus(core | unix.SIGTRAP)},
		// TODO(paulc): Skipping these two until wait4 behaves the same as in Linux
		// {"Continued", 0, cpid, unix.WCONTINUED, []syscall.Signal{unix.SIGSTOP, unix.SIGCONT}, cpid, nil, 0xffff},
		// {"Intmin", 0, -2147483648, 0, []syscall.Signal{}, -1, unix.ESRCH, exited},
	}

	for _, c := range testCases {
		t.Run(c.name, func(t *testing.T) {
			cmd := exec.Command(os.Args[0], "-test.run=^TestWait4$", fmt.Sprint(c.exitCode))
			cmd.Env = []string{"TEST_WAIT4_HELPER=1"}
			if err := cmd.Start(); err != nil {
				t.Fatal(err)
			}

			for i, sig := range c.signals {
				if err := cmd.Process.Signal(sig); err != nil {
					cmd.Process.Kill()
					t.Fatal(err)
				}
				if i != len(c.signals)-1 {
					time.Sleep(1000 * time.Millisecond)
				}
			}

			pid, wpid := c.pid, c.wpid
			if c.pid == childPid {
				pid = cmd.Process.Pid
			}
			if c.wpid == childPid {
				wpid = cmd.Process.Pid
			}
			ws := unix.WaitStatus(0)
			ru := unix.Rusage{}
			ret, err := unix.Wait4(pid, &ws, c.options, &ru)
			if err != c.err {
				t.Fatalf("expected %s error but got %s error\n", c.err, err)
			}
			if ret != wpid {
				t.Fatalf("expected return value of %d but got %d\n", wpid, ret)
			}
			if ws != c.ws {
				t.Fatalf("expected wait status %x but got %x\n", c.ws, ws)
			}
			if err == nil && len(c.signals) == 0 && c.options&unix.WNOHANG != unix.WNOHANG {
				if emptyRU := new(unix.Rusage); ru == *emptyRU {
					t.Fatalf("expected non-empty rusage but got %+v", ru)
				}
			}
			cmd.Process.Kill()
		})
	}
}

func TestNanosleep(t *testing.T) {
	waitTime := int64(10000000)

	var ts, tsLeftover unix.Timespec
	ts = unix.Timespec{
		Sec:  0,
		Nsec: waitTime,
	}
	tsLeftover = unix.Timespec{
		Sec:  0,
		Nsec: 0,
	}

	t1 := time.Now().UnixNano()
	if err := unix.Nanosleep(&ts, &tsLeftover); err != nil {
		t.Fatalf("Nanosleep: %v", err)
	}
	t2 := time.Now().UnixNano()

	if t2-t1 < waitTime {
		t.Fatalf("Nanosleep: did not wait long enough. Expected: %d, got: %d", waitTime, t2-t1)
	}
}

func TestOpenat2(t *testing.T) {
	how := &unix.OpenHow{
		Flags: unix.O_RDONLY,
	}
	fd, err := unix.Openat2(unix.AT_FDCWD, ".", how)
	if err != nil {
		t.Fatalf("openat2: %v", err)
	}
	if err := unix.Close(fd); err != nil {
		t.Fatalf("close: %v", err)
	}

	// prepare
	tempDir := t.TempDir()

	subdir := filepath.Join(tempDir, "dir")
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

func TestUtimesNanoAt(t *testing.T) {
	// z/OS currently does not support setting milli/micro/nanoseconds for files
	// The Nsec field will be 0 when trying to get atime/mtime

	// Create temp dir and file
	f, err := os.Create(filepath.Join(t.TempDir(), "utimesNanoAt_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer f.Close()

	fd := int(f.Fd())

	// Set atime and mtime
	ts := []unix.Timespec{
		unix.Timespec{123456789, 123456789},
		unix.Timespec{123456789, 123456789},
	}
	atimeTS := time.Unix(ts[0].Sec, ts[0].Nsec)
	mtimeTS := time.Unix(ts[1].Sec, ts[1].Nsec)

	err = unix.UtimesNanoAt(fd, f.Name(), ts, 0)
	if err != nil {
		t.Fatalf("TestUtimesNanoAt: %v", err)
	}

	// Compare atime and mtime
	var statAfter unix.Stat_t
	if err = unix.Fstat(fd, &statAfter); err != nil {
		t.Fatalf("Fstat: %v", err)
	}
	atimeAfter := time.Unix(statAfter.Atim.Sec, statAfter.Atim.Nsec)
	mtimeAfter := time.Unix(statAfter.Mtim.Sec, statAfter.Mtim.Nsec)

	// TODO (joon): check using time.Equal() once z/OS supports finer timestamps for files
	if atimeAfter.Unix() != atimeTS.Unix() {
		t.Fatalf("Expected atime to be %v. Got %v", atimeAfter.Unix(), atimeTS.Unix())
	}
	if mtimeAfter.Unix() != mtimeTS.Unix() {
		t.Fatalf("Expected mtime to be %v. Got %v", atimeAfter.Unix(), atimeTS.Unix())
	}
}

func TestPivotRoot(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}

	err := unix.Unshare(unix.CLONE_NEWNS)
	if err != nil {
		t.Fatalf("Unshare: %v", err)
	}

	// Create our 'new_root' and bind mount it to satisfy one of the conditions of pivot_root
	newRoot := t.TempDir()

	err = unix.Mount(newRoot, newRoot, "", unix.MS_BIND|unix.MS_REC, "")
	if err != nil {
		t.Fatalf("Mount: %v", err)
	}

	// Create our 'old_root'
	oldRoot, err := os.MkdirTemp(newRoot, "oldRoot")
	if err != nil {
		t.Fatalf("TempDir: %v", err)
	}

	// Perform the pivot and check that our old root is now a subfolder in our new root
	if err = unix.PivotRoot(newRoot, oldRoot); err != nil {
		t.Fatalf("PivotRoot: %v", err)
	}

	err = unix.Chdir("/")
	if err != nil {
		t.Fatalf("Chdir: %v", err)
	}

	if _, err := os.Stat("/" + filepath.Base(oldRoot)); os.IsNotExist(err) {
		t.Fatalf("Expected to see old root as a subdirectory.")
	}
}

func TestMountUnmount(t *testing.T) {
	if v, r := zosLeVersion(); v < 2 || (v == 2 && r < 4) {
		t.Skipf("New mount can't be used in %d.%d < 2.5. Run TestLegacyMountUnmount", v, r)
	}

	if _, err := os.Stat("/proc/self"); errors.Is(err, os.ErrNotExist) {
		t.Skip("/proc/self is not exist skipping the test")
	}

	// Check that TFS is installed on the system, otherwise the test cannot be performed.
	b, err := os.ReadFile("/proc/filesystems")
	if err != nil {
		t.Fatalf("ReadFile: %v", err)
	}
	filesystems := string(b)
	if !strings.Contains(filesystems, "TFS") {
		t.Skip("Missing TFS filesystem")
	}

	// Create a temp directory for the TFS
	tempSrc := t.TempDir()

	// Mount the TFS
	tfs := "testTFS"
	err = exec.Command("/usr/sbin/mount", "-t", "TFS", "-f", tfs, tempSrc).Run()
	if err != nil {
		t.Skip("Could not create TFS")
	}

	// Create a temp dir and test Mount()
	tempTgt := t.TempDir()

	err = unix.Mount(tempSrc, tempTgt, "TFS", 0, "")
	if err != nil {
		t.Fatalf("Mount: %v", err)
	}

	// Unmount and cleanup
	err = unix.Unmount(tempTgt, 0)
	if err != nil {
		t.Fatalf("Unmount: %v", err)
	}
	err = exec.Command("/usr/sbin/unmount", "-f", tfs).Run()
	if err != nil {
		t.Fatalf("Could not remove TFS")
	}
}

func TestMountNamespace(t *testing.T) {
	if v, r := zosLeVersion(); v <= 2 && r <= 4 {
		t.Skipf("Namespaces not available on z/OS %v.%v", v, r)
	}

	// Check that TFS is installed on the system, otherwise the test cannot be performed.
	b, err := os.ReadFile("/proc/filesystems")
	if err != nil {
		t.Skipf("Problem with reading /proc/filesystems: %v", err)
	}
	filesystems := string(b)
	if !strings.Contains(filesystems, "TFS") {
		t.Skipf("Missing TFS filesystem")
	}

	if os.Getenv("SETNS_HELPER_PROCESS") == "1" {
		err := unix.Unshare(unix.CLONE_NEWNS)
		if err != nil {
			t.Skipf("Unshare: %v", err)
		}

		// Create a temp directory for the TFS
		tempSrc := t.TempDir()

		// Mount the TFS
		err = exec.Command("/usr/sbin/mount", "-t", "TFS", "-f", "testTFS", tempSrc).Run()
		if err != nil {
			t.Skipf("Could not create TFS")
		}

		data, err := os.ReadFile("/proc/mounts")
		if err != nil {
			t.Fatalf("ReadFile: %v", err)
		}

		err = os.WriteFile(os.Getenv("MNT_NS_FILE"), data, 644)
		if err != nil {
			t.Fatalf("WriteFile: %v", err)
		}
		return
	}

	// Make a file to copy the child process' mount information
	f, err := os.CreateTemp("", "mntNsTestFile")
	if err != nil {
		t.Fatalf("Could not create temp file")
	}
	defer os.Remove(f.Name())
	f.Close()

	cmd := exec.Command(os.Args[0], "-test.v", "-test.run=^TestMountNamespace$")
	cmd.Env = append(os.Environ(), "SETNS_HELPER_PROCESS=1")
	cmd.Env = append(cmd.Env, "MNT_NS_FILE="+f.Name())

	// Create the child process and get the path of the TFS mount to be cleaned up
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("helper process failed: %v\n%v", err, string(out))
	}
	if strings.Contains(string(out), "SKIP") {
		t.Skipf("helper process: %v", string(out))
	}
	tfsDir := strings.Split(string(out), "\n")[1]
	defer os.RemoveAll(tfsDir)

	d1, err := os.ReadFile(f.Name())
	if err != nil {
		t.Fatalf("ReadFile: %v", err)
	}

	d2, err := os.ReadFile("/proc/mounts")
	if err != nil {
		t.Fatalf("ReadFile: %v", err)
	}

	// Check that the TFS created in the child process was not made in the parent
	if !strings.Contains(string(d1), tfsDir) {
		t.Errorf("Expected to see %v in child process' /proc/mounts", tfsDir)
	}

	if strings.Contains(string(d2), tfsDir) {
		t.Errorf("Expected not to see %v in parent process' /proc/mounts", tfsDir)
	}
}
func TestMkfifoat(t *testing.T) {
	name := fmt.Sprintf("fifo%d", os.Getpid())
	pname := fmt.Sprintf("/tmp/fifo%d", os.Getpid())
	dirStream, err := unix.Opendir("/tmp")
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	defer unix.Closedir(dirStream)

	dirFd, err := unix.Dirfd(dirStream)
	if err != nil {
		t.Fatalf("Dirfd: %v", err)
	}
	if dirFd < 0 {
		t.Fatalf("Dirfd: fd < 0, (fd = %v)", dirFd)
	}
	err = unix.Mkfifoat(dirFd, name, 0666)
	if err != nil {
		t.Fatalf("Mkfifoat: failed to create FIFO: %v at %d", err, dirFd)
	}
	st, err := os.Stat(pname)
	if err != nil {
		t.Fatalf("Mkfifoat: failed to stat FIFO: %s %v", pname, err)
	}
	if st.Mode()&os.ModeNamedPipe != os.ModeNamedPipe {
		t.Fatalf("Mkfifoat: %s is not a FIFO", pname)
	}

	os.Remove(pname)
}

func TestMkdirat(t *testing.T) {
	name := fmt.Sprintf("dir%d", os.Getpid())
	pname := fmt.Sprintf("/tmp/dir%d", os.Getpid())
	dirStream, err := unix.Opendir("/tmp")
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	defer unix.Closedir(dirStream)

	dirFd, err := unix.Dirfd(dirStream)
	if err != nil {
		t.Fatalf("Dirfd: %v", err)
	}
	if dirFd < 0 {
		t.Fatalf("Dirfd: fd < 0, (fd = %v)", dirFd)
	}
	err = unix.Mkdirat(dirFd, name, 0777)
	if err != nil {
		t.Fatalf("Mkdirat: failed to create directory: %v at %d", err, dirFd)
	}
	st, err := os.Stat(pname)
	if err != nil {
		t.Fatalf("Mkdirat: failed to stat directory: %s %v", pname, err)
	}
	if !st.Mode().IsDir() {
		t.Fatalf("Mkdirat: %s is not a directory", pname)
	}
	os.Remove(pname)
}

func TestLinkat(t *testing.T) {
	lName := fmt.Sprintf("testLinkatLink%d", os.Getpid())

	dirStream, err := unix.Opendir("/tmp")
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	defer unix.Closedir(dirStream)

	dirFd, err := unix.Dirfd(dirStream)
	if err != nil {
		t.Fatalf("Dirfd: %v", err)
	}
	if dirFd < 0 {
		t.Fatalf("Dirfd: fd < 0, (fd = %v)", dirFd)
	}

	f, err := os.CreateTemp("/tmp", "tesLinkatFile")
	if err != nil {
		t.Fatalf("CreateTemp: %v", err)
	}
	defer os.Remove(f.Name())
	defer f.Close()

	err = unix.Linkat(dirFd, f.Name(), dirFd, lName, 0)
	if err != nil {
		t.Fatalf("Linkat: Failed to create link: %v", err)
	}
	defer os.Remove("/tmp/" + lName)

	fInfo, err := os.Lstat(f.Name())
	if err != nil {
		t.Fatalf("Lstat: %v", err)
	}
	lInfo, err := os.Lstat(filepath.Join("/tmp/", lName))
	if err != nil {
		t.Fatalf("Lstat: %v", err)
	}

	if !os.SameFile(fInfo, lInfo) {
		t.Errorf("Expected FileInfo for %s to match %s", lName, f.Name())
	}
}

func TestSymlinkat(t *testing.T) {
	f, err := os.Create(filepath.Join(t.TempDir(), "symlinkatTestFile"))
	if err != nil {
		t.Fatal("CreateTemp:", err)
	}
	f.Close()

	dir, err := os.Open(filepath.Dir(f.Name()))
	if err != nil {
		t.Fatal("Open:", err)
	}
	defer dir.Close()

	linkName := fmt.Sprintf("testSymlink%d", os.Getpid())
	err = unix.Symlinkat(f.Name(), int(dir.Fd()), linkName)
	if err != nil {
		t.Fatal("Symlinkat:", err)
	}

	buf := make([]byte, 256)
	_, err = unix.Readlinkat(int(dir.Fd()), linkName, buf)
	if err != nil {
		t.Fatal("Readlink:", err)
	}

	if string(f.Name()) != string(buf[:len(f.Name())]) {
		t.Errorf("Expected buffer contents to be: %s. Got: %s.", f.Name(), string(buf[:]))
	}
}

func TestMknodat(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}

	dirStream, err := unix.Opendir("/tmp")
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	defer unix.Closedir(dirStream)

	dirFd, err := unix.Dirfd(dirStream)
	if err != nil {
		t.Fatalf("Dirfd: %v", err)
	}
	if dirFd < 0 {
		t.Fatalf("Dirfd: fd < 0, (fd = %v)", dirFd)
	}

	name := fmt.Sprintf("mknodatTest%d", os.Getpid())
	fifoName := fmt.Sprintf("mknodatTestFIFO%d", os.Getpid())
	scfName := fmt.Sprintf("mknodatTestSCF%d", os.Getpid())

	err = unix.Mknodat(dirFd, name, unix.S_IFREG, 0)
	if err != nil {
		t.Fatalf("Mknodat - regular: %v", err)
	}
	defer os.Remove("/tmp/" + name)

	err = unix.Mknodat(dirFd, fifoName, unix.S_IFIFO, 0)
	if err != nil {
		t.Fatalf("Mknodat - directory: %v", err)
	}
	defer os.Remove("/tmp/" + fifoName)

	err = unix.Mknodat(dirFd, scfName, unix.S_IFCHR|unix.S_IRUSR|unix.S_IWUSR, 0x00010000|0x0001)
	if err != nil {
		t.Fatalf("Mknodat - character special file: %v", err)
	}
	defer os.Remove("/tmp/" + scfName)
}

func TestFchownat(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}

	f, err := os.Create(filepath.Join(t.TempDir(), "fchownatTestFile"))
	if err != nil {
		t.Fatalf("CreateTemp: %v", err)
	}
	defer f.Close()

	dir, err := os.Open(filepath.Dir(f.Name()))
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer dir.Close()

	dirfd := int(dir.Fd())

	err = unix.Fchownat(0, f.Name(), os.Getuid(), os.Getgid(), 0)
	if err != nil {
		t.Errorf("Fchownat: %v", err)
	}

	unix.Fchownat(dirfd, filepath.Base(f.Name()), os.Getuid(), os.Getgid(), 0)
	if err != nil {
		t.Errorf("Fchownat: %v", err)
	}
	err = unix.Fchownat(dirfd, "blah", os.Getuid(), os.Getgid(), 0)
	if err != nil {
		if !strings.Contains(err.Error(), "EDC5129I No such file or directory.") {
			t.Errorf("Expected: EDC5129I No such file or directory. Got: %v", err)
		}
	} else {
		t.Error("Fchownat: Expected to get error \"EDC5129I No such file or directory.\"")
	}
}

func TestFaccessat(t *testing.T) {
	f, err := os.CreateTemp("/tmp", "faccessatTestFile")
	if err != nil {
		t.Fatalf("CreateTemp: %v", err)
	}
	defer os.Remove(f.Name())
	defer f.Close()

	dirStream, err := unix.Opendir("/tmp")
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	defer unix.Closedir(dirStream)

	dirfd, err := unix.Dirfd(dirStream)
	if err != nil {
		t.Fatalf("Dirfd: %v", err)
	}
	if dirfd < 0 {
		t.Fatalf("Dirfd: fd < 0, (fd = %v)", dirfd)
	}

	err = unix.Faccessat(dirfd, filepath.Base(f.Name()), unix.R_OK|unix.W_OK, unix.AT_EACCESS)
	if err != nil {
		t.Errorf("Faccessat - relative file path: %v", err)
	}
	err = unix.Faccessat2(dirfd, filepath.Base(f.Name()), unix.R_OK|unix.W_OK, unix.AT_EACCESS)
	if err != nil {
		t.Errorf("Faccessat - relative file path: %v", err)
	}

	err = unix.Faccessat(dirfd, f.Name(), unix.R_OK|unix.W_OK, unix.AT_EACCESS)
	if err != nil {
		t.Errorf("Faccessat - absolute file path: %v", err)
	}

	err = unix.Faccessat(0, filepath.Base(f.Name()), unix.R_OK, unix.AT_EACCESS)
	if err != nil {
		if !strings.Contains(err.Error(), "EDC5135I Not a directory.") {
			t.Errorf("Expected: EDC5135I Not a directory. Got: %v", err)
		}
	} else {
		t.Error("Faccessat: Expected to get error \"EDC5135I Not a directory.\"")
	}

	err = unix.Faccessat(0, "/", unix.R_OK, unix.AT_EACCESS)
	if err != nil {
		t.Errorf("Faccessat - read root directory: %v", err)
	}

	err = unix.Faccessat(0, "/", unix.W_OK, unix.AT_EACCESS)
	if err != nil {
		if !strings.Contains(err.Error(), "EDC5141I Read-only file system.") {
			t.Errorf("Expected: EDC5141I Read-only file system. Got: %v", err)
		}
	} else {
		if BypassTestOnUntil("zoscan56", "2024-04-01T12:45:21.123Z") {
			fmt.Fprintf(os.Stderr, "Faccessat: Expected to get error \"EDC5141I Read-only file system.\"")
		} else {
			t.Error("Faccessat: Expected to get error \"EDC5141I Read-only file system.\"")
		}
	}
}

func TestUnlinkat(t *testing.T) {
	tmpdir := t.TempDir()
	f, err := os.CreateTemp(tmpdir, "unlinkatTestFile")
	if err != nil {
		log.Fatal("CreateTemp:", err)
	}
	// file close later

	dirStream, err := unix.Opendir(tmpdir)
	if err != nil {
		t.Fatalf("Opendir: %v", err)
	}
	defer unix.Closedir(dirStream)

	dirfd, err := unix.Dirfd(dirStream)
	if err != nil {
		t.Fatalf("Dirfd: %v", err)
	}
	if dirfd < 0 {
		t.Fatalf("Dirfd: fd < 0, (fd = %v)", dirfd)
	}

	if err := f.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	err = unix.Unlinkat(dirfd, filepath.Base(f.Name()), 0)
	if err != nil {
		t.Fatalf("Unlinkat: %v", err)
	}

	_, err = os.Open(f.Name())
	if err != nil {
		if !os.IsNotExist(err) {
			t.Errorf("Expected to get error \"EDC5129I No such file or directory\". Got: %v", err)
		}
	} else {
		t.Error("Unlinkat: Expected to get error \"EDC5129I No such file or directory\"")
	}

	err = unix.Unlinkat(dirfd, tmpdir, unix.AT_REMOVEDIR)
	if err != nil {
		t.Fatalf("Unlinkat: %v", err)
	}

	_, err = os.Open(tmpdir)
	if err != nil {
		if !os.IsNotExist(err) {
			t.Errorf("Expected to get error \"EDC5129I No such file or directory\". Got: %v", err)
		}
	} else {
		t.Error("Unlinkat: Expected to get error \"EDC5129I No such file or directory\"")
	}
}

func TestRenameat(t *testing.T) {
	defer chtmpdir(t)()

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

func TestRenameat2(t *testing.T) {
	defer chtmpdir(t)()

	from, to := "renamefrom", "renameto"

	touch(t, from)

	err := unix.Renameat2(unix.AT_FDCWD, from, unix.AT_FDCWD, to, 0)
	if err != nil {
		t.Fatalf("Renameat2: unexpected error: %v", err)
	}

	_, err = os.Stat(to)
	if err != nil {
		t.Error(err)
	}

	_, err = os.Stat(from)
	if err == nil {
		t.Errorf("Renameat2: stat of renamed file %q unexpectedly succeeded", from)
	}

	touch(t, from)

	err = unix.Renameat2(unix.AT_FDCWD, from, unix.AT_FDCWD, to, unix.RENAME_NOREPLACE)
	if err != nil {
		if err.Error() != "EDC5117I File exists." {
			t.Errorf("Renameat2: expected to get error \"EDC5117I File exists.\" Got: %v", err)
		}
	} else {
		t.Errorf("Renameat2: Unexpected error: %v", err)
	}
}

func TestFchmodat(t *testing.T) {
	defer chtmpdir(t)()

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
		if err == unix.EOPNOTSUPP {
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
func TestPosix_openpt(t *testing.T) {
	masterfd, err := unix.Posix_openpt(unix.O_RDWR)
	if err != nil {
		t.Fatal(err)
	}
	defer unix.Close(masterfd)
	_, err = unix.Grantpt(masterfd)
	if err != nil {
		t.Fatal(err)
	}
	_, err = unix.Unlockpt(masterfd)
	if err != nil {
		t.Fatal(err)
	}
	slavename, err := unix.Ptsname(masterfd)
	if err != nil {
		t.Fatal(err)
	}
	fd, err := unix.Open(slavename, unix.O_RDWR, 0)
	if err != nil {
		t.Fatal(err)
	}
	unix.Close(fd)
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
	defer chtmpdir(t)()

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

func TestFreezeUnfreeze(t *testing.T) {
	rv, rc, rn := unix.Bpx4ptq(unix.QUIESCE_FREEZE, "FREEZE")
	if rc != 0 {
		t.Fatalf(fmt.Sprintf("Bpx4ptq FREEZE %v %v %v\n", rv, rc, rn))
	}
	rv, rc, rn = unix.Bpx4ptq(unix.QUIESCE_UNFREEZE, "UNFREEZE")
	if rc != 0 {
		t.Fatalf(fmt.Sprintf("Bpx4ptq UNFREEZE %v %v %v\n", rv, rc, rn))
	}
}
func TestPtrace(t *testing.T) {
	cmd := exec.Command("/bin/sleep", "1000")
	cmd.Stdout = os.Stdout
	err := cmd.Start()
	if err != nil {
		log.Fatal(err)
	}
	rv, rc, rn := unix.Bpx4ptr(unix.PT_ATTACH, int32(cmd.Process.Pid), unsafe.Pointer(uintptr(0)), unsafe.Pointer(uintptr(0)), unsafe.Pointer(uintptr(0)))
	if rc != 0 {
		t.Fatalf("ptrace: Bpx4ptr rv %d, rc %d, rn %d\n", rv, rc, rn)
	}
	cmd.Process.Kill()
}

func TestFutimesat(t *testing.T) {
	// Create temp dir and file
	tempDir := t.TempDir()

	dir, err := os.Open(tempDir)
	if err != nil {
		t.Fatal("Can not open tempDir: ", tempDir)
	}
	defer dir.Close()

	tempFile, err := os.CreateTemp(tempDir, "futimesat_test_file")
	if err != nil {
		t.Fatalf("TempFile: %s", err.Error())
	}
	defer tempFile.Close()

	// Set mod time to newTime
	newTime := time.Date(2001, time.Month(2), 15, 7, 7, 7, 0, time.UTC)
	err = unix.Futimesat(
		int(dir.Fd()),
		filepath.Base(tempFile.Name()),
		[]unix.Timeval{
			unix.Timeval{Sec: newTime.Unix(), Usec: 0},
			unix.Timeval{Sec: newTime.Unix(), Usec: 0},
		})
	if err != nil {
		t.Fatalf("TestFutimes: %v", err)
	}

	// Compare mod time
	stats, err := tempFile.Stat()
	if err != nil {
		t.Fatalf("Stat: %v", err)
	}

	modTime := stats.ModTime()
	if modTime.UTC() != newTime {
		t.Fatalf("TestFutimes: modTime = %v, want %v", modTime.UTC(), newTime)
	}
}

func TestInotifyAccess(t *testing.T) {
	// Create temporary files
	tempFile, err := os.Create(filepath.Join(t.TempDir(), "inotify_access_test_file"))
	if err != nil {
		t.Fatalf("TempFile: %v", err)
	}
	defer tempFile.Close()

	// Setup iNotify
	infd, err := unix.InotifyInit()
	if err != nil {
		t.Fatalf("InotifyInit1: %v", err)
	}

	wd, err := unix.InotifyAddWatch(infd, tempFile.Name(), unix.IN_ACCESS)
	if err != nil {
		t.Fatalf("InotifyAddWatch: %v", err)
	}

	// Trigger Event
	n, err := tempFile.Write([]byte("Writing before reading"))
	if err != nil {
		t.Fatalf("Write: %v", err)
	}
	if n <= 0 {
		t.Fatalf("Did not write any data")
	}
	tempFile.Seek(0, 0)

	buf := make([]byte, 64)
	n, err = tempFile.Read(buf)
	if err != nil {
		t.Fatalf("Read: %v", err)
	}
	if n <= 0 {
		t.Fatalf("Did not read any data")
	}

	// Expect event
	buf = make([]byte, unix.SizeofInotifyEvent)
	n, err = unix.Read(infd, buf[:])
	if n == -1 {
		t.Fatalf("No event was read from the iNotify fd")
	}

	// Remove Watch
	if _, err = unix.InotifyRmWatch(infd, uint32(wd)); err != nil {
		t.Fatalf("InotifyRmWatch: %v", err)
	}
}

func TestAccess(t *testing.T) {
	tempFile, err := os.Create(filepath.Join(t.TempDir(), "test_access"))
	if err != nil {
		t.Fatal("fail to create temp file ", tempFile)
	}
	defer tempFile.Close()
	err = unix.Access(tempFile.Name(), unix.R_OK|unix.W_OK)
	if err != nil {
		t.Fatalf("error when access %s: %v", tempFile.Name(), err)
	}
	err = unix.Access("not_exist_file", unix.F_OK)
	if err == nil {
		t.Fatalf("error when access not exist file: %v", err)
	}
}

func TestCreat(t *testing.T) {
	tempFile, err := os.Create(filepath.Join(t.TempDir(), "test_create"))
	if err != nil {
		t.Fatal("fail to create temp file ", tempFile)
	}
	defer tempFile.Close()

	tempFile.Write([]byte("random1"))
	if err != nil {
		t.Fatal("error write to file: ", err)
	}
	// creat
	fd, err := unix.Creat(tempFile.Name(), 0o777)
	if err != nil {
		t.Fatal("Creat error: ", err)
	}
	writeContent := []byte("random2")
	n, err := unix.Write(fd, writeContent)
	if err != nil {
		t.Fatal("Write error: ", err)
	} else if n <= 0 {
		t.Fatal("Write error: 0 is written")
	}
	// Using creat is the equivalent of using the open callable service
	// with the create, truncate, and write-only options:
	// so we can not use the same file descriptor
	b, err := os.ReadFile(tempFile.Name())
	if err != nil {
		t.Fatal("Read error: ", err)
	}
	if n <= 0 {
		t.Fatal("Creat error: Cannot truncate file")
	}
	if string(b) != string(writeContent) {
		t.Fatal("data missmatch: expect ", string(writeContent), " actual: ", string(b))
	}

	// testing file create function
	newFile := tempFile.Name() + "2"
	fd2, err := unix.Creat(newFile, 0o777)
	if err != nil {
		t.Fatal("Creat error: ", err)
	}
	writeContent = []byte("random3")
	n, err = unix.Write(fd2, writeContent)
	if err != nil {
		t.Fatal("Write error: ", err)
	} else if n <= 0 {
		t.Fatal("Write error: 0 is written")
	}

	b, err = os.ReadFile(newFile)
	if err != nil {
		t.Fatal("Read error: ", err)
	}
	if n <= 0 {
		t.Fatal("Creat error: Cannot truncate file")
	}
	if string(b) != string(writeContent) {
		t.Fatal("data missmatch: expect ", string(writeContent), " actual: ", string(b))
	}

}

func TestGetPageSize(t *testing.T) {
	size := unix.Getpagesize()
	if size <= 0 {
		t.Fatal("get page size return: ", size)
	}
}

func TestSyscallSetegid(t *testing.T) {
	err := unix.Setegid(unix.Getgid())
	if err != nil {
		t.Fatal("error setting euid: ", err)
	}
	id := unix.Getegid()
	if id != unix.Getgid() {
		t.Fatal("euid mismatch: expect ", unix.Getgid(), ", actual ", id)
	}
}

func TestSyscallSeteuid(t *testing.T) {
	err := unix.Seteuid(unix.Getuid())
	if err != nil {
		t.Fatal("error setting euid: ", err)
	}
	id := unix.Geteuid()
	if id != unix.Getuid() {
		t.Fatal("euid mismatch: expect ", unix.Getuid(), ", actual ", id)
	}
}

func TestSyscallSetgid(t *testing.T) {
	err := unix.Setgid(unix.Getegid())
	if err != nil {
		t.Fatal("error setting gid: ", err)
	}
	id := unix.Getgid()
	if id != unix.Getegid() {
		t.Fatal("guid mismatch: expect 0, actual ", id)
	}
}

func TestSyscallSetpgid(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}

	pid := unix.Getpid()
	pgid, _ := unix.Getpgid(pid)
	err := unix.Setpgid(pid, pgid)
	if err != nil {
		t.Fatal("error setting pgid: ", err)
	}
	id, err := unix.Getpgid(pid)
	if err != nil {
		t.Fatal("Getpgid error: ", err)
	}
	if gid, _ := unix.Getpgid(pid); gid != id {
		t.Fatal("pgid mismatch: expect ", gid, ", actual ", id)
	}
}

func TestSyscallSetregid(t *testing.T) {
	gid := unix.Getgid()
	err := unix.Setregid(gid, gid)
	if err != nil {
		t.Fatal("error setting regid: ", err)
	}
	// currently missing Getresgid can not validate
	// The get function also not provided in syscall package as well as other platform
}

func TestSyscallSetreuid(t *testing.T) {
	uid := unix.Getuid()
	err := unix.Setreuid(uid, uid)
	if err != nil {
		t.Fatal("error setting reuid: ", err)
	}
	// currently missing Getresgid can not validate
	// The get function also not provided in syscall package as well as other platform

}

func TestWriteAndSync(t *testing.T) {
	// this test cannot really test sync function
	// since unix.write does not require a sync function to actual write to the file

	tempFile, err := os.Create(filepath.Join(t.TempDir(), "test_write_and_sync"))
	if err != nil {
		t.Fatal("error: ", err)
	}
	defer tempFile.Close()
	fileContent := "hello world"
	n, err := unix.Write(int(tempFile.Fd()), []byte(fileContent))
	if err != nil {
		t.Fatal("write error: ", err)
	}
	if n != len(fileContent) {
		t.Fatal("error: write length mismatch")
	}
	unix.Sync()

	b := make([]byte, len(fileContent), 256)
	unix.Seek(int(tempFile.Fd()), 0, 0)
	_, err = unix.Read(int(tempFile.Fd()), b)
	if err != nil {
		t.Fatal("read error: ", err)
	}
	if string(b) != fileContent {
		t.Fatal("file data mismatch: expect ", fileContent, " actual", string(b))
	}
}

func TestTimes(t *testing.T) {
	var startTimes, endTimes unix.Tms

	// Get the start time
	_, err := unix.Times(&startTimes)
	if err != nil {
		t.Fatal("times error: ", err)
	}

	sum := 0
	// Perform some operations
	for i := 0; i < 1000000000; i++ {
		sum += i % 100
	}

	// Get the end time
	_, err = unix.Times(&endTimes)
	if err != nil {
		t.Fatal("times error: ", err)
	}

	if int64(endTimes.Utime)-int64(startTimes.Utime) <= 0 || int64(endTimes.Stime)-int64(startTimes.Stime) <= 0 {
		t.Fatal("times error: endtime - starttime <= 0")
	}
}

func TestMlock(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}

	twoM := 2 * 1024 * 1024
	b := make([]byte, twoM, twoM)
	for i := 0; i < twoM; i++ {
		b[i] = byte(i % 127)
	}
	err := unix.Mlock(b)
	if err != nil {
		t.Fatal("mlock error: ", err)
	}
	for i := 0; i < twoM; i++ {
		if b[i] != byte(i%127) {
			t.Fatal("error: memory not correct: expect ", i%127, " actual ", b[i])
		}
	}

	err = unix.Munlock(b)
	if err != nil {
		t.Fatal("munlock error: ", err)
	}
	for i := 0; i < twoM; i++ {
		if b[i] != byte(i%127) {
			t.Fatal("error: memory not correct: expect ", i%127, " actual ", b[i])
		}
	}

}

func TestMlockAll(t *testing.T) {
	if euid != 0 {
		t.Skip("euid != 0")
	}

	twoM := 2 * 1024 * 1024
	b := make([]byte, twoM, twoM)
	for i := 0; i < twoM; i++ {
		b[i] = byte(i % 127)
	}
	// Mlockall flag do not have zos semantics, so passing 0
	err := unix.Mlockall(0)
	if err != nil {
		t.Fatal("mlock error: ", err)
	}
	for i := 0; i < twoM; i++ {
		if b[i] != byte(i%127) {
			t.Fatal("error: memory not correct: expect ", i%127, " actual ", b[i])
		}
	}

	err = unix.Munlockall()
	if err != nil {
		t.Fatal("munlock error: ", err)
	}
	for i := 0; i < twoM; i++ {
		if b[i] != byte(i%127) {
			t.Fatal("error: memory not correct: expect ", i%127, " actual ", b[i])
		}
	}
}

func TestGettid(t *testing.T) {
	tid := unix.Gettid()
	if tid < 0 {
		t.Fatal("error: tid less than 0: tid = ", tid)
	}
}

func TestSetns(t *testing.T) {
	// TODO (joon): for some reason changing ipc on zos fails
	namespaces := map[string]int{
		// "ipc": unix.CLONE_NEWIPC,
		"uts": unix.CLONE_NEWUTS,
		"net": unix.CLONE_NEWNET,
		// "pid": unix.CLONE_NEWPID,
	}

	if unix.Geteuid() != 0 {
		t.Skip("euid != 0")
	}

	if os.Getenv("SETNS_HELPER_PROCESS") == "1" {
		pid := unix.Getppid()

		fmt.Scanln()

		for k, v := range namespaces {
			path := fmt.Sprintf("/proc/%d/ns/%s", pid, k)
			fd, err := unix.Open(path, unix.O_RDONLY, 0)
			err = unix.Setns(fd, v)
			if err != nil {
				t.Fatalf("Setns failed: %v", err)
			}
		}
		for {
		}
	}

	cmd := exec.Command(os.Args[0], "-test.run=^TestSetns$")
	cmd.Env = append(os.Environ(), "SETNS_HELPER_PROCESS=1")
	stdin, err := cmd.StdinPipe()
	if err != nil {
		t.Fatalf("Failed to get stdin for helper process: %v\n", err)
	}

	if err := cmd.Start(); err != nil {
		t.Fatalf("failed to create helper process: %v\n", err)
	}
	defer cmd.Process.Kill()

	ppid := unix.Getpid()
	pid := cmd.Process.Pid

	for k, _ := range namespaces {
		hPath := fmt.Sprintf("/proc/%d/ns/%s", pid, k)
		pPath := fmt.Sprintf("/proc/%d/ns/%s", ppid, k)

		hFI, _ := os.Stat(hPath)
		pFI, _ := os.Stat(pPath)

		if !os.SameFile(hFI, pFI) {
			t.Fatalf("namespace links for %s did not match before calling Unshare in parent\n", k)
		}
	}

	unix.Unshare(unix.CLONE_NEWUTS | unix.CLONE_NEWNET)

	for k, _ := range namespaces {
		hPath := fmt.Sprintf("/proc/%d/ns/%s", pid, k)
		pPath := fmt.Sprintf("/proc/%d/ns/%s", ppid, k)

		hFI, _ := os.Stat(hPath)
		pFI, _ := os.Stat(pPath)

		if os.SameFile(hFI, pFI) {
			t.Fatalf("Setns: namespace link for %s matched after calling Unshare but before Setns\n", k)
		}
	}

	stdin.Write([]byte("\n"))
	stdin.Close()
	time.Sleep(1000 * time.Millisecond)

	for k, _ := range namespaces {
		hPath := fmt.Sprintf("/proc/%d/ns/%s", pid, k)
		pPath := fmt.Sprintf("/proc/%d/ns/%s", ppid, k)

		hFI, _ := os.Stat(hPath)
		pFI, _ := os.Stat(pPath)

		if !os.SameFile(hFI, pFI) {
			t.Errorf("Setns: namespace link for %s did not match after calling Setns\n", k)
		}
	}

}

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

// This test is based on mmap_unix_test, but tweaked for z/OS, which does not support memadvise
// or anonymous mmapping.
func TestConsole2(t *testing.T) {
	var cmsg unix.ConsMsg2
	var nullptr *byte
	var cmsg_cmd uint32

	cmsg_rout := [2]uint32{1, 0}
	cmsg_desc := [2]uint32{12, 0}
	cmsg.Cm2Format = unix.CONSOLE_FORMAT_2
	msg := "__console2 test"
	cmsg.Cm2Msg = &unix.ZosStringToEbcdicBytes(msg, true)[0]
	cmsg.Cm2Msglength = uint32(len(msg))
	cmsg.Cm2Routcde = &cmsg_rout[0]
	cmsg.Cm2Descr = &cmsg_desc[0]
	cmsg.Cm2Token = 0

	err := unix.Console2(&cmsg, nullptr, &cmsg_cmd)
	if err != nil {
		t.Fatalf("__console2: %v", err)
	}
}

func TestConsole2modify(t *testing.T) {
	if os.Getenv("ZOS_MANUAL_TEST") != "1" {
		t.Skip("This test is not run unless env-var ZOS_MANUAL_TEST=1 is set")
	}

	job, err := unix.ZosJobname()
	if err != nil {
		t.Fatalf("Failed to get jobname  %v", err)
	}

	var cmsg unix.ConsMsg2
	var cmsg_cmd uint32
	cmsg_rout := [2]uint32{1, 0}
	cmsg_desc := [2]uint32{12, 0}
	cmsg.Cm2Format = unix.CONSOLE_FORMAT_2
	msg := "Issue console command 'F " + job + ",APPL=123' to continue"
	cmsg.Cm2Msg = &unix.ZosStringToEbcdicBytes(msg, true)[0]
	cmsg.Cm2Msglength = uint32(len(msg))
	cmsg.Cm2Routcde = &cmsg_rout[0]
	cmsg.Cm2Descr = &cmsg_desc[0]
	cmsg.Cm2Token = 0

	var modstr [128]byte
	t.Logf("Issue console command 'F %s,APPL=123' to continue\n", job)

	err = unix.Console2(&cmsg, &modstr[0], &cmsg_cmd)
	if err != nil {
		t.Fatalf("__console2: %v", err)
	}

	recv := unix.ZosEbcdicBytesToString(modstr[:], true)
	if recv != "123" || cmsg_cmd != 1 {
		t.Fatalf("__console2 modify: Got %s %x, Expect 123 1\n", unix.ZosEbcdicBytesToString(modstr[:], true), cmsg_cmd)
	}

	t.Logf("Got %s %x\n", unix.ZosEbcdicBytesToString(modstr[:], true), cmsg_cmd)
}
