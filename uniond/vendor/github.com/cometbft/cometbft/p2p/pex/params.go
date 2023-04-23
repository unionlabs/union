package pex

import "time"

const (
	// addresses under which the address manager will claim to need more addresses.
	needAddressThreshold = 1000

	// interval used to dump the address cache to disk for future use.
	dumpAddressInterval = time.Minute * 2

	// max addresses in each old address bucket.
	oldBucketSize = 64

	// buckets we split old addresses over.
	oldBucketCount = 64

	// max addresses in each new address bucket.
	newBucketSize = 64

	// buckets that we spread new addresses over.
	newBucketCount = 256

	// old buckets over which an address group will be spread.
	oldBucketsPerGroup = 4

	// new buckets over which a source address group will be spread.
	newBucketsPerGroup = 32

	// buckets a frequently seen new address may end up in.
	maxNewBucketsPerAddress = 4

	// days before which we assume an address has vanished
	// if we have not seen it announced in that long.
	numMissingDays = 7

	// tries without a single success before we assume an address is bad.
	numRetries = 3

	// max failures we will accept without a success before considering an address bad.
	maxFailures = 10 // ?

	// days since the last success before we will consider evicting an address.
	minBadDays = 7

	// % of total addresses known returned by GetSelection.
	getSelectionPercent = 23

	// min addresses that must be returned by GetSelection. Useful for bootstrapping.
	minGetSelection = 32

	// max addresses returned by GetSelection
	// NOTE: this must match "maxMsgSize"
	maxGetSelection = 250
)
