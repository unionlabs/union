package blstoexec

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestPendingBLSToExecChanges(t *testing.T) {
	t.Run("empty pool", func(t *testing.T) {
		pool := NewPool()
		changes, err := pool.PendingBLSToExecChanges()
		require.NoError(t, err)
		assert.Equal(t, 0, len(changes))
	})
	t.Run("non-empty pool", func(t *testing.T) {
		pool := NewPool()
		pool.InsertBLSToExecChange(&eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: 0,
			},
		})
		pool.InsertBLSToExecChange(&eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: 1,
			},
		})
		changes, err := pool.PendingBLSToExecChanges()
		require.NoError(t, err)
		assert.Equal(t, 2, len(changes))
	})
}

func TestBLSToExecChangesForInclusion(t *testing.T) {
	spb := &eth.BeaconStateCapella{
		Fork: &eth.Fork{
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
		},
	}
	numValidators := 2 * params.BeaconConfig().MaxBlsToExecutionChanges
	validators := make([]*eth.Validator, numValidators)
	blsChanges := make([]*eth.BLSToExecutionChange, numValidators)
	spb.Balances = make([]uint64, numValidators)
	privKeys := make([]common.SecretKey, numValidators)
	maxEffectiveBalance := params.BeaconConfig().MaxEffectiveBalance
	executionAddress := []byte{0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13}

	for i := range validators {
		v := &eth.Validator{}
		v.EffectiveBalance = maxEffectiveBalance
		v.WithdrawableEpoch = params.BeaconConfig().FarFutureEpoch
		v.WithdrawalCredentials = make([]byte, 32)
		priv, err := bls.RandKey()
		require.NoError(t, err)
		privKeys[i] = priv
		pubkey := priv.PublicKey().Marshal()

		message := &eth.BLSToExecutionChange{
			ToExecutionAddress: executionAddress,
			ValidatorIndex:     primitives.ValidatorIndex(i),
			FromBlsPubkey:      pubkey,
		}

		hashFn := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
		digest := hashFn.Hash(pubkey)
		digest[0] = params.BeaconConfig().BLSWithdrawalPrefixByte
		copy(v.WithdrawalCredentials, digest[:])
		validators[i] = v
		blsChanges[i] = message
	}
	spb.Validators = validators
	st, err := state_native.InitializeFromProtoCapella(spb)
	require.NoError(t, err)

	signedChanges := make([]*eth.SignedBLSToExecutionChange, numValidators)
	for i, message := range blsChanges {
		signature, err := signing.ComputeDomainAndSign(st, time.CurrentEpoch(st), message, params.BeaconConfig().DomainBLSToExecutionChange, privKeys[i])
		require.NoError(t, err)

		signed := &eth.SignedBLSToExecutionChange{
			Message:   message,
			Signature: signature,
		}
		signedChanges[i] = signed
	}

	t.Run("empty pool", func(t *testing.T) {
		pool := NewPool()
		changes, err := pool.BLSToExecChangesForInclusion(st)
		require.NoError(t, err)
		assert.Equal(t, 0, len(changes))
	})
	t.Run("Less than MaxBlsToExecutionChanges in pool", func(t *testing.T) {
		pool := NewPool()
		for i := uint64(0); i < params.BeaconConfig().MaxBlsToExecutionChanges-1; i++ {
			pool.InsertBLSToExecChange(signedChanges[i])
		}
		changes, err := pool.BLSToExecChangesForInclusion(st)
		require.NoError(t, err)
		assert.Equal(t, int(params.BeaconConfig().MaxBlsToExecutionChanges)-1, len(changes))
	})
	t.Run("MaxBlsToExecutionChanges in pool", func(t *testing.T) {
		pool := NewPool()
		for i := uint64(0); i < params.BeaconConfig().MaxBlsToExecutionChanges; i++ {
			pool.InsertBLSToExecChange(signedChanges[i])
		}
		changes, err := pool.BLSToExecChangesForInclusion(st)
		require.NoError(t, err)
		assert.Equal(t, int(params.BeaconConfig().MaxBlsToExecutionChanges), len(changes))
	})
	t.Run("more than MaxBlsToExecutionChanges in pool", func(t *testing.T) {
		pool := NewPool()
		for i := uint64(0); i < numValidators; i++ {
			pool.InsertBLSToExecChange(signedChanges[i])
		}
		changes, err := pool.BLSToExecChangesForInclusion(st)
		require.NoError(t, err)
		// We want FIFO semantics, which means validator with index 16 shouldn't be returned
		assert.Equal(t, int(params.BeaconConfig().MaxBlsToExecutionChanges), len(changes))
		for _, ch := range changes {
			assert.NotEqual(t, primitives.ValidatorIndex(15), ch.Message.ValidatorIndex)
		}
	})
	t.Run("One Bad change", func(t *testing.T) {
		pool := NewPool()
		saveByte := signedChanges[1].Message.FromBlsPubkey[5]
		signedChanges[1].Message.FromBlsPubkey[5] = 0xff
		for i := uint64(0); i < numValidators; i++ {
			pool.InsertBLSToExecChange(signedChanges[i])
		}
		changes, err := pool.BLSToExecChangesForInclusion(st)
		require.NoError(t, err)
		assert.Equal(t, int(params.BeaconConfig().MaxBlsToExecutionChanges), len(changes))
		assert.Equal(t, primitives.ValidatorIndex(30), changes[1].Message.ValidatorIndex)
		signedChanges[1].Message.FromBlsPubkey[5] = saveByte
	})
	t.Run("One Bad Signature", func(t *testing.T) {
		pool := NewPool()
		copy(signedChanges[30].Signature, signedChanges[31].Signature)
		for i := uint64(0); i < numValidators; i++ {
			pool.InsertBLSToExecChange(signedChanges[i])
		}
		changes, err := pool.BLSToExecChangesForInclusion(st)
		require.NoError(t, err)
		assert.Equal(t, int(params.BeaconConfig().MaxBlsToExecutionChanges), len(changes))
		assert.Equal(t, primitives.ValidatorIndex(30), changes[1].Message.ValidatorIndex)
	})
	t.Run("invalid change not returned", func(t *testing.T) {
		pool := NewPool()
		saveByte := signedChanges[1].Message.FromBlsPubkey[5]
		signedChanges[1].Message.FromBlsPubkey[5] = 0xff
		pool.InsertBLSToExecChange(signedChanges[1])
		changes, err := pool.BLSToExecChangesForInclusion(st)
		require.NoError(t, err)
		assert.Equal(t, 0, len(changes))
		signedChanges[1].Message.FromBlsPubkey[5] = saveByte
	})
}

