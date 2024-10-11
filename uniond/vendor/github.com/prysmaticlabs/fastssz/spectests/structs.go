package spectests

import (
	"github.com/prysmaticlabs/fastssz/spectests/external"
	"github.com/prysmaticlabs/fastssz/spectests/external2"
)

type AggregateAndProof struct {
	Index          uint64             `json:"aggregator_index"`
	Aggregate      *Attestation       `json:"aggregate"`
	SelectionProof external.Signature `json:"selection_proof" ssz-size:"96"`
}

type Checkpoint struct {
	Epoch external2.EpochAlias `json:"epoch"`
	Root  []byte               `json:"root" ssz-size:"32"`
}

type Slot uint64 // alias from the same package

type Hash [32]byte

type AttestationData struct {
	Slot            Slot        `json:"slot"`
	Index           uint64      `json:"index"`
	BeaconBlockHash Hash        `json:"beacon_block_root" ssz-size:"32"`
	Source          *Checkpoint `json:"source"`
	Target          *Checkpoint `json:"target"`
}

type Attestation struct {
	AggregationBits []byte              `json:"aggregation_bits" ssz:"bitlist" ssz-max:"2048"`
	Data            *AttestationData    `json:"data"`
	Signature       *external.Signature `json:"signature" ssz-size:"96"`
}

type DepositData struct {
	Pubkey                [48]byte       `json:"pubkey" ssz-size:"48"`
	WithdrawalCredentials [32]byte       `json:"withdrawal_credentials" ssz-size:"32"`
	Amount                uint64         `json:"amount"`
	Signature             external.Bytes `json:"signature" ssz-size:"96"`
	Root                  [32]byte       `ssz:"-"`
}

type Deposit struct {
	Proof [][]byte `ssz-size:"33,32"`
	Data  *DepositData
}

type DepositMessage struct {
	Pubkey                []byte `json:"pubkey" ssz-size:"48"`
	WithdrawalCredentials []byte `json:"withdrawal_credentials" ssz-size:"32"`
	Amount                uint64 `json:"amount"`
}

type IndexedAttestation struct {
	AttestationIndices []uint64         `json:"attesting_indices" ssz-max:"2048"`
	Data               *AttestationData `json:"data"`
	Signature          []byte           `json:"signature" ssz-size:"96"`
}

type PendingAttestation struct {
	AggregationBits []byte           `json:"aggregation_bits" ssz:"bitlist" ssz-max:"2048"`
	Data            *AttestationData `json:"data"`
	InclusionDelay  uint64           `json:"inclusion_delay"`
	ProposerIndex   uint64           `json:"proposer_index"`
}

type Fork struct {
	PreviousVersion []byte `json:"previous_version" ssz-size:"4"`
	CurrentVersion  []byte `json:"current_version" ssz-size:"4"`
	Epoch           uint64 `json:"epoch"`
}

type Validator struct {
	Pubkey                     []byte `json:"pubkey" ssz-size:"48"`
	WithdrawalCredentials      []byte `json:"withdrawal_credentials" ssz-size:"32"`
	EffectiveBalance           uint64 `json:"effective_balance"`
	Slashed                    bool   `json:"slashed"`
	ActivationEligibilityEpoch uint64 `json:"activation_eligibility_epoch"`
	ActivationEpoch            uint64 `json:"activation_epoch"`
	ExitEpoch                  uint64 `json:"exit_epoch"`
	WithdrawableEpoch          uint64 `json:"withdrawable_epoch"`
}

type VoluntaryExit struct {
	Epoch          uint64 `json:"epoch"`
	ValidatorIndex uint64 `json:"validator_index"`
}

type SignedVoluntaryExit struct {
	Exit      *VoluntaryExit          `json:"message"`
	Signature external.FixedSignature `json:"signature" ssz-size:"96"`
}

type Eth1Block struct {
	Timestamp    uint64 `json:"timestamp"`
	DepositRoot  []byte `json:"deposit_root" ssz-size:"32"`
	DepositCount uint64 `json:"deposit_count" ssz-size:"32"`
}

