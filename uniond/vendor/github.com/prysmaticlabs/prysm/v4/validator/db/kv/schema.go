package kv

var (
	// Genesis information bucket key.
	genesisInfoBucket = []byte("genesis-info-bucket")

	// Validator slashing protection from double proposals.
	historicProposalsBucket            = []byte("proposal-history-bucket-interchange")
	deprecatedAttestationHistoryBucket = []byte("attestation-history-bucket-interchange")

	// Buckets for lowest signed source and target epoch for individual validator.
	lowestSignedSourceBucket = []byte("lowest-signed-source-bucket")
	lowestSignedTargetBucket = []byte("lowest-signed-target-bucket")

	// Lowest and highest signed proposals.
	lowestSignedProposalsBucket  = []byte("lowest-signed-proposals-bucket")
	highestSignedProposalsBucket = []byte("highest-signed-proposals-bucket")

	// Slashable public keys bucket.
	slashablePublicKeysBucket = []byte("slashable-public-keys")

	// Genesis validators root bucket key.
	genesisValidatorsRootKey = []byte("genesis-val-root")

	// Optimized slashing protection buckets and keys.
	pubKeysBucket                 = []byte("pubkeys-bucket")
	attestationSigningRootsBucket = []byte("att-signing-roots-bucket")
	attestationSourceEpochsBucket = []byte("att-source-epochs-bucket")
	attestationTargetEpochsBucket = []byte("att-target-epochs-bucket")

	// Migrations
	migrationsBucket = []byte("migrations")

	// Graffiti
	graffitiBucket = []byte("graffiti")

	// Graffiti ordered index and hash keys
	graffitiOrderedIndexKey = []byte("graffiti-ordered-index")
	graffitiFileHashKey     = []byte("graffiti-file-hash")

	// ProposerSettings stores the encoded proposer settings file
	proposerSettingsBucket = []byte("proposer-settings-bucket")
	proposerSettingsKey    = []byte("proposer-settings")
)
