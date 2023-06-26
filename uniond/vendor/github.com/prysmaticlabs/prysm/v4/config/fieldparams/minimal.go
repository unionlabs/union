//go:build minimal

package field_params

const (
	Preset                                = "minimal"
	BlockRootsLength                      = 64            // SLOTS_PER_HISTORICAL_ROOT
	StateRootsLength                      = 64            // SLOTS_PER_HISTORICAL_ROOT
	RandaoMixesLength                     = 64            // EPOCHS_PER_HISTORICAL_VECTOR
	HistoricalRootsLength                 = 16777216      // HISTORICAL_ROOTS_LIMIT
	ValidatorRegistryLimit                = 1099511627776 // VALIDATOR_REGISTRY_LIMIT
	Eth1DataVotesLength                   = 32            // SLOTS_PER_ETH1_VOTING_PERIOD
	PreviousEpochAttestationsLength       = 1024          // MAX_ATTESTATIONS * SLOTS_PER_EPOCH
	CurrentEpochAttestationsLength        = 1024          // MAX_ATTESTATIONS * SLOTS_PER_EPOCH
	SlashingsLength                       = 64            // EPOCHS_PER_SLASHINGS_VECTOR
	SyncCommitteeLength                   = 32            // SYNC_COMMITTEE_SIZE
	RootLength                            = 32            // RootLength defines the byte length of a Merkle root.
	BLSSignatureLength                    = 96            // BLSSignatureLength defines the byte length of a BLSSignature.
	BLSPubkeyLength                       = 48            // BLSPubkeyLength defines the byte length of a BLSSignature.
	MaxTxsPerPayloadLength                = 1048576       // MaxTxsPerPayloadLength defines the maximum number of transactions that can be included in a payload.
	MaxBytesPerTxLength                   = 1073741824    // MaxBytesPerTxLength defines the maximum number of bytes that can be included in a transaction.
	FeeRecipientLength                    = 20            // FeeRecipientLength defines the byte length of a fee recipient.
	LogsBloomLength                       = 256           // LogsBloomLength defines the byte length of a logs bloom.
	VersionLength                         = 4             // VersionLength defines the byte length of a fork version number.
	SlotsPerEpoch                         = 8             // SlotsPerEpoch defines the number of slots per epoch.
	SyncCommitteeAggregationBytesLength   = 1             // SyncCommitteeAggregationBytesLength defines the sync committee aggregate bytes.
	SyncAggregateSyncCommitteeBytesLength = 4             // SyncAggregateSyncCommitteeBytesLength defines the length of sync committee bytes in a sync aggregate.
	MaxWithdrawalsPerPayload              = 4             // MaxWithdrawalsPerPayloadLength defines the maximum number of withdrawals that can be included in a payload.
)