type Eth1Data struct {
	DepositRoot  []byte `json:"deposit_root" ssz-size:"32"`
	DepositCount uint64 `json:"deposit_count"`
	BlockHash    []byte `json:"block_hash" ssz-size:"32"`
}

type SigningRoot struct {
	ObjectRoot []byte `json:"object_root" ssz-size:"32"`
	Domain     []byte `json:"domain" ssz-size:"8"`
}

type HistoricalBatch struct {
	BlockRoots [64][32]byte `json:"block_roots" ssz-size:"64,32"`
	StateRoots [][]byte     `json:"state_roots" ssz-size:"64,32"`
}

type ProposerSlashing struct {
	Header1 *SignedBeaconBlockHeader `json:"signed_header_1"`
	Header2 *SignedBeaconBlockHeader `json:"signed_header_2"`
}

type AttesterSlashing struct {
	Attestation1 *IndexedAttestation `json:"attestation_1"`
	Attestation2 *IndexedAttestation `json:"attestation_2"`
}

type BeaconState struct {
	GenesisTime           uint64             `json:"genesis_time"`
	GenesisValidatorsRoot []byte             `json:"genesis_validators_root" ssz-size:"32"`
	Slot                  uint64             `json:"slot"`
	Fork                  *Fork              `json:"fork"`
	LatestBlockHeader     *BeaconBlockHeader `json:"latest_block_header"`
	BlockRoots            [64][32]byte       `json:"block_roots" ssz-size:"64,32"`
	StateRoots            [][32]byte         `json:"state_roots" ssz-size:"64,32"`
	HistoricalRoots       [][32]byte         `json:"historical_roots" ssz-size:"?,32" ssz-max:"16777216"`
	Eth1Data              *Eth1Data          `json:"eth1_data"`
	Eth1DataVotes         []*Eth1Data        `json:"eth1_data_votes" ssz-max:"32"`
	Eth1DepositIndex      uint64             `json:"eth1_deposit_index"`
	Validators            []*Validator       `json:"validators" ssz-max:"1099511627776"`
	Balances              []uint64           `json:"balances" ssz-max:"1099511627776"`
	RandaoMixes           [][]byte           `json:"randao_mixes" ssz-size:"64,32"`
	Slashings             []uint64           `json:"slashings" ssz-size:"64"`

	PreviousEpochParticipation []uint8 `json:"previous_epoch_participation" ssz-max:"1099511627776"`
	CurrentEpochParticipation  []uint8 `json:"current_epoch_participation" ssz-max:"1099511627776"`
	JustificationBits          []byte  `json:"justification_bits" ssz-size:"1"`

	PreviousJustifiedCheckpoint *Checkpoint `json:"previous_justified_checkpoint"`
	CurrentJustifiedCheckpoint  *Checkpoint `json:"current_justified_checkpoint"`
	FinalizedCheckpoint         *Checkpoint `json:"finalized_checkpoint"`

	InactivityScores    []uint64              `json:"inactivity_scores" ssz-max:"1099511627776"`
	CurrentSyncCommitee *SyncCommitteeMinimal `json:"current_sync_committee"`
	NextSyncCommittee   *SyncCommitteeMinimal `json:"next_sync_committee"`
}

type BeaconBlock struct {
	Slot          uint64           `json:"slot"`
	ProposerIndex uint64           `json:"proposer_index"`
	ParentRoot    []byte           `json:"parent_root" ssz-size:"32"`
	StateRoot     []byte           `json:"state_root" ssz-size:"32"`
	Body          *BeaconBlockBody `json:"body"`
}

type SignedBeaconBlock struct {
	Block     *BeaconBlock `json:"message"`
	Signature []byte       `json:"signature" ssz-size:"96"`
}

type Transfer struct {
	Sender    uint64 `json:"sender"`
	Recipient uint64 `json:"recipient"`
	Amount    uint64 `json:"amount"`
	Fee       uint64 `json:"fee"`
	Slot      uint64 `json:"slot"`
	Pubkey    []byte `json:"pubkey" ssz-size:"48"`
	Signature []byte `json:"signature" ssz-size:"96"`
}

