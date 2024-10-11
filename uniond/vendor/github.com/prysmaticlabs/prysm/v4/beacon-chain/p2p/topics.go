package p2p

const (
	// GossipProtocolAndDigest represents the protocol and fork digest prefix in a gossip topic.
	GossipProtocolAndDigest = "/eth2/%x/"

	// Message Types
	//
	// GossipAttestationMessage is the name for the attestation message type. It is
	// specially extracted so as to determine the correct message type from an attestation
	// subnet.
	GossipAttestationMessage = "beacon_attestation"
	// GossipSyncCommitteeMessage is the name for the sync committee message type. It is
	// specially extracted so as to determine the correct message type from a sync committee
	// subnet.
	GossipSyncCommitteeMessage = "sync_committee"
	// GossipBlockMessage is the name for the block message type.
	GossipBlockMessage = "beacon_block"
	// GossipExitMessage is the name for the voluntary exit message type.
	GossipExitMessage = "voluntary_exit"
	// GossipProposerSlashingMessage is the name for the proposer slashing message type.
	GossipProposerSlashingMessage = "proposer_slashing"
	// GossipAttesterSlashingMessage is the name for the attester slashing message type.
	GossipAttesterSlashingMessage = "attester_slashing"
	// GossipAggregateAndProofMessage is the name for the attestation aggregate and proof message type.
	GossipAggregateAndProofMessage = "beacon_aggregate_and_proof"
	// GossipContributionAndProofMessage is the name for the sync contribution and proof message type.
	GossipContributionAndProofMessage = "sync_committee_contribution_and_proof"
	// GossipBlsToExecutionChangeMessage is the name for the bls to execution change message type.
	GossipBlsToExecutionChangeMessage = "bls_to_execution_change"
	// GossipBlobSidecarMessage is the name for the blob sidecar message type.
	GossipBlobSidecarMessage = "blob_sidecar"
	// Topic Formats
	//
	// AttestationSubnetTopicFormat is the topic format for the attestation subnet.
	AttestationSubnetTopicFormat = GossipProtocolAndDigest + GossipAttestationMessage + "_%d"
	// SyncCommitteeSubnetTopicFormat is the topic format for the sync committee subnet.
	SyncCommitteeSubnetTopicFormat = GossipProtocolAndDigest + GossipSyncCommitteeMessage + "_%d"
	// BlockSubnetTopicFormat is the topic format for the block subnet.
	BlockSubnetTopicFormat = GossipProtocolAndDigest + GossipBlockMessage
	// ExitSubnetTopicFormat is the topic format for the voluntary exit subnet.
	ExitSubnetTopicFormat = GossipProtocolAndDigest + GossipExitMessage
	// ProposerSlashingSubnetTopicFormat is the topic format for the proposer slashing subnet.
	ProposerSlashingSubnetTopicFormat = GossipProtocolAndDigest + GossipProposerSlashingMessage
	// AttesterSlashingSubnetTopicFormat is the topic format for the attester slashing subnet.
	AttesterSlashingSubnetTopicFormat = GossipProtocolAndDigest + GossipAttesterSlashingMessage
	// AggregateAndProofSubnetTopicFormat is the topic format for the aggregate and proof subnet.
	AggregateAndProofSubnetTopicFormat = GossipProtocolAndDigest + GossipAggregateAndProofMessage
	// SyncContributionAndProofSubnetTopicFormat is the topic format for the sync aggregate and proof subnet.
	SyncContributionAndProofSubnetTopicFormat = GossipProtocolAndDigest + GossipContributionAndProofMessage
	// BlsToExecutionChangeSubnetTopicFormat is the topic format for the bls to execution change subnet.
	BlsToExecutionChangeSubnetTopicFormat = GossipProtocolAndDigest + GossipBlsToExecutionChangeMessage
	// BlobSubnetTopicFormat is the topic format for the blob subnet.
	BlobSubnetTopicFormat = GossipProtocolAndDigest + GossipBlobSidecarMessage + "_%d"
)