func TestInsertBLSToExecChange(t *testing.T) {
	t.Run("empty pool", func(t *testing.T) {
		pool := NewPool()
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			},
		}
		pool.InsertBLSToExecChange(change)
		require.Equal(t, 1, pool.pending.Len())
		require.Equal(t, 1, len(pool.m))
		n, ok := pool.m[0]
		require.Equal(t, true, ok)
		v, err := n.Value()
		require.NoError(t, err)
		assert.DeepEqual(t, change, v)
	})
	t.Run("item in pool", func(t *testing.T) {
		pool := NewPool()
		old := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			},
		}
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(1),
			},
		}
		pool.InsertBLSToExecChange(old)
		pool.InsertBLSToExecChange(change)
		require.Equal(t, 2, pool.pending.Len())
		require.Equal(t, 2, len(pool.m))
		n, ok := pool.m[0]
		require.Equal(t, true, ok)
		v, err := n.Value()
		require.NoError(t, err)
		assert.DeepEqual(t, old, v)
		n, ok = pool.m[1]
		require.Equal(t, true, ok)
		v, err = n.Value()
		require.NoError(t, err)
		assert.DeepEqual(t, change, v)
	})
	t.Run("validator index already exists", func(t *testing.T) {
		pool := NewPool()
		old := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			},
			Signature: []byte("old"),
		}
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			},
			Signature: []byte("change"),
		}
		pool.InsertBLSToExecChange(old)
		pool.InsertBLSToExecChange(change)
		assert.Equal(t, 1, pool.pending.Len())
		require.Equal(t, 1, len(pool.m))
		n, ok := pool.m[0]
		require.Equal(t, true, ok)
		v, err := n.Value()
		require.NoError(t, err)
		assert.DeepEqual(t, old, v)
	})
}

