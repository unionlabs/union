package blockchain

import (
	"testing"

	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
)

func testServiceOptsWithDB(t *testing.T) []Option {
	beaconDB := testDB.SetupDB(t)
	fcs := doublylinkedtree.New()
	cs := startup.NewClockSynchronizer()
	return []Option{
		WithDatabase(beaconDB),
		WithStateGen(stategen.New(beaconDB, fcs)),
		WithForkChoiceStore(fcs),
		WithClockSynchronizer(cs),
	}
}

// WARNING: only use these opts when you are certain there are no db calls
// in your code path. this is a lightweight way to satisfy the stategen/beacondb
// initialization requirements w/o the overhead of db init.
func testServiceOptsNoDB() []Option {
	cs := startup.NewClockSynchronizer()
	return []Option{WithClockSynchronizer(cs)}
}
