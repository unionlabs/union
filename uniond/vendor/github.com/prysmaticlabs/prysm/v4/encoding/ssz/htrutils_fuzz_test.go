//go:build go1.18

package ssz_test

import (
	"testing"

	"github.com/pkg/errors"
	fssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func FuzzUint64Root(f *testing.F) {
	f.Fuzz(func(t *testing.T, i uint64) {
		_ = ssz.Uint64Root(i)
	})
}

func FuzzForkRoot(f *testing.F) {
	frk := &pb.Fork{
		PreviousVersion: params.BeaconConfig().GenesisForkVersion,
		CurrentVersion:  params.BeaconConfig().AltairForkVersion,
		Epoch:           100,
	}
	example, err := frk.MarshalSSZ()
	if err != nil {
		f.Fatal(err)
	}
	f.Add(example)

	f.Fuzz(func(t *testing.T, b []byte) {
		frk := &pb.Fork{}
		if err := frk.UnmarshalSSZ(b); err != nil {
			if errors.Is(err, fssz.ErrSize) {
				return
			}
			t.Fatal(err)
		}

		if _, err := ssz.ForkRoot(frk); err != nil {
			t.Fatal(err)
		}
	})
}

func FuzzPackChunks(f *testing.F) {
	f.Fuzz(func(t *testing.T, b []byte) {
		if _, err := ssz.PackByChunk([][]byte{b}); err != nil {
			t.Fatal(err)
		}
	})
}
