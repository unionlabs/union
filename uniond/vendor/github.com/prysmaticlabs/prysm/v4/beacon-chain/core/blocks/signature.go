package blocks

import (
	"context"
	"encoding/binary"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// retrieves the signature batch from the raw data, public key,signature and domain provided.
func signatureBatch(signedData, pub, signature, domain []byte, desc string) (*bls.SignatureBatch, error) {
	publicKey, err := bls.PublicKeyFromBytes(pub)
	if err != nil {
		return nil, errors.Wrap(err, "could not convert bytes to public key")
	}
	signingData := &ethpb.SigningData{
		ObjectRoot: signedData,
		Domain:     domain,
	}
	root, err := signingData.HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "could not hash container")
	}
	return &bls.SignatureBatch{
		Signatures:   [][]byte{signature},
		PublicKeys:   []bls.PublicKey{publicKey},
		Messages:     [][32]byte{root},
		Descriptions: []string{desc},
	}, nil
}

// verifies the signature from the raw data, public key and domain provided.
func verifySignature(signedData, pub, signature, domain []byte) error {
	set, err := signatureBatch(signedData, pub, signature, domain, signing.UnknownSignature)
	if err != nil {
		return err
	}
	if len(set.Signatures) != 1 {
		return errors.Errorf("signature set contains %d signatures instead of 1", len(set.Signatures))
	}
	// We assume only one signature set is returned here.
	sig := set.Signatures[0]
	publicKey := set.PublicKeys[0]
	root := set.Messages[0]
	rSig, err := bls.SignatureFromBytes(sig)
	if err != nil {
		return err
	}
	if !rSig.Verify(publicKey, root[:]) {
		return signing.ErrSigFailedToVerify
	}
	return nil
}

// VerifyBlockSignature verifies the proposer signature of a beacon block.
func VerifyBlockSignature(beaconState state.ReadOnlyBeaconState,
	proposerIndex primitives.ValidatorIndex,
	sig []byte,
	rootFunc func() ([32]byte, error)) error {
	currentEpoch := slots.ToEpoch(beaconState.Slot())
	domain, err := signing.Domain(beaconState.Fork(), currentEpoch, params.BeaconConfig().DomainBeaconProposer, beaconState.GenesisValidatorsRoot())
	if err != nil {
		return err
	}
	proposer, err := beaconState.ValidatorAtIndex(proposerIndex)
	if err != nil {
		return err
	}
	proposerPubKey := proposer.PublicKey
	return signing.VerifyBlockSigningRoot(proposerPubKey, sig, domain, rootFunc)
}

// VerifyBlockHeaderSignature verifies the proposer signature of a beacon block header.
func VerifyBlockHeaderSignature(beaconState state.BeaconState, header *ethpb.SignedBeaconBlockHeader) error {
	currentEpoch := slots.ToEpoch(beaconState.Slot())
	domain, err := signing.Domain(beaconState.Fork(), currentEpoch, params.BeaconConfig().DomainBeaconProposer, beaconState.GenesisValidatorsRoot())
	if err != nil {
		return err
	}
	proposer, err := beaconState.ValidatorAtIndex(header.Header.ProposerIndex)
	if err != nil {
		return err
	}
	proposerPubKey := proposer.PublicKey
	return signing.VerifyBlockHeaderSigningRoot(header.Header, proposerPubKey, header.Signature, domain)
}

// VerifyBlockSignatureUsingCurrentFork verifies the proposer signature of a beacon block. This differs
// from the above method by not using fork data from the state and instead retrieving it
// via the respective epoch.
func VerifyBlockSignatureUsingCurrentFork(beaconState state.ReadOnlyBeaconState, blk interfaces.ReadOnlySignedBeaconBlock) error {
	currentEpoch := slots.ToEpoch(blk.Block().Slot())
	fork, err := forks.Fork(currentEpoch)
	if err != nil {
		return err
	}
	domain, err := signing.Domain(fork, currentEpoch, params.BeaconConfig().DomainBeaconProposer, beaconState.GenesisValidatorsRoot())
	if err != nil {
		return err
	}
	proposer, err := beaconState.ValidatorAtIndex(blk.Block().ProposerIndex())
	if err != nil {
		return err
	}
	proposerPubKey := proposer.PublicKey
	sig := blk.Signature()
	return signing.VerifyBlockSigningRoot(proposerPubKey, sig[:], domain, blk.Block().HashTreeRoot)
}

// BlockSignatureBatch retrieves the block signature batch from the provided block and its corresponding state.
func BlockSignatureBatch(beaconState state.ReadOnlyBeaconState,
	proposerIndex primitives.ValidatorIndex,
	sig []byte,
	rootFunc func() ([32]byte, error)) (*bls.SignatureBatch, error) {
	currentEpoch := slots.ToEpoch(beaconState.Slot())
	domain, err := signing.Domain(beaconState.Fork(), currentEpoch, params.BeaconConfig().DomainBeaconProposer, beaconState.GenesisValidatorsRoot())
	if err != nil {
		return nil, err
	}
	proposer, err := beaconState.ValidatorAtIndex(proposerIndex)
	if err != nil {
		return nil, err
	}
	proposerPubKey := proposer.PublicKey
	return signing.BlockSignatureBatch(proposerPubKey, sig, domain, rootFunc)
}

