package unorderedtx_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/x/auth/ante/unorderedtx"
)

func TestUnorderedTxManager_Close(t *testing.T) {
	txm := unorderedtx.NewManager(t.TempDir())
	txm.Start()

	require.NoError(t, txm.Close())
	require.Panics(t, func() { txm.Close() })
}

func TestUnorderedTxManager_SimpleSize(t *testing.T) {
	txm := unorderedtx.NewManager(t.TempDir())
	defer func() {
		require.NoError(t, txm.Close())
	}()

	txm.Start()

	txm.Add([32]byte{0xFF}, time.Now())
	txm.Add([32]byte{0xAA}, time.Now())
	txm.Add([32]byte{0xCC}, time.Now())

	require.Equal(t, 3, txm.Size())
}

func TestUnorderedTxManager_SimpleContains(t *testing.T) {
	txm := unorderedtx.NewManager(t.TempDir())
	defer func() {
		require.NoError(t, txm.Close())
	}()

	txm.Start()

	for i := 0; i < 10; i++ {
		txHash := [32]byte{byte(i)}
		txm.Add(txHash, time.Now())
		require.True(t, txm.Contains(txHash))
	}

	for i := 10; i < 20; i++ {
		txHash := [32]byte{byte(i)}
		require.False(t, txm.Contains(txHash))
	}
}

func TestUnorderedTxManager_InitEmpty(t *testing.T) {
	txm := unorderedtx.NewManager(t.TempDir())
	defer func() {
		require.NoError(t, txm.Close())
	}()

	txm.Start()

	require.NoError(t, txm.OnInit())
}

func TestUnorderedTxManager_CloseInit(t *testing.T) {
	dataDir := t.TempDir()
	txm := unorderedtx.NewManager(dataDir)
	txm.Start()

	// add a handful of unordered txs
	for i := 0; i < 100; i++ {
		txm.Add([32]byte{byte(i)}, time.Now())
	}

	// close the manager, which should flush all unexpired txs to file
	require.NoError(t, txm.Close())

	// create a new manager, start it
	txm2 := unorderedtx.NewManager(dataDir)
	defer func() {
		require.NoError(t, txm2.Close())
	}()

	// start and execute OnInit, which should load the unexpired txs from file
	txm2.Start()
	require.NoError(t, txm2.OnInit())
	require.Equal(t, 100, txm2.Size())

	for i := 0; i < 100; i++ {
		require.True(t, txm2.Contains([32]byte{byte(i)}))
	}
}

func TestUnorderedTxManager_Flow(t *testing.T) {
	txm := unorderedtx.NewManager(t.TempDir())
	defer func() {
		require.NoError(t, txm.Close())
	}()

	txm.Start()

	currentTime := time.Now()

	// Seed the manager with a txs, some of which should eventually be purged and
	// the others will remain. First 25 Txs should be purged.
	for i := 1; i <= 50; i++ {
		txHash := [32]byte{byte(i)}

		if i <= 25 {
			txm.Add(txHash, currentTime.Add(time.Millisecond*500*time.Duration(i)))
		} else {
			txm.Add(txHash, currentTime.Add(time.Hour))
		}
	}

	// start a goroutine that mimics new blocks being made every 500ms
	doneBlockCh := make(chan bool)
	go func() {
		ticker := time.NewTicker(time.Millisecond * 500)
		defer ticker.Stop()

		for t := range ticker.C {
			txm.OnNewBlock(t)

			if t.After(currentTime.Add(time.Millisecond * 500 * time.Duration(25))) {
				doneBlockCh <- true
				return
			}
		}
	}()

	// Eventually all the txs that are expired should be purged.
	// The remaining txs should remain.
	require.Eventually(
		t,
		func() bool {
			return txm.Size() == 25
		},
		2*time.Minute,
		5*time.Second,
	)

	<-doneBlockCh
}
