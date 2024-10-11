package slasherkv

// The schema will define how to store and retrieve data from the db.
// we can prefix or suffix certain values such as `block` with attributes
// for prefix-wide scans across the underlying BoltDB buckets when filtering data.
// For example, we might store attestations as shard + attestation_root -> attestation, making
// it easy to scan for keys that have a certain shard number as a prefix and return those
// corresponding attestations.
var (
	// Slasher buckets.
	attestedEpochsByValidator  = []byte("attested-epochs-by-validator")
	attestationRecordsBucket   = []byte("attestation-records")
	attestationDataRootsBucket = []byte("attestation-data-roots")
	proposalRecordsBucket      = []byte("proposal-records")
	slasherChunksBucket        = []byte("slasher-chunks")
)