func TestMarkIncluded(t *testing.T) {
	t.Run("one element in pool", func(t *testing.T) {
		pool := NewPool()
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		pool.InsertBLSToExecChange(change)
		pool.MarkIncluded(change)
		assert.Equal(t, 0, pool.pending.Len())
		_, ok := pool.m[0]
		assert.Equal(t, false, ok)
	})
	t.Run("first of multiple elements", func(t *testing.T) {
		pool := NewPool()
		first := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		second := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(1),
			}}
		third := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(2),
			}}
		pool.InsertBLSToExecChange(first)
		pool.InsertBLSToExecChange(second)
		pool.InsertBLSToExecChange(third)
		pool.MarkIncluded(first)
		require.Equal(t, 2, pool.pending.Len())
		_, ok := pool.m[0]
		assert.Equal(t, false, ok)
	})
	t.Run("last of multiple elements", func(t *testing.T) {
		pool := NewPool()
		first := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		second := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(1),
			}}
		third := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(2),
			}}
		pool.InsertBLSToExecChange(first)
		pool.InsertBLSToExecChange(second)
		pool.InsertBLSToExecChange(third)
		pool.MarkIncluded(third)
		require.Equal(t, 2, pool.pending.Len())
		_, ok := pool.m[2]
		assert.Equal(t, false, ok)
	})
	t.Run("in the middle of multiple elements", func(t *testing.T) {
		pool := NewPool()
		first := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		second := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(1),
			}}
		third := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(2),
			}}
		pool.InsertBLSToExecChange(first)
		pool.InsertBLSToExecChange(second)
		pool.InsertBLSToExecChange(third)
		pool.MarkIncluded(second)
		require.Equal(t, 2, pool.pending.Len())
		_, ok := pool.m[1]
		assert.Equal(t, false, ok)
	})
	t.Run("not in pool", func(t *testing.T) {
		pool := NewPool()
		first := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		second := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(1),
			}}
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(2),
			}}
		pool.InsertBLSToExecChange(first)
		pool.InsertBLSToExecChange(second)
		pool.MarkIncluded(change)
		require.Equal(t, 2, pool.pending.Len())
		_, ok := pool.m[0]
		require.Equal(t, true, ok)
		assert.NotNil(t, pool.m[0])
		_, ok = pool.m[1]
		require.Equal(t, true, ok)
		assert.NotNil(t, pool.m[1])
	})
}

func TestValidatorExists(t *testing.T) {
	t.Run("no validators in pool", func(t *testing.T) {
		pool := NewPool()
		assert.Equal(t, false, pool.ValidatorExists(0))
	})
	t.Run("validator added to pool", func(t *testing.T) {
		pool := NewPool()
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		pool.InsertBLSToExecChange(change)
		assert.Equal(t, true, pool.ValidatorExists(0))
	})
	t.Run("multiple validators added to pool", func(t *testing.T) {
		pool := NewPool()
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		pool.InsertBLSToExecChange(change)
		change = &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(10),
			}}
		pool.InsertBLSToExecChange(change)
		change = &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(30),
			}}
		pool.InsertBLSToExecChange(change)

		assert.Equal(t, true, pool.ValidatorExists(0))
		assert.Equal(t, true, pool.ValidatorExists(10))
		assert.Equal(t, true, pool.ValidatorExists(30))
	})
	t.Run("validator added and then removed", func(t *testing.T) {
		pool := NewPool()
		change := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		pool.InsertBLSToExecChange(change)
		pool.MarkIncluded(change)
		assert.Equal(t, false, pool.ValidatorExists(0))
	})
	t.Run("multiple validators added to pool and removed", func(t *testing.T) {
		pool := NewPool()
		firstChange := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(0),
			}}
		pool.InsertBLSToExecChange(firstChange)
		secondChange := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(10),
			}}
		pool.InsertBLSToExecChange(secondChange)
		thirdChange := &eth.SignedBLSToExecutionChange{
			Message: &eth.BLSToExecutionChange{
				ValidatorIndex: primitives.ValidatorIndex(30),
			}}
		pool.InsertBLSToExecChange(thirdChange)

		pool.MarkIncluded(firstChange)
		pool.MarkIncluded(thirdChange)

		assert.Equal(t, false, pool.ValidatorExists(0))
		assert.Equal(t, true, pool.ValidatorExists(10))
		assert.Equal(t, false, pool.ValidatorExists(30))
	})
}

func TestPoolCycleMap(t *testing.T) {
	pool := NewPool()
	firstChange := &eth.SignedBLSToExecutionChange{
		Message: &eth.BLSToExecutionChange{
			ValidatorIndex: primitives.ValidatorIndex(0),
		}}
	pool.InsertBLSToExecChange(firstChange)
	secondChange := &eth.SignedBLSToExecutionChange{
		Message: &eth.BLSToExecutionChange{
			ValidatorIndex: primitives.ValidatorIndex(10),
		}}
	pool.InsertBLSToExecChange(secondChange)
	thirdChange := &eth.SignedBLSToExecutionChange{
		Message: &eth.BLSToExecutionChange{
			ValidatorIndex: primitives.ValidatorIndex(30),
		}}
	pool.InsertBLSToExecChange(thirdChange)

	pool.cycleMap()
	require.Equal(t, true, pool.ValidatorExists(0))
	require.Equal(t, true, pool.ValidatorExists(10))
	require.Equal(t, true, pool.ValidatorExists(30))
	require.Equal(t, false, pool.ValidatorExists(20))

}
