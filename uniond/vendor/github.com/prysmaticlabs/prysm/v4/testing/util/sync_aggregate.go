package util

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	p2pType "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func generateSyncAggregate(st state.BeaconState, privs []bls.SecretKey, parentRoot [32]byte) (*ethpb.SyncAggregate, error) {
	nextSlotEpoch := slots.ToEpoch(st.Slot() + 1)
	currEpoch := slots.ToEpoch(st.Slot())

	var syncCommittee *ethpb.SyncCommittee
	var err error
	if slots.SyncCommitteePeriod(currEpoch) == slots.SyncCommitteePeriod(nextSlotEpoch) {
		syncCommittee, err = st.CurrentSyncCommittee()
		if err != nil {
			return nil, err
		}
	} else {
		syncCommittee, err = st.NextSyncCommittee()
		if err != nil {
			return nil, err
		}
	}
	sigs := make([]bls.Signature, 0, len(syncCommittee.Pubkeys))
	var bVector []byte
	currSize := new(ethpb.SyncAggregate).SyncCommitteeBits.Len()
	switch currSize {
	case 512:
		bVector = bitfield.NewBitvector512()
	case 32:
		bVector = bitfield.NewBitvector32()
	default:
		return nil, errors.New("invalid bit vector size")
	}

	for i, p := range syncCommittee.Pubkeys {
		idx, ok := st.ValidatorIndexByPubkey(bytesutil.ToBytes48(p))
		if !ok {
			continue
		}
		d, err := signing.Domain(st.Fork(), slots.ToEpoch(st.Slot()), params.BeaconConfig().DomainSyncCommittee, st.GenesisValidatorsRoot())
		if err != nil {
			return nil, err
		}
		sszBytes := p2pType.SSZBytes(parentRoot[:])
		r, err := signing.ComputeSigningRoot(&sszBytes, d)
		if err != nil {
			return nil, err
		}
		sigs = append(sigs, privs[idx].Sign(r[:]))
		if currSize == 512 {
			bitfield.Bitvector512(bVector).SetBitAt(uint64(i), true)
		}
		if currSize == 32 {
			bitfield.Bitvector32(bVector).SetBitAt(uint64(i), true)
		}
	}
	if len(sigs) == 0 {
		fakeSig := [96]byte{0xC0}
		return &ethpb.SyncAggregate{SyncCommitteeSignature: fakeSig[:], SyncCommitteeBits: bVector}, nil
	}
	aggSig := bls.AggregateSignatures(sigs)
	return &ethpb.SyncAggregate{SyncCommitteeSignature: aggSig.Marshal(), SyncCommitteeBits: bVector}, nil
}
