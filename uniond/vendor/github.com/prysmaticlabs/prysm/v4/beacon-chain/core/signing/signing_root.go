package signing

import (
	"sync"

	"github.com/pkg/errors"
	fssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// ForkVersionByteLength length of fork version byte array.
const ForkVersionByteLength = 4

// DomainByteLength length of domain byte array.
const DomainByteLength = 4

// digestMap maps the fork version and genesis validator root to the
// resultant fork digest.
var digestMapLock sync.RWMutex
var digestMap = make(map[string][32]byte)

// ErrSigFailedToVerify returns when a signature of a block object(ie attestation, slashing, exit... etc)
// failed to verify.
var ErrSigFailedToVerify = errors.New("signature did not verify")

// List of descriptions for different kinds of signatures
const (
	// UnknownSignature represents all signatures other than below types
	UnknownSignature string = "unknown signature"
	// BlockSignature represents the block signature from block proposer
	BlockSignature = "block signature"
	// RandaoSignature represents randao specific signature
	RandaoSignature = "randao signature"
	// SelectionProof represents selection proof
	SelectionProof = "selection proof"
	// AggregatorSignature represents aggregator's signature
	AggregatorSignature = "aggregator signature"
	// AttestationSignature represents aggregated attestation signature
	AttestationSignature = "attestation signature"
	// BlsChangeSignature represents signature to BLSToExecutionChange
	BlsChangeSignature = "blschange signature"
	// SyncCommitteeSignature represents sync committee signature
	SyncCommitteeSignature = "sync committee signature"
	// SyncSelectionProof represents sync committee selection proof
	SyncSelectionProof = "sync selection proof"
	// ContributionSignature represents sync committee contributor's signature
	ContributionSignature = "sync committee contribution signature"
	// SyncAggregateSignature represents sync committee aggregator's signature
	SyncAggregateSignature = "sync committee aggregator signature"
)

// ComputeDomainAndSign computes the domain and signing root and sign it using the passed in private key.
func ComputeDomainAndSign(st state.ReadOnlyBeaconState, epoch primitives.Epoch, obj fssz.HashRoot, domain [4]byte, key bls.SecretKey) ([]byte, error) {
	return ComputeDomainAndSignWithoutState(st.Fork(), epoch, domain, st.GenesisValidatorsRoot(), obj, key)
}

// ComputeDomainAndSignWithoutState offers the same functionalit as ComputeDomainAndSign without the need to provide a BeaconState.
// This is particularly helpful for signing values in tests.
func ComputeDomainAndSignWithoutState(fork *ethpb.Fork, epoch primitives.Epoch, domain [4]byte, vr []byte, obj fssz.HashRoot, key bls.SecretKey) ([]byte, error) {
	// EIP-7044: Beginning in Deneb, fix the fork version to Capella for signed exits.
	// This allows for signed validator exits to be valid forever.
	if domain == params.BeaconConfig().DomainVoluntaryExit && epoch >= params.BeaconConfig().DenebForkEpoch {
		fork = &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().CapellaForkVersion,
			CurrentVersion:  params.BeaconConfig().CapellaForkVersion,
			Epoch:           params.BeaconConfig().CapellaForkEpoch,
		}
	}
	d, err := Domain(fork, epoch, domain, vr)
	if err != nil {
		return nil, err
	}
	sr, err := ComputeSigningRoot(obj, d)
	if err != nil {
		return nil, err
	}
	return key.Sign(sr[:]).Marshal(), nil
}

// ComputeSigningRoot computes the root of the object by calculating the hash tree root of the signing data with the given domain.
//
// Spec pseudocode definition:
//
//		def compute_signing_root(ssz_object: SSZObject, domain: Domain) -> Root:
//	   """
//	   Return the signing root for the corresponding signing data.
//	   """
//	   return hash_tree_root(SigningData(
//	       object_root=hash_tree_root(ssz_object),
//	       domain=domain,
//	   ))
func ComputeSigningRoot(object fssz.HashRoot, domain []byte) ([32]byte, error) {
	return Data(object.HashTreeRoot, domain)
}

