package cpuid

import (
	"os/exec"
	"strconv"
	"strings"
	"testing"
)

// Tests that the returned L1 instruction cache value matches the value returned from
// a call to the OS `sysctl` utility. Skips the test if we can't run it.
func TestDarwinL1ICache(t *testing.T) {
	out, err := exec.Command("/usr/sbin/sysctl", "-n", "hw.l1icachesize").Output()
	if err != nil {
		t.Skipf("cannot run sysctl utility: %v", err)
		return
	}
	v, err := strconv.ParseInt(strings.TrimSpace(string(out)), 10, 0)
	if err != nil {
		t.Skipf("sysctl output %q could not be parsed as int: %v", string(out), err)
		return
	}
	if CPU.Cache.L1I != int(v) {
		t.Fatalf("sysctl output %q did not match CPU.Cache.L1I %d", string(out), CPU.Cache.L1I)
	}
}

// Tests that the returned L1 data cache value matches the value returned from a call
// to the OS `sysctl` utility. Skips the test if we can't run it.
func TestDarwinL1DCache(t *testing.T) {
	out, err := exec.Command("/usr/sbin/sysctl", "-n", "hw.l1dcachesize").Output()
	if err != nil {
		t.Skipf("cannot run sysctl utility: %v", err)
		return
	}
	v, err := strconv.ParseInt(strings.TrimSpace(string(out)), 10, 0)
	if err != nil {
		t.Skipf("sysctl output %q could not be parsed as int: %v", string(out), err)
		return
	}
	if CPU.Cache.L1D != int(v) {
		t.Fatalf("sysctl output %q did not match CPU.Cache.L1D %d", string(out), CPU.Cache.L1D)
	}
}