// RandaoSignatureBatch retrieves the relevant randao specific signature batch object
// from a block and its corresponding state.
func RandaoSignatureBatch(
	ctx context.Context,
	beaconState state.ReadOnlyBeaconState,
	reveal []byte,
) (*bls.SignatureBatch, error) {
	buf, proposerPub, domain, err := randaoSigningData(ctx, beaconState)
	if err != nil {
		return nil, err
	}
	set, err := signatureBatch(buf, proposerPub, reveal, domain, signing.RandaoSignature)
	if err != nil {
		return nil, err
	}
	return set, nil
}

// retrieves the randao related signing data from the state.
func randaoSigningData(ctx context.Context, beaconState state.ReadOnlyBeaconState) ([]byte, []byte, []byte, error) {
	proposerIdx, err := helpers.BeaconProposerIndex(ctx, beaconState)
	if err != nil {
		return nil, nil, nil, errors.Wrap(err, "could not get beacon proposer index")
	}
	proposerPub := beaconState.PubkeyAtIndex(proposerIdx)

	currentEpoch := slots.ToEpoch(beaconState.Slot())
	buf := make([]byte, 32)
	binary.LittleEndian.PutUint64(buf, uint64(currentEpoch))

	domain, err := signing.Domain(beaconState.Fork(), currentEpoch, params.BeaconConfig().DomainRandao, beaconState.GenesisValidatorsRoot())
	if err != nil {
		return nil, nil, nil, err
	}
	return buf, proposerPub[:], domain, nil
}

// Method to break down attestations of the same domain and collect them into a single signature batch.
func createAttestationSignatureBatch(
	ctx context.Context,
	beaconState state.ReadOnlyBeaconState,
	atts []*ethpb.Attestation,
	domain []byte,
) (*bls.SignatureBatch, error) {
	if len(atts) == 0 {
		return nil, nil
	}

	sigs := make([][]byte, len(atts))
	pks := make([]bls.PublicKey, len(atts))
	msgs := make([][32]byte, len(atts))
	descs := make([]string, len(atts))
	for i, a := range atts {
		sigs[i] = a.Signature
		c, err := helpers.BeaconCommitteeFromState(ctx, beaconState, a.Data.Slot, a.Data.CommitteeIndex)
		if err != nil {
			return nil, err
		}
		ia, err := attestation.ConvertToIndexed(ctx, a, c)
		if err != nil {
			return nil, err
		}
		if err := attestation.IsValidAttestationIndices(ctx, ia); err != nil {
			return nil, err
		}
		indices := ia.AttestingIndices
		pubkeys := make([][]byte, len(indices))
		for i := 0; i < len(indices); i++ {
			pubkeyAtIdx := beaconState.PubkeyAtIndex(primitives.ValidatorIndex(indices[i]))
			pubkeys[i] = pubkeyAtIdx[:]
		}
		aggP, err := bls.AggregatePublicKeys(pubkeys)
		if err != nil {
			return nil, err
		}
		pks[i] = aggP

		root, err := signing.ComputeSigningRoot(ia.Data, domain)
		if err != nil {
			return nil, errors.Wrap(err, "could not get signing root of object")
		}
		msgs[i] = root

		descs[i] = signing.AttestationSignature
	}
	return &bls.SignatureBatch{
		Signatures:   sigs,
		PublicKeys:   pks,
		Messages:     msgs,
		Descriptions: descs,
	}, nil
}

// AttestationSignatureBatch retrieves all the related attestation signature data such as the relevant public keys,
// signatures and attestation signing data and collate it into a signature batch object.
func AttestationSignatureBatch(ctx context.Context, beaconState state.ReadOnlyBeaconState, atts []*ethpb.Attestation) (*bls.SignatureBatch, error) {
	if len(atts) == 0 {
		return bls.NewSet(), nil
	}

	fork := beaconState.Fork()
	gvr := beaconState.GenesisValidatorsRoot()
	dt := params.BeaconConfig().DomainBeaconAttester

	// Split attestations by fork. Note: the signature domain will differ based on the fork.
	var preForkAtts []*ethpb.Attestation
	var postForkAtts []*ethpb.Attestation
	for _, a := range atts {
		if slots.ToEpoch(a.Data.Slot) < fork.Epoch {
			preForkAtts = append(preForkAtts, a)
		} else {
			postForkAtts = append(postForkAtts, a)
		}
	}
	set := bls.NewSet()

	// Check attestations from before the fork.
	if fork.Epoch > 0 && len(preForkAtts) > 0 { // Check to prevent underflow and there is valid attestations to create sig batch.
		prevDomain, err := signing.Domain(fork, fork.Epoch-1, dt, gvr)
		if err != nil {
			return nil, err
		}
		aSet, err := createAttestationSignatureBatch(ctx, beaconState, preForkAtts, prevDomain)
		if err != nil {
			return nil, err
		}
		if aSet != nil {
			set.Join(aSet)
		}
	} else if len(preForkAtts) > 0 {
		// This is a sanity check that preForkAtts were not ignored when fork.Epoch == 0. This
		// condition is not possible, but it doesn't hurt to check anyway.
		return nil, errors.New("some attestations were not verified from previous fork before genesis")
	}

	if len(postForkAtts) > 0 {
		// Then check attestations from after the fork.
		currDomain, err := signing.Domain(fork, fork.Epoch, dt, gvr)
		if err != nil {
			return nil, err
		}

		aSet, err := createAttestationSignatureBatch(ctx, beaconState, postForkAtts, currDomain)
		if err != nil {
			return nil, err
		}
		if aSet != nil {
			return set.Join(aSet), nil
		}
	}

	return set, nil
}
