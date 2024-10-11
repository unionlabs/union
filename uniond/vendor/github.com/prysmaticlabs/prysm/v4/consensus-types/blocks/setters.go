package blocks

import (
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

// SetSignature sets the signature of the signed beacon block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetSignature(sig []byte) {
	copy(b.signature[:], sig)
}

// SetSlot sets the respective slot of the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetSlot(slot primitives.Slot) {
	b.block.slot = slot
}

// SetProposerIndex sets the proposer index of the beacon block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetProposerIndex(proposerIndex primitives.ValidatorIndex) {
	b.block.proposerIndex = proposerIndex
}

// SetParentRoot sets the parent root of beacon block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetParentRoot(parentRoot []byte) {
	copy(b.block.parentRoot[:], parentRoot)
}

// SetStateRoot sets the state root of the underlying beacon block
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetStateRoot(root []byte) {
	copy(b.block.stateRoot[:], root)
}

// SetRandaoReveal sets the randao reveal in the block body.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetRandaoReveal(r []byte) {
	copy(b.block.body.randaoReveal[:], r)
}

// SetGraffiti sets the graffiti in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetGraffiti(g []byte) {
	copy(b.block.body.graffiti[:], g)
}

// SetEth1Data sets the eth1 data in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetEth1Data(e *eth.Eth1Data) {
	b.block.body.eth1Data = e
}

// SetProposerSlashings sets the proposer slashings in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetProposerSlashings(p []*eth.ProposerSlashing) {
	b.block.body.proposerSlashings = p
}

// SetAttesterSlashings sets the attester slashings in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetAttesterSlashings(a []*eth.AttesterSlashing) {
	b.block.body.attesterSlashings = a
}

// SetAttestations sets the attestations in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetAttestations(a []*eth.Attestation) {
	b.block.body.attestations = a
}

// SetDeposits sets the deposits in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetDeposits(d []*eth.Deposit) {
	b.block.body.deposits = d
}

// SetVoluntaryExits sets the voluntary exits in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetVoluntaryExits(v []*eth.SignedVoluntaryExit) {
	b.block.body.voluntaryExits = v
}

// SetSyncAggregate sets the sync aggregate in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetSyncAggregate(s *eth.SyncAggregate) error {
	if b.version == version.Phase0 {
		return consensus_types.ErrNotSupported("SyncAggregate", b.version)
	}
	b.block.body.syncAggregate = s
	return nil
}

// SetExecution sets the execution payload of the block body.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetExecution(e interfaces.ExecutionData) error {
	if b.version == version.Phase0 || b.version == version.Altair {
		return consensus_types.ErrNotSupported("Execution", b.version)
	}
	if e.IsBlinded() {
		b.block.body.executionPayloadHeader = e
		return nil
	}
	b.block.body.executionPayload = e
	return nil
}

// SetBLSToExecutionChanges sets the BLS to execution changes in the block.
// This function is not thread safe, it is only used during block creation.
func (b *SignedBeaconBlock) SetBLSToExecutionChanges(blsToExecutionChanges []*eth.SignedBLSToExecutionChange) error {
	if b.version < version.Capella {
		return consensus_types.ErrNotSupported("BLSToExecutionChanges", b.version)
	}
	b.block.body.blsToExecutionChanges = blsToExecutionChanges
	return nil
}

// SetBlobKzgCommitments sets the blob kzg commitments in the block.
func (b *SignedBeaconBlock) SetBlobKzgCommitments(c [][]byte) error {
	switch b.version {
	case version.Phase0, version.Altair, version.Bellatrix, version.Capella:
		return consensus_types.ErrNotSupported("SetBlobKzgCommitments", b.version)
	case version.Deneb:
		b.block.body.blobKzgCommitments = c
		return nil
	default:
		return errIncorrectBlockVersion
	}
}
