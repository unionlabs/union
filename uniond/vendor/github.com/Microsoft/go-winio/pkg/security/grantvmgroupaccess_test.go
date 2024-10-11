//go:build windows
// +build windows

package security

import (
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	exec "golang.org/x/sys/execabs"
)

const (
	vmAccountName = `NT VIRTUAL MACHINE\\Virtual Machines`
	vmAccountSID  = "S-1-5-83-0"
)

// TestGrantVmGroupAccess verifies for the three case of a file, a directory,
// and a file in a directory that the appropriate ACEs are set, including
// inheritance in the second two examples. These are the expected ACES. Is
// verified by running icacls and comparing output.
//
// File:
// S-1-15-3-1024-2268835264-3721307629-241982045-173645152-1490879176-104643441-2915960892-1612460704:(R,W)
// S-1-5-83-1-3166535780-1122986932-343720105-43916321:(R,W)
//
// Directory:
// S-1-15-3-1024-2268835264-3721307629-241982045-173645152-1490879176-104643441-2915960892-1612460704:(OI)(CI)(R,W)
// S-1-5-83-1-3166535780-1122986932-343720105-43916321:(OI)(CI)(R,W)
//
// File in directory (inherited):
// S-1-15-3-1024-2268835264-3721307629-241982045-173645152-1490879176-104643441-2915960892-1612460704:(I)(R,W)
// S-1-5-83-1-3166535780-1122986932-343720105-43916321:(I)(R,W)

func TestGrantVmGroupAccess(t *testing.T) {
	f, err := os.CreateTemp("", "gvmgafile")
	if err != nil {
		t.Fatal(err)
	}
	defer func() {
		f.Close()
		os.Remove(f.Name())
	}()

	d := t.TempDir()
	find, err := os.Create(filepath.Join(d, "find.txt"))
	if err != nil {
		t.Fatal(err)
	}
	defer find.Close()

	if err := GrantVmGroupAccess(f.Name()); err != nil {
		t.Fatal(err)
	}

	if err := GrantVmGroupAccess(d); err != nil {
		t.Fatal(err)
	}

	verifyVMAccountDACLs(t,
		f.Name(),
		[]string{`(R)`},
	)

	// Two items here:
	//  - One explicit read only.
	//  - Other applies to this folder, subfolders and files
	//      (OI): object inherit
	//      (CI): container inherit
	//      (IO): inherit only
	//      (GR): generic read
	//
	// In properties for the directory, advanced security settings, this will
	// show as a single line "Allow/Virtual Machines/Read/Inherited from none/This folder, subfolder and files
	verifyVMAccountDACLs(t,
		d,
		[]string{`(R)`, `(OI)(CI)(IO)(GR)`},
	)

	verifyVMAccountDACLs(t,
		find.Name(),
		[]string{`(I)(R)`},
	)
}

func verifyVMAccountDACLs(t *testing.T, name string, permissions []string) {
	cmd := exec.Command("icacls", name)
	outb, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatal(err)
	}
	out := string(outb)

	for _, p := range permissions {
		// Avoid '(' and ')' being part of match groups
		p = strings.Replace(p, "(", "\\(", -1)
		p = strings.Replace(p, ")", "\\)", -1)

		nameToCheck := vmAccountName + ":" + p
		sidToCheck := vmAccountSID + ":" + p

		rxName := regexp.MustCompile(nameToCheck)
		rxSID := regexp.MustCompile(sidToCheck)

		matchesName := rxName.FindAllStringIndex(out, -1)
		matchesSID := rxSID.FindAllStringIndex(out, -1)

		if len(matchesName) != 1 && len(matchesSID) != 1 {
			t.Fatalf("expected one match for %s or %s\n%s", nameToCheck, sidToCheck, out)
		}
	}
}
