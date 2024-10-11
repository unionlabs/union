// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package tests

import (
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
)

func TestBlockchain(t *testing.T) {
	bt := new(testMatcher)
	// General state tests are 'exported' as blockchain tests, but we can run them natively.
	// For speedier CI-runs, the line below can be uncommented, so those are skipped.
	// For now, in hardfork-times (Berlin), we run the tests both as StateTests and
	// as blockchain tests, since the latter also covers things like receipt root
	bt.skipLoad(`^GeneralStateTests/`)

	// Skip random failures due to selfish mining test
	bt.skipLoad(`.*bcForgedTest/bcForkUncle\.json`)

	// Slow tests
	bt.slow(`.*bcExploitTest/DelegateCallSpam.json`)
	bt.slow(`.*bcExploitTest/ShanghaiLove.json`)
	bt.slow(`.*bcExploitTest/SuicideIssue.json`)
	bt.slow(`.*/bcForkStressTest/`)
	bt.slow(`.*/bcGasPricerTest/RPC_API_Test.json`)
	bt.slow(`.*/bcWalletTest/`)

	// Very slow test
	bt.skipLoad(`.*/stTimeConsuming/.*`)
	// test takes a lot for time and goes easily OOM because of sha3 calculation on a huge range,
	// using 4.6 TGas
	bt.skipLoad(`.*randomStatetest94.json.*`)

	bt.walk(t, blockTestDir, func(t *testing.T, name string, test *BlockTest) {
		execBlockTest(t, bt, test)
	})
	// There is also a LegacyTests folder, containing blockchain tests generated
	// prior to Istanbul. However, they are all derived from GeneralStateTests,
	// which run natively, so there's no reason to run them here.
}

// TestExecutionSpec runs the test fixtures from execution-spec-tests.
func TestExecutionSpec(t *testing.T) {
	if !common.FileExist(executionSpecDir) {
		t.Skipf("directory %s does not exist", executionSpecDir)
	}
	bt := new(testMatcher)

	bt.walk(t, executionSpecDir, func(t *testing.T, name string, test *BlockTest) {
		execBlockTest(t, bt, test)
	})
}

func execBlockTest(t *testing.T, bt *testMatcher, test *BlockTest) {
	if err := bt.checkFailure(t, test.Run(false, rawdb.HashScheme, nil)); err != nil {
		t.Errorf("test in hash mode without snapshotter failed: %v", err)
		return
	}
	if err := bt.checkFailure(t, test.Run(true, rawdb.HashScheme, nil)); err != nil {
		t.Errorf("test in hash mode with snapshotter failed: %v", err)
		return
	}
	if err := bt.checkFailure(t, test.Run(false, rawdb.PathScheme, nil)); err != nil {
		t.Errorf("test in path mode without snapshotter failed: %v", err)
		return
	}
	if err := bt.checkFailure(t, test.Run(true, rawdb.PathScheme, nil)); err != nil {
		t.Errorf("test in path mode with snapshotter failed: %v", err)
		return
	}
}