// Data computes the signing data by utilising the provided root function and then
// returning the signing data of the container object.
func Data(rootFunc func() ([32]byte, error), domain []byte) ([32]byte, error) {
	objRoot, err := rootFunc()
	if err != nil {
		return [32]byte{}, err
	}
	return ComputeSigningRootForRoot(objRoot, domain)
}

// ComputeSigningRootForRoot works the same as ComputeSigningRoot,
// except that gets the root from an argument instead of a callback.
func ComputeSigningRootForRoot(root [32]byte, domain []byte) ([32]byte, error) {
	container := &ethpb.SigningData{
		ObjectRoot: root[:],
		Domain:     domain,
	}
	return container.HashTreeRoot()
}

// ComputeDomainVerifySigningRoot computes domain and verifies signing root of an object given the beacon state, validator index and signature.
func ComputeDomainVerifySigningRoot(st state.ReadOnlyBeaconState, index primitives.ValidatorIndex, epoch primitives.Epoch, obj fssz.HashRoot, domain [4]byte, sig []byte) error {
	v, err := st.ValidatorAtIndex(index)
	if err != nil {
		return err
	}
	d, err := Domain(st.Fork(), epoch, domain, st.GenesisValidatorsRoot())
	if err != nil {
		return err
	}
	return VerifySigningRoot(obj, v.PublicKey, sig, d)
}

// VerifySigningRoot verifies the signing root of an object given its public key, signature and domain.
func VerifySigningRoot(obj fssz.HashRoot, pub, signature, domain []byte) error {
	publicKey, err := bls.PublicKeyFromBytes(pub)
	if err != nil {
		return errors.Wrap(err, "could not convert bytes to public key")
	}
	sig, err := bls.SignatureFromBytes(signature)
	if err != nil {
		return errors.Wrap(err, "could not convert bytes to signature")
	}
	root, err := ComputeSigningRoot(obj, domain)
	if err != nil {
		return errors.Wrap(err, "could not compute signing root")
	}
	if !sig.Verify(publicKey, root[:]) {
		return ErrSigFailedToVerify
	}
	return nil
}

// VerifyBlockHeaderSigningRoot verifies the signing root of a block header given its public key, signature and domain.
func VerifyBlockHeaderSigningRoot(blkHdr *ethpb.BeaconBlockHeader, pub, signature, domain []byte) error {
	publicKey, err := bls.PublicKeyFromBytes(pub)
	if err != nil {
		return errors.Wrap(err, "could not convert bytes to public key")
	}
	sig, err := bls.SignatureFromBytes(signature)
	if err != nil {
		return errors.Wrap(err, "could not convert bytes to signature")
	}
	root, err := Data(blkHdr.HashTreeRoot, domain)
	if err != nil {
		return errors.Wrap(err, "could not compute signing root")
	}
	if !sig.Verify(publicKey, root[:]) {
		return ErrSigFailedToVerify
	}
	return nil
}

// VerifyBlockSigningRoot verifies the signing root of a block given its public key, signature and domain.
func VerifyBlockSigningRoot(pub, signature, domain []byte, rootFunc func() ([32]byte, error)) error {
	set, err := BlockSignatureBatch(pub, signature, domain, rootFunc)
	if err != nil {
		return err
	}
	// We assume only one signature batch is returned here.
	sig := set.Signatures[0]
	publicKey := set.PublicKeys[0]
	root := set.Messages[0]

	rSig, err := bls.SignatureFromBytes(sig)
	if err != nil {
		return err
	}
	if !rSig.Verify(publicKey, root[:]) {
		return ErrSigFailedToVerify
	}
	return nil
}

// BlockSignatureBatch retrieves the relevant signature, message and pubkey data from a block and collating it
// into a signature batch object.
func BlockSignatureBatch(pub, signature, domain []byte, rootFunc func() ([32]byte, error)) (*bls.SignatureBatch, error) {
	publicKey, err := bls.PublicKeyFromBytes(pub)
	if err != nil {
		return nil, errors.Wrap(err, "could not convert bytes to public key")
	}
	// utilize custom block hashing function
	root, err := Data(rootFunc, domain)
	if err != nil {
		return nil, errors.Wrap(err, "could not compute signing root")
	}
	desc := BlockSignature
	return &bls.SignatureBatch{
		Signatures:   [][]byte{signature},
		PublicKeys:   []bls.PublicKey{publicKey},
		Messages:     [][32]byte{root},
		Descriptions: []string{desc},
	}, nil
}

