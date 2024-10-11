package pex

import (
	"time"

	"github.com/cometbft/cometbft/p2p"
)

// knownAddress tracks information about a known network address
// that is used to determine how viable an address is.
type knownAddress struct {
	Addr        *p2p.NetAddress `json:"addr"`
	Src         *p2p.NetAddress `json:"src"`
	Buckets     []int           `json:"buckets"`
	Attempts    int32           `json:"attempts"`
	BucketType  byte            `json:"bucket_type"`
	LastAttempt time.Time       `json:"last_attempt"`
	LastSuccess time.Time       `json:"last_success"`
	LastBanTime time.Time       `json:"last_ban_time"`
}

func newKnownAddress(addr *p2p.NetAddress, src *p2p.NetAddress) *knownAddress {
	return &knownAddress{
		Addr:        addr,
		Src:         src,
		Attempts:    0,
		LastAttempt: time.Now(),
		BucketType:  bucketTypeNew,
		Buckets:     nil,
	}
}

func (ka *knownAddress) ID() p2p.ID {
	return ka.Addr.ID
}

func (ka *knownAddress) isOld() bool {
	return ka.BucketType == bucketTypeOld
}

func (ka *knownAddress) isNew() bool {
	return ka.BucketType == bucketTypeNew
}

func (ka *knownAddress) markAttempt() {
	now := time.Now()
	ka.LastAttempt = now
	ka.Attempts++
}

func (ka *knownAddress) markGood() {
	now := time.Now()
	ka.LastAttempt = now
	ka.Attempts = 0
	ka.LastSuccess = now
}

func (ka *knownAddress) ban(banTime time.Duration) {
	if ka.LastBanTime.Before(time.Now().Add(banTime)) {
		ka.LastBanTime = time.Now().Add(banTime)
	}
}

func (ka *knownAddress) isBanned() bool {
	return ka.LastBanTime.After(time.Now())
}

func (ka *knownAddress) addBucketRef(bucketIdx int) int {
	for _, bucket := range ka.Buckets {
		if bucket == bucketIdx {
			// TODO refactor to return error?
			// log.Warn(Fmt("Bucket already exists in ka.Buckets: %v", ka))
			return -1
		}
	}
	ka.Buckets = append(ka.Buckets, bucketIdx)
	return len(ka.Buckets)
}

func (ka *knownAddress) removeBucketRef(bucketIdx int) int {
	buckets := []int{}
	for _, bucket := range ka.Buckets {
		if bucket != bucketIdx {
			buckets = append(buckets, bucket)
		}
	}
	if len(buckets) != len(ka.Buckets)-1 {
		// TODO refactor to return error?
		// log.Warn(Fmt("bucketIdx not found in ka.Buckets: %v", ka))
		return -1
	}
	ka.Buckets = buckets
	return len(ka.Buckets)
}

/*
An address is bad if the address in question is a New address, has not been tried in the last
minute, and meets one of the following criteria:

1) It claims to be from the future
2) It hasn't been seen in over a week
3) It has failed at least three times and never succeeded
4) It has failed ten times in the last week

All addresses that meet these criteria are assumed to be worthless and not
worth keeping hold of.
*/
func (ka *knownAddress) isBad() bool {
	// Is Old --> good
	if ka.BucketType == bucketTypeOld {
		return false
	}

	// Has been attempted in the last minute --> good
	if ka.LastAttempt.After(time.Now().Add(-1 * time.Minute)) {
		return false
	}

	// TODO: From the future?

	// Too old?
	// TODO: should be a timestamp of last seen, not just last attempt
	if ka.LastAttempt.Before(time.Now().Add(-1 * numMissingDays * time.Hour * 24)) {
		return true
	}

	// Never succeeded?
	if ka.LastSuccess.IsZero() && ka.Attempts >= numRetries {
		return true
	}

	// Hasn't succeeded in too long?
	if ka.LastSuccess.Before(time.Now().Add(-1*minBadDays*time.Hour*24)) &&
		ka.Attempts >= maxFailures {
		return true
	}

	return false
}
