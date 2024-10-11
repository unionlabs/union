// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package unix_test

import (
	"bytes"
	"net"
	"os"
	"path/filepath"
	"testing"

	"golang.org/x/sys/unix"
)

var testData = []byte("This is a test\n")

// stringsFromByteSlice converts a sequence of attributes to a []string.
// On Darwin, each entry is a NULL-terminated string.
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

func createTestFile(t *testing.T) string {
	filename := filepath.Join(t.TempDir(), t.Name())
	err := os.WriteFile(filename, testData, 0600)
	if err != nil {
		t.Fatal(err)
	}
	return filename
}

func TestClonefile(t *testing.T) {
	fileName := createTestFile(t)

	clonedName := fileName + "-cloned"
	err := unix.Clonefile(fileName, clonedName, 0)
	if err == unix.ENOSYS || err == unix.ENOTSUP {
		t.Skip("clonefile is not available or supported, skipping test")
	} else if err != nil {
		t.Fatal(err)
	}

	clonedData, err := os.ReadFile(clonedName)
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(testData, clonedData) {
		t.Errorf("Clonefile: got %q, expected %q", clonedData, testData)
	}
}

func TestClonefileatWithCwd(t *testing.T) {
	fileName := createTestFile(t)

	clonedName := fileName + "-cloned"
	err := unix.Clonefileat(unix.AT_FDCWD, fileName, unix.AT_FDCWD, clonedName, 0)
	if err == unix.ENOSYS || err == unix.ENOTSUP {
		t.Skip("clonefileat is not available or supported, skipping test")
	} else if err != nil {
		t.Fatal(err)
	}

	clonedData, err := os.ReadFile(clonedName)
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(testData, clonedData) {
		t.Errorf("Clonefileat: got %q, expected %q", clonedData, testData)
	}
}

func TestClonefileatWithRelativePaths(t *testing.T) {
	srcFileName := createTestFile(t)
	srcDir := filepath.Dir(srcFileName)
	srcFd, err := unix.Open(srcDir, unix.O_RDONLY|unix.O_DIRECTORY, 0)
	if err != nil {
		t.Fatal(err)
	}
	defer unix.Close(srcFd)

	dstDir := t.TempDir()
	dstFd, err := unix.Open(dstDir, unix.O_RDONLY|unix.O_DIRECTORY, 0)
	if err != nil {
		t.Fatal(err)
	}
	defer unix.Close(dstFd)

	dstFile, err := os.Create(filepath.Join(dstDir, "TestClonefileat"))
	if err != nil {
		t.Fatal(err)
	}
	err = os.Remove(dstFile.Name())
	if err != nil {
		t.Fatal(err)
	}

	src := filepath.Base(srcFileName)
	dst := filepath.Base(dstFile.Name())
	err = unix.Clonefileat(srcFd, src, dstFd, dst, 0)
	if err == unix.ENOSYS || err == unix.ENOTSUP {
		t.Skip("clonefileat is not available or supported, skipping test")
	} else if err != nil {
		t.Fatal(err)
	}

	clonedData, err := os.ReadFile(dstFile.Name())
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(testData, clonedData) {
		t.Errorf("Clonefileat: got %q, expected %q", clonedData, testData)
	}
}

func TestFclonefileat(t *testing.T) {
	fileName := createTestFile(t)
	dir := filepath.Dir(fileName)

	fd, err := unix.Open(fileName, unix.O_RDONLY, 0)
	if err != nil {
		t.Fatal(err)
	}
	defer unix.Close(fd)

	dstFile, err := os.Create(filepath.Join(dir, "dst"))
	if err != nil {
		t.Fatal(err)
	}
	os.Remove(dstFile.Name())

	err = unix.Fclonefileat(fd, unix.AT_FDCWD, dstFile.Name(), 0)
	if err == unix.ENOSYS || err == unix.ENOTSUP {
		t.Skip("clonefileat is not available or supported, skipping test")
	} else if err != nil {
		t.Fatal(err)
	}

	clonedData, err := os.ReadFile(dstFile.Name())
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(testData, clonedData) {
		t.Errorf("Fclonefileat: got %q, expected %q", clonedData, testData)
	}
}

func TestFcntlFstore(t *testing.T) {
	f, err := os.CreateTemp(t.TempDir(), t.Name())
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	fstore := &unix.Fstore_t{
		Flags:   unix.F_ALLOCATEALL,
		Posmode: unix.F_PEOFPOSMODE,
		Offset:  0,
		Length:  1 << 10,
	}
	err = unix.FcntlFstore(f.Fd(), unix.F_PREALLOCATE, fstore)
	if err == unix.EOPNOTSUPP {
		t.Skipf("fcntl with F_PREALLOCATE not supported, skipping test")
	} else if err != nil {
		t.Fatalf("FcntlFstore: %v", err)
	}

	st, err := f.Stat()
	if err != nil {
		t.Fatal(err)
	}

	if st.Size() != 0 {
		t.Errorf("FcntlFstore: got size = %d, want %d", st.Size(), 0)
	}

}

func TestGetsockoptXucred(t *testing.T) {
	fds, err := unix.Socketpair(unix.AF_LOCAL, unix.SOCK_STREAM, 0)
	if err != nil {
		t.Fatalf("Socketpair: %v", err)
	}

	srvFile := os.NewFile(uintptr(fds[0]), "server")
	cliFile := os.NewFile(uintptr(fds[1]), "client")
	defer srvFile.Close()
	defer cliFile.Close()

	srv, err := net.FileConn(srvFile)
	if err != nil {
		t.Fatalf("FileConn: %v", err)
	}
	defer srv.Close()

	cli, err := net.FileConn(cliFile)
	if err != nil {
		t.Fatalf("FileConn: %v", err)
	}
	defer cli.Close()

	cred, err := unix.GetsockoptXucred(fds[1], unix.SOL_LOCAL, unix.LOCAL_PEERCRED)
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("got: %+v", cred)
	if got, want := cred.Uid, os.Getuid(); int(got) != int(want) {
		t.Errorf("uid = %v; want %v", got, want)
	}
	if cred.Ngroups > 0 {
		if got, want := cred.Groups[0], os.Getgid(); int(got) != int(want) {
			t.Errorf("gid = %v; want %v", got, want)
		}
	}
}

func TestSysctlKinfoProc(t *testing.T) {
	pid := unix.Getpid()
	kp, err := unix.SysctlKinfoProc("kern.proc.pid", pid)
	if err != nil {
		t.Fatalf("SysctlKinfoProc: %v", err)
	}
	if got, want := int(kp.Proc.P_pid), pid; got != want {
		t.Errorf("got pid %d, want %d", got, want)
	}
}

func TestSysctlKinfoProcSlice(t *testing.T) {
	kps, err := unix.SysctlKinfoProcSlice("kern.proc.all")
	if err != nil {
		t.Fatalf("SysctlKinfoProc: %v", err)
	}
	if len(kps) == 0 {
		t.Errorf("SysctlKinfoProcSlice: expected at least one process")
	}

	uid := unix.Getuid()
	kps, err = unix.SysctlKinfoProcSlice("kern.proc.uid", uid)
	if err != nil {
		t.Fatalf("SysctlKinfoProc: %v", err)
	}
	if len(kps) == 0 {
		t.Errorf("SysctlKinfoProcSlice: expected at least one process")
	}

	for _, kp := range kps {
		if got, want := int(kp.Eproc.Ucred.Uid), uid; got != want {
			t.Errorf("process %d: got uid %d, want %d", kp.Proc.P_pid, got, want)
		}
	}
}