// ComputeDomain returns the domain version for BLS private key to sign and verify with a zeroed 4-byte
// array as the fork version.
//
// def compute_domain(domain_type: DomainType, fork_version: Version=None, genesis_validators_root: Root=None) -> Domain:
//
//	"""
//	Return the domain for the ``domain_type`` and ``fork_version``.
//	"""
//	if fork_version is None:
//	    fork_version = GENESIS_FORK_VERSION
//	if genesis_validators_root is None:
//	    genesis_validators_root = Root()  # all bytes zero by default
//	fork_data_root = compute_fork_data_root(fork_version, genesis_validators_root)
//	return Domain(domain_type + fork_data_root[:28])
func ComputeDomain(domainType [DomainByteLength]byte, forkVersion, genesisValidatorsRoot []byte) ([]byte, error) {
	if forkVersion == nil {
		forkVersion = params.BeaconConfig().GenesisForkVersion
	}
	if genesisValidatorsRoot == nil {
		genesisValidatorsRoot = params.BeaconConfig().ZeroHash[:]
	}
	var forkBytes [ForkVersionByteLength]byte
	copy(forkBytes[:], forkVersion)

	forkDataRoot, err := computeForkDataRoot(forkBytes[:], genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}

	return domain(domainType, forkDataRoot[:]), nil
}

// This returns the bls domain given by the domain type and fork data root.
func domain(domainType [DomainByteLength]byte, forkDataRoot []byte) []byte {
	var b []byte
	b = append(b, domainType[:4]...)
	b = append(b, forkDataRoot[:28]...)
	return b
}

// this returns the 32byte fork data root for the “current_version“ and “genesis_validators_root“.
// This is used primarily in signature domains to avoid collisions across forks/chains.
//
// Spec pseudocode definition:
//
//		def compute_fork_data_root(current_version: Version, genesis_validators_root: Root) -> Root:
//	   """
//	   Return the 32-byte fork data root for the ``current_version`` and ``genesis_validators_root``.
//	   This is used primarily in signature domains to avoid collisions across forks/chains.
//	   """
//	   return hash_tree_root(ForkData(
//	       current_version=current_version,
//	       genesis_validators_root=genesis_validators_root,
//	   ))
func computeForkDataRoot(version, root []byte) ([32]byte, error) {
	digestMapLock.RLock()
	if val, ok := digestMap[string(version)+string(root)]; ok {
		digestMapLock.RUnlock()
		return val, nil
	}
	digestMapLock.RUnlock()
	r, err := (&ethpb.ForkData{
		CurrentVersion:        version,
		GenesisValidatorsRoot: root,
	}).HashTreeRoot()
	if err != nil {
		return [32]byte{}, err
	}
	// Cache result of digest computation
	// as this is a hot path and doesn't need
	// to be constantly computed.
	digestMapLock.Lock()
	digestMap[string(version)+string(root)] = r
	digestMapLock.Unlock()
	return r, nil
}

// ComputeForkDigest returns the fork for the current version and genesis validators root
//
// Spec pseudocode definition:
//
//		def compute_fork_digest(current_version: Version, genesis_validators_root: Root) -> ForkDigest:
//	   """
//	   Return the 4-byte fork digest for the ``current_version`` and ``genesis_validators_root``.
//	   This is a digest primarily used for domain separation on the p2p layer.
//	   4-bytes suffices for practical separation of forks/chains.
//	   """
//	   return ForkDigest(compute_fork_data_root(current_version, genesis_validators_root)[:4])
func ComputeForkDigest(version, genesisValidatorsRoot []byte) ([4]byte, error) {
	dataRoot, err := computeForkDataRoot(version, genesisValidatorsRoot)
	if err != nil {
		return [4]byte{}, err
	}
	return bytesutil.ToBytes4(dataRoot[:]), nil
}
