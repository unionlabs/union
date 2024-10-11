package sync

import (
	"io"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/sirupsen/logrus"
)

func TestMain(m *testing.M) {
	logrus.SetLevel(logrus.DebugLevel)
	logrus.SetOutput(io.Discard)

	resetFlags := flags.Get()
	flags.Init(&flags.GlobalFlags{
		BlockBatchLimit:            64,
		BlockBatchLimitBurstFactor: 10,
		BlobBatchLimit:             8,
		BlobBatchLimitBurstFactor:  2,
	})
	defer func() {
		flags.Init(resetFlags)
	}()
	m.Run()
}
