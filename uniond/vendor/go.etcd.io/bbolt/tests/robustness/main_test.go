//go:build linux

package robustness

import (
	"flag"
	"os"
	"testing"

	testutils "go.etcd.io/bbolt/tests/utils"
)

func TestMain(m *testing.M) {
	flag.Parse()
	testutils.RequiresRoot()
	os.Exit(m.Run())
}