type BeaconBlockBody struct {
	RandaoReveal      []byte                 `json:"randao_reveal" ssz-size:"96"`
	Eth1Data          *Eth1Data              `json:"eth1_data"`
	Graffiti          [32]byte               `json:"graffiti" ssz-size:"32"`
	ProposerSlashings []*ProposerSlashing    `json:"proposer_slashings" ssz-max:"16"`
	AttesterSlashings []*AttesterSlashing    `json:"attester_slashings" ssz-max:"2"`
	Attestations      []*Attestation         `json:"attestations" ssz-max:"128"`
	Deposits          []*Deposit             `json:"deposits" ssz-max:"16"`
	VoluntaryExits    []*SignedVoluntaryExit `json:"voluntary_exits" ssz-max:"16"`
	SyncAggregate     *SyncAggregate         `json:"sync_aggregate"`
}

type SignedBeaconBlockHeader struct {
	Header    *BeaconBlockHeader `json:"message"`
	Signature []byte             `json:"signature" ssz-size:"96"`
}

type BeaconBlockHeader struct {
	Slot          uint64 `json:"slot"`
	ProposerIndex uint64 `json:"proposer_index"`
	ParentRoot    []byte `json:"parent_root" ssz-size:"32"`
	StateRoot     []byte `json:"state_root" ssz-size:"32"`
	BodyRoot      []byte `json:"body_root" ssz-size:"32"`
}

type ErrorResponse struct {
	Message external.DynamicBytes `ssz-max:"256"`
}

type Dummy struct {
}

type Interface interface {
}

type SyncCommittee struct {
	PubKeys          [][]byte     `json:"pubkeys" ssz-size:"1024,48"`
	PubKeyAggregates [16][48]byte `json:"pubkey_aggregates" ssz-size:"16,48"`
}

type SyncAggregate struct {
	SyncCommiteeBits      []byte   `json:"sync_committee_bits" ssz-size:"128"`
	SyncCommiteeSignature [96]byte `json:"sync_committee_signature" ssz-size:"96"`
}

// minimal versions

type SyncCommitteeMinimal struct {
	PubKeys          [][]byte    `json:"pubkeys" ssz-size:"32,48"`
	PubKeyAggregates [2][48]byte `json:"pubkey_aggregates" ssz-size:"2,48"`
}

type SyncAggregateMinimal struct {
	SyncCommiteeBits      []byte   `json:"sync_committee_bits" ssz-size:"4"`
	SyncCommiteeSignature [96]byte `json:"sync_committee_signature" ssz-size:"96"`
}

type SignedBeaconBlockMinimal struct {
	Block     *BeaconBlockMinimal `json:"message"`
	Signature []byte              `json:"signature" ssz-size:"96"`
}

type BeaconBlockBodyMinimal struct {
	RandaoReveal      []byte                 `json:"randao_reveal" ssz-size:"96"`
	Eth1Data          *Eth1Data              `json:"eth1_data"`
	Graffiti          [32]byte               `json:"graffiti" ssz-size:"32"`
	ProposerSlashings []*ProposerSlashing    `json:"proposer_slashings" ssz-max:"16"`
	AttesterSlashings []*AttesterSlashing    `json:"attester_slashings" ssz-max:"2"`
	Attestations      []*Attestation         `json:"attestations" ssz-max:"128"`
	Deposits          []*Deposit             `json:"deposits" ssz-max:"16"`
	VoluntaryExits    []*SignedVoluntaryExit `json:"voluntary_exits" ssz-max:"16"`
	SyncAggregate     *SyncAggregateMinimal  `json:"sync_aggregate"`
}

type BeaconBlockMinimal struct {
	Slot          uint64                  `json:"slot"`
	ProposerIndex uint64                  `json:"proposer_index"`
	ParentRoot    []byte                  `json:"parent_root" ssz-size:"32"`
	StateRoot     []byte                  `json:"state_root" ssz-size:"32"`
	Body          *BeaconBlockBodyMinimal `json:"body"`
}
