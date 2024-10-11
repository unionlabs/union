// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

// mkpost processes the output of cgo -godefs to
// modify the generated types. It is used to clean up
// the sys API in an architecture specific manner.
//
// mkpost is run after cgo -godefs; see README.md.
package main

import (
	"bytes"
	"fmt"
	"go/format"
	"io"
	"log"
	"os"
	"regexp"
)

func main() {
	// Get the OS and architecture (using GOARCH_TARGET if it exists)
	goos := os.Getenv("GOOS_TARGET")
	if goos == "" {
		goos = os.Getenv("GOOS")
	}
	goarch := os.Getenv("GOARCH_TARGET")
	if goarch == "" {
		goarch = os.Getenv("GOARCH")
	}
	// Check that we are using the Docker-based build system if we should be.
	if goos == "linux" {
		if os.Getenv("GOLANG_SYS_BUILD") != "docker" {
			os.Stderr.WriteString("In the Docker-based build system, mkpost should not be called directly.\n")
			os.Stderr.WriteString("See README.md\n")
			os.Exit(1)
		}
	}

	b, err := io.ReadAll(os.Stdin)
	if err != nil {
		log.Fatal(err)
	}

	if goos == "aix" {
		// Replace type of Atim, Mtim and Ctim by Timespec in Stat_t
		// to avoid having both StTimespec and Timespec.
		sttimespec := regexp.MustCompile(`_Ctype_struct_st_timespec`)
		b = sttimespec.ReplaceAll(b, []byte("Timespec"))
	}

	if goos == "darwin" {
		// KinfoProc contains various pointers to objects stored
		// in kernel space. Replace these by uintptr to prevent
		// accidental dereferencing.
		kinfoProcPointerRegex := regexp.MustCompile(`\*_Ctype_struct_(pgrp|proc|session|sigacts|ucred|user|vnode)`)
		b = kinfoProcPointerRegex.ReplaceAll(b, []byte("uintptr"))

		// ExternProc contains a p_un member that in kernel
		// space stores a pair of pointers and in user space
		// stores the process creation time. We only care about
		// the process creation time.
		externProcStarttimeRegex := regexp.MustCompile(`P_un\s*\[\d+\]byte`)
		b = externProcStarttimeRegex.ReplaceAll(b, []byte("P_starttime Timeval"))

		// Convert [n]int8 to [n]byte in Eproc and ExternProc members to
		// simplify conversion to string.
		convertEprocRegex := regexp.MustCompile(`(P_comm|Wmesg|Login)(\s+)\[(\d+)\]int8`)
		b = convertEprocRegex.ReplaceAll(b, []byte("$1$2[$3]byte"))
	}

	if goos == "freebsd" {
		// Inside PtraceLwpInfoStruct replace __Siginfo with __PtraceSiginfo,
		// Create __PtraceSiginfo as a copy of __Siginfo where every *byte instance is replaced by uintptr
		ptraceLwpInfoStruct := regexp.MustCompile(`(?s:type PtraceLwpInfoStruct struct \{.*?\})`)
		b = ptraceLwpInfoStruct.ReplaceAllFunc(b, func(in []byte) []byte {
			return bytes.ReplaceAll(in, []byte("__Siginfo"), []byte("__PtraceSiginfo"))
		})

		siginfoStruct := regexp.MustCompile(`(?s:type __Siginfo struct \{.*?\})`)
		b = siginfoStruct.ReplaceAllFunc(b, func(in []byte) []byte {
			out := append([]byte{}, in...)
			out = append(out, '\n', '\n')
			out = append(out,
				bytes.ReplaceAll(
					bytes.ReplaceAll(in, []byte("__Siginfo"), []byte("__PtraceSiginfo")),
					[]byte("*byte"), []byte("uintptr"))...)
			return out
		})

		// Inside PtraceIoDesc replace the Offs field, which refers to an address
		// in the child process (not the Go parent), with a uintptr.
		ptraceIoDescStruct := regexp.MustCompile(`(?s:type PtraceIoDesc struct \{.*?\})`)
		addrField := regexp.MustCompile(`(\bOffs\s+)\*byte`)
		b = ptraceIoDescStruct.ReplaceAllFunc(b, func(in []byte) []byte {
			return addrField.ReplaceAll(in, []byte(`${1}uintptr`))
		})
	}

	if goos == "solaris" {
		// Convert *int8 to *byte in Iovec.Base like on every other platform.
		convertIovecBase := regexp.MustCompile(`Base\s+\*int8`)
		iovecType := regexp.MustCompile(`type Iovec struct {[^}]*}`)
		iovecStructs := iovecType.FindAll(b, -1)
		for _, s := range iovecStructs {
			newNames := convertIovecBase.ReplaceAll(s, []byte("Base *byte"))
			b = bytes.Replace(b, s, newNames, 1)
		}
	}

	if goos == "linux" && goarch != "riscv64" {
		// The RISCV_HWPROBE_ constants are only defined on Linux for riscv64
		hwprobeConstRexexp := regexp.MustCompile(`const\s+\(\s+RISCV_HWPROBE_[^\)]+\)`)
		b = hwprobeConstRexexp.ReplaceAll(b, nil)
	}

	// Intentionally export __val fields in Fsid and Sigset_t
	valRegex := regexp.MustCompile(`type (Fsid|Sigset_t) struct {(\s+)X__(bits|val)(\s+\S+\s+)}`)
	b = valRegex.ReplaceAll(b, []byte("type $1 struct {${2}Val$4}"))

	// Intentionally export __fds_bits field in FdSet
	fdSetRegex := regexp.MustCompile(`type (FdSet) struct {(\s+)X__fds_bits(\s+\S+\s+)}`)
	b = fdSetRegex.ReplaceAll(b, []byte("type $1 struct {${2}Bits$3}"))

	// Intentionally export __icmp6_filt field in icmpv6_filter
	icmpV6Regex := regexp.MustCompile(`type (ICMPv6Filter) struct {(\s+)X__icmp6_filt(\s+\S+\s+)}`)
	b = icmpV6Regex.ReplaceAll(b, []byte("type $1 struct {${2}Filt$3}"))

	// Intentionally export address storage field in SockaddrStorage convert it to [N]byte.
	convertSockaddrStorageData := regexp.MustCompile(`(X__ss_padding)\s+\[(\d+)\]u?int8`)
	sockaddrStorageType := regexp.MustCompile(`type SockaddrStorage struct {[^}]*}`)
	sockaddrStorageStructs := sockaddrStorageType.FindAll(b, -1)
	for _, s := range sockaddrStorageStructs {
		newNames := convertSockaddrStorageData.ReplaceAll(s, []byte("Data [$2]byte"))
		b = bytes.Replace(b, s, newNames, 1)
	}

	// If we have empty Ptrace structs, we should delete them. Only s390x emits
	// nonempty Ptrace structs.
	ptraceRexexp := regexp.MustCompile(`type Ptrace((Psw|Fpregs|Per) struct {\s*})`)
	b = ptraceRexexp.ReplaceAll(b, nil)

	// If we have an empty RISCVHWProbePairs struct, we should delete it. Only riscv64 emits
	// nonempty RISCVHWProbePairs structs.
	hwprobeRexexp := regexp.MustCompile(`type RISCVHWProbePairs struct {\s*}`)
	b = hwprobeRexexp.ReplaceAll(b, nil)

	// Replace the control_regs union with a blank identifier for now.
	controlRegsRegex := regexp.MustCompile(`(Control_regs)\s+\[0\]uint64`)
	b = controlRegsRegex.ReplaceAll(b, []byte("_ [0]uint64"))

	// Remove fields that are added by glibc
	// Note that this is unstable as the identifers are private.
	removeFieldsRegex := regexp.MustCompile(`X__glibc\S*`)
	b = removeFieldsRegex.ReplaceAll(b, []byte("_"))

	// Convert [65]int8 to [65]byte in Utsname members to simplify
	// conversion to string; see golang.org/issue/20753
	convertUtsnameRegex := regexp.MustCompile(`((Sys|Node|Domain)name|Release|Version|Machine)(\s+)\[(\d+)\]u?int8`)
	b = convertUtsnameRegex.ReplaceAll(b, []byte("$1$3[$4]byte"))

	// Convert [n]int8 to [n]byte in Statvfs_t and Statfs_t members to simplify
	// conversion to string.
	convertStatvfsRegex := regexp.MustCompile(`(([Ff]stype|[Mm]nton|[Mm]ntfrom)name|mntfromspec)(\s+)\[(\d+)\]int8`)
	b = convertStatvfsRegex.ReplaceAll(b, []byte("$1$3[$4]byte"))

	// Convert []int8 to []byte in device mapper ioctl interface
	convertDmIoctlNames := regexp.MustCompile(`(Name|Uuid|Target_type|Data)(\s+)\[(\d+)\]u?int8`)
	dmIoctlTypes := regexp.MustCompile(`type Dm(\S+) struct {[^}]*}`)
	dmStructs := dmIoctlTypes.FindAll(b, -1)
	for _, s := range dmStructs {
		newNames := convertDmIoctlNames.ReplaceAll(s, []byte("$1$2[$3]byte"))
		b = bytes.Replace(b, s, newNames, 1)
	}

	// Convert []int8 to []byte in EthtoolDrvinfo
	convertEthtoolDrvinfoNames := regexp.MustCompile(`(Driver|Version|Fw_version|Bus_info|Erom_version|Reserved2)(\s+)\[(\d+)\]u?int8`)
	ethtoolDrvinfoTypes := regexp.MustCompile(`type EthtoolDrvinfo struct {[^}]*}`)
	ethtoolDrvinfoStructs := ethtoolDrvinfoTypes.FindAll(b, -1)
	for _, s := range ethtoolDrvinfoStructs {
		newNames := convertEthtoolDrvinfoNames.ReplaceAll(s, []byte("$1$2[$3]byte"))
		b = bytes.Replace(b, s, newNames, 1)
	}

	// Convert []int8 to []byte in ctl_info ioctl interface
	convertCtlInfoName := regexp.MustCompile(`(Name)(\s+)\[(\d+)\]int8`)
	ctlInfoType := regexp.MustCompile(`type CtlInfo struct {[^}]*}`)
	ctlInfoStructs := ctlInfoType.FindAll(b, -1)
	for _, s := range ctlInfoStructs {
		newNames := convertCtlInfoName.ReplaceAll(s, []byte("$1$2[$3]byte"))
		b = bytes.Replace(b, s, newNames, 1)
	}

	// Convert [1024]int8 to [1024]byte in Ptmget members
	convertPtmget := regexp.MustCompile(`([SC]n)(\s+)\[(\d+)\]u?int8`)
	b = convertPtmget.ReplaceAll(b, []byte("$1[$3]byte"))

	// Remove spare fields (e.g. in Statx_t)
	spareFieldsRegex := regexp.MustCompile(`X__spare\S*`)
	b = spareFieldsRegex.ReplaceAll(b, []byte("_"))

	// Rename chunk_size field in XDPUmemReg.
	// When the struct was originally added (CL 136695) the only
	// field with a prefix was chunk_size, so cgo rewrote the
	// field to Size. Later Linux added a tx_metadata_len field,
	// so cgo left chunk_size as Chunk_size (CL 577975).
	// Go back to Size so that packages like gvisor don't have
	// to adjust.
	xdpUmemRegType := regexp.MustCompile(`type XDPUmemReg struct {[^}]*}`)
	xdpUmemRegStructs := xdpUmemRegType.FindAll(b, -1)
	for _, s := range xdpUmemRegStructs {
		newName := bytes.Replace(s, []byte("Chunk_size"), []byte("Size"), 1)
		b = bytes.Replace(b, s, newName, 1)
	}

	// Remove cgo padding fields
	removePaddingFieldsRegex := regexp.MustCompile(`Pad_cgo_\d+`)
	b = removePaddingFieldsRegex.ReplaceAll(b, []byte("_"))

	// Remove padding, hidden, or unused fields
	removeFieldsRegex = regexp.MustCompile(`\b(X_\S+|Padding)`)
	b = removeFieldsRegex.ReplaceAll(b, []byte("_"))

	// Remove the first line of warning from cgo
	b = b[bytes.IndexByte(b, '\n')+1:]
	// Modify the command in the header to include:
	//  mkpost, our own warning, and a build tag.
	replacement := fmt.Sprintf(`$1 | go run mkpost.go
// Code generated by the command above; see README.md. DO NOT EDIT.

//go:build %s && %s`, goarch, goos)
	cgoCommandRegex := regexp.MustCompile(`(cgo -godefs .*)`)
	b = cgoCommandRegex.ReplaceAll(b, []byte(replacement))

	// Rename Stat_t time fields
	if goos == "freebsd" && goarch == "386" {
		// Hide Stat_t.[AMCB]tim_ext fields
		renameStatTimeExtFieldsRegex := regexp.MustCompile(`[AMCB]tim_ext`)
		b = renameStatTimeExtFieldsRegex.ReplaceAll(b, []byte("_"))
	}
	renameStatTimeFieldsRegex := regexp.MustCompile(`([AMCB])(?:irth)?time?(?:spec)?\s+(Timespec|StTimespec)`)
	b = renameStatTimeFieldsRegex.ReplaceAll(b, []byte("${1}tim ${2}"))

	// gofmt
	b, err = format.Source(b)
	if err != nil {
		log.Fatal(err)
	}

	os.Stdout.Write(b)
}
