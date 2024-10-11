// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"sort"
	"strconv"
	"strings"
)

var (
	goos, goarch string
)

// cmdLine returns this programs's commandline arguments
func cmdLine() string {
	return "go run linux/mksysnum.go " + strings.Join(os.Args[1:], " ")
}

// goBuildTags returns build tags in the go:build format.
func goBuildTags() string {
	return fmt.Sprintf("%s && %s", goarch, goos)
}

func format(name string, num int, offset int) (int, string) {
	if num > 999 {
		// ignore deprecated syscalls that are no longer implemented
		// https://git.kernel.org/cgit/linux/kernel/git/torvalds/linux.git/tree/include/uapi/asm-generic/unistd.h?id=refs/heads/master#n716
		return 0, ""
	}
	name = strings.ToUpper(name)
	num = num + offset
	return num, fmt.Sprintf("	SYS_%s = %d;\n", name, num)
}

func checkErr(err error) {
	if err != nil {
		fmt.Fprintf(os.Stderr, "%v\n", err)
		os.Exit(1)
	}
}

// source string and substring slice for regexp
type re struct {
	str string   // source string
	sub []string // matched sub-string
}

// Match performs regular expression match
func (r *re) Match(exp string) bool {
	r.sub = regexp.MustCompile(exp).FindStringSubmatch(r.str)
	if r.sub != nil {
		return true
	}
	return false
}

// syscallNum holds the syscall number and the string
// we will write to the generated file.
type syscallNum struct {
	num         int
	declaration string
}

// syscallNums is a slice of syscallNum sorted by the syscall number in ascending order.
type syscallNums []syscallNum

// addSyscallNum adds the syscall declaration to syscallNums.
func (nums *syscallNums) addSyscallNum(num int, declaration string) {
	if declaration == "" {
		return
	}
	if len(*nums) == 0 || (*nums)[len(*nums)-1].num <= num {
		// This is the most common case as the syscall declarations output by the preprocessor
		// are almost always sorted.
		*nums = append(*nums, syscallNum{num, declaration})
		return
	}
	i := sort.Search(len(*nums), func(i int) bool { return (*nums)[i].num >= num })

	// Maintain the ordering in the preprocessor output when we have multiple definitions with
	// the same value. i cannot be > len(nums) - 1 as nums[len(nums)-1].num > num.
	for ; (*nums)[i].num == num; i++ {
	}
	*nums = append((*nums)[:i], append([]syscallNum{{num, declaration}}, (*nums)[i:]...)...)
}

func main() {
	// Get the OS and architecture (using GOARCH_TARGET if it exists)
	goos = os.Getenv("GOOS")
	goarch = os.Getenv("GOARCH_TARGET")
	if goarch == "" {
		goarch = os.Getenv("GOARCH")
	}
	// Check if GOOS and GOARCH environment variables are defined
	if goarch == "" || goos == "" {
		fmt.Fprintf(os.Stderr, "GOARCH or GOOS not defined in environment\n")
		os.Exit(1)
	}
	// Check that we are using the new build system if we should
	if os.Getenv("GOLANG_SYS_BUILD") != "docker" {
		fmt.Fprintf(os.Stderr, "In the new build system, mksysnum should not be called directly.\n")
		fmt.Fprintf(os.Stderr, "See README.md\n")
		os.Exit(1)
	}

	cc := os.Getenv("CC")
	if cc == "" {
		fmt.Fprintf(os.Stderr, "CC is not defined in environment\n")
		os.Exit(1)
	}
	args := os.Args[1:]
	args = append([]string{"-E", "-dD"}, args...)
	cmd, err := exec.Command(cc, args...).Output() // execute command and capture output
	if err != nil {
		fmt.Fprintf(os.Stderr, "can't run %s", cc)
		os.Exit(1)
	}
	s := bufio.NewScanner(strings.NewReader(string(cmd)))
	var offset, prev, asOffset int
	var nums syscallNums
	for s.Scan() {
		t := re{str: s.Text()}

		// The generated zsysnum_linux_*.go files for some platforms (arm64, loong64, riscv64)
		// treat SYS_ARCH_SPECIFIC_SYSCALL as if it's a syscall which it isn't.  It's an offset.
		// However, as this constant is already part of the public API we leave it in place.
		// Lines of type SYS_ARCH_SPECIFIC_SYSCALL = 244 are thus processed twice, once to extract
		// the offset and once to add the constant.

		if t.Match(`^#define __NR_arch_specific_syscall\s+([0-9]+)`) {
			// riscv: extract arch specific offset
			asOffset, _ = strconv.Atoi(t.sub[1]) // Make asOffset=0 if empty or non-numeric
		}

		if t.Match(`^#define __NR_Linux\s+([0-9]+)`) {
			// mips/mips64: extract offset
			offset, _ = strconv.Atoi(t.sub[1]) // Make offset=0 if empty or non-numeric
		} else if t.Match(`^#define __NR(\w*)_SYSCALL_BASE\s+([0-9]+)`) {
			// arm: extract offset
			offset, _ = strconv.Atoi(t.sub[1]) // Make offset=0 if empty or non-numeric
		} else if t.Match(`^#define __NR_syscalls\s+`) {
			// ignore redefinitions of __NR_syscalls
		} else if t.Match(`^#define __NR_(\w*)Linux_syscalls\s+`) {
			// mips/mips64: ignore definitions about the number of syscalls
		} else if t.Match(`^#define __NR_(\w+)\s+([0-9]+)`) {
			prev, err = strconv.Atoi(t.sub[2])
			checkErr(err)
			nums.addSyscallNum(format(t.sub[1], prev, offset))
		} else if t.Match(`^#define __NR3264_(\w+)\s+([0-9]+)`) {
			prev, err = strconv.Atoi(t.sub[2])
			checkErr(err)
			nums.addSyscallNum(format(t.sub[1], prev, offset))
		} else if t.Match(`^#define __NR_(\w+)\s+\(\w+\+\s*([0-9]+)\)`) {
			r2, err := strconv.Atoi(t.sub[2])
			checkErr(err)
			nums.addSyscallNum(format(t.sub[1], prev+r2, offset))
		} else if t.Match(`^#define __NR_(\w+)\s+\(__NR_(?:SYSCALL_BASE|Linux) \+ ([0-9]+)`) {
			r2, err := strconv.Atoi(t.sub[2])
			checkErr(err)
			nums.addSyscallNum(format(t.sub[1], r2, offset))
		} else if asOffset != 0 && t.Match(`^#define __NR_(\w+)\s+\(__NR_arch_specific_syscall \+ ([0-9]+)`) {
			r2, err := strconv.Atoi(t.sub[2])
			checkErr(err)
			nums.addSyscallNum(format(t.sub[1], r2, asOffset))
		}
	}
	err = s.Err()
	checkErr(err)
	var text strings.Builder
	for _, num := range nums {
		text.WriteString(num.declaration)
	}
	fmt.Printf(template, cmdLine(), goBuildTags(), text.String())
}

const template = `// %s
// Code generated by the command above; see README.md. DO NOT EDIT.

//go:build %s

package unix

const(
%s)`
