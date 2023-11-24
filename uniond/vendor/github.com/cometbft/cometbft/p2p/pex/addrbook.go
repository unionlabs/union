// Modified for CometBFT
// Originally Copyright (c) 2013-2014 Conformal Systems LLC.
// https://github.com/conformal/btcd/blob/master/LICENSE

package pex

import (
	"encoding/binary"
	"fmt"
	"hash"
	"math"
	"math/rand"
	"net"
	"sync"
	"time"

	"github.com/minio/highwayhash"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/libs/log"
	cmtmath "github.com/cometbft/cometbft/libs/math"
	cmtrand "github.com/cometbft/cometbft/libs/rand"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
)

const (
	bucketTypeNew = 0x01
	bucketTypeOld = 0x02
)

// AddrBook is an address book used for tracking peers
// so we can gossip about them to others and select
// peers to dial.
// TODO: break this up?
type AddrBook interface {
	service.Service

	// Add our own addresses so we don't later add ourselves
	AddOurAddress(*p2p.NetAddress)
	// Check if it is our address
	OurAddress(*p2p.NetAddress) bool

	AddPrivateIDs([]string)

	// Add and remove an address
	AddAddress(addr *p2p.NetAddress, src *p2p.NetAddress) error
	RemoveAddress(*p2p.NetAddress)

	// Check if the address is in the book
	HasAddress(*p2p.NetAddress) bool

	// Do we need more peers?
	NeedMoreAddrs() bool
	// Is Address Book Empty? Answer should not depend on being in your own
	// address book, or private peers
	Empty() bool

	// Pick an address to dial
	PickAddress(biasTowardsNewAddrs int) *p2p.NetAddress

	// Mark address
	MarkGood(p2p.ID)
	MarkAttempt(*p2p.NetAddress)
	MarkBad(*p2p.NetAddress, time.Duration) // Move peer to bad peers list
	// Add bad peers back to addrBook
	ReinstateBadPeers()

	IsGood(*p2p.NetAddress) bool
	IsBanned(*p2p.NetAddress) bool

	// Send a selection of addresses to peers
	GetSelection() []*p2p.NetAddress
	// Send a selection of addresses with bias
	GetSelectionWithBias(biasTowardsNewAddrs int) []*p2p.NetAddress

	Size() int

	// Persist to disk
	Save()
}

var _ AddrBook = (*addrBook)(nil)

// addrBook - concurrency safe peer address manager.
// Implements AddrBook.
type addrBook struct {
	service.BaseService

	// accessed concurrently
	mtx        cmtsync.Mutex
	rand       *cmtrand.Rand
	ourAddrs   map[string]struct{}
	privateIDs map[p2p.ID]struct{}
	addrLookup map[p2p.ID]*knownAddress // new & old
	badPeers   map[p2p.ID]*knownAddress // blacklisted peers
	bucketsOld []map[string]*knownAddress
	bucketsNew []map[string]*knownAddress
	nOld       int
	nNew       int

	// immutable after creation
	filePath          string
	key               string // random prefix for bucket placement
	routabilityStrict bool
	hasher            hash.Hash64

	wg sync.WaitGroup
}

func mustNewHasher() hash.Hash64 {
	key := crypto.CRandBytes(highwayhash.Size)
	hasher, err := highwayhash.New64(key)
	if err != nil {
		panic(err)
	}
	return hasher
}

// NewAddrBook creates a new address book.
// Use Start to begin processing asynchronous address updates.
func NewAddrBook(filePath string, routabilityStrict bool) AddrBook {
	am := &addrBook{
		rand:              cmtrand.NewRand(),
		ourAddrs:          make(map[string]struct{}),
		privateIDs:        make(map[p2p.ID]struct{}),
		addrLookup:        make(map[p2p.ID]*knownAddress),
		badPeers:          make(map[p2p.ID]*knownAddress),
		filePath:          filePath,
		routabilityStrict: routabilityStrict,
	}
	am.init()
	am.BaseService = *service.NewBaseService(nil, "AddrBook", am)
	return am
}

// Initialize the buckets.
// When modifying this, don't forget to update loadFromFile()
func (a *addrBook) init() {
	a.key = crypto.CRandHex(24) // 24/2 * 8 = 96 bits
	// New addr buckets
	a.bucketsNew = make([]map[string]*knownAddress, newBucketCount)
	for i := range a.bucketsNew {
		a.bucketsNew[i] = make(map[string]*knownAddress)
	}
	// Old addr buckets
	a.bucketsOld = make([]map[string]*knownAddress, oldBucketCount)
	for i := range a.bucketsOld {
		a.bucketsOld[i] = make(map[string]*knownAddress)
	}
	a.hasher = mustNewHasher()
}

// OnStart implements Service.
func (a *addrBook) OnStart() error {
	if err := a.BaseService.OnStart(); err != nil {
		return err
	}
	a.loadFromFile(a.filePath)

	// wg.Add to ensure that any invocation of .Wait()
	// later on will wait for saveRoutine to terminate.
	a.wg.Add(1)
	go a.saveRoutine()

	return nil
}

// OnStop implements Service.
func (a *addrBook) OnStop() {
	a.BaseService.OnStop()
}

func (a *addrBook) Wait() {
	a.wg.Wait()
}

func (a *addrBook) FilePath() string {
	return a.filePath
}

//-------------------------------------------------------

// AddOurAddress one of our addresses.
func (a *addrBook) AddOurAddress(addr *p2p.NetAddress) {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	a.Logger.Info("Add our address to book", "addr", addr)
	a.ourAddrs[addr.String()] = struct{}{}
}

// OurAddress returns true if it is our address.
func (a *addrBook) OurAddress(addr *p2p.NetAddress) bool {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	_, ok := a.ourAddrs[addr.String()]
	return ok
}

func (a *addrBook) AddPrivateIDs(ids []string) {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	for _, id := range ids {
		a.privateIDs[p2p.ID(id)] = struct{}{}
	}
}

// AddAddress implements AddrBook
// Add address to a "new" bucket. If it's already in one, only add it probabilistically.
// Returns error if the addr is non-routable. Does not add self.
// NOTE: addr must not be nil
func (a *addrBook) AddAddress(addr *p2p.NetAddress, src *p2p.NetAddress) error {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	return a.addAddress(addr, src)
}

// RemoveAddress implements AddrBook - removes the address from the book.
func (a *addrBook) RemoveAddress(addr *p2p.NetAddress) {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	a.removeAddress(addr)
}

// IsGood returns true if peer was ever marked as good and haven't
// done anything wrong since then.
func (a *addrBook) IsGood(addr *p2p.NetAddress) bool {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	return a.addrLookup[addr.ID].isOld()
}

// IsBanned returns true if the peer is currently banned
func (a *addrBook) IsBanned(addr *p2p.NetAddress) bool {
	a.mtx.Lock()
	_, ok := a.badPeers[addr.ID]
	a.mtx.Unlock()

	return ok
}

// HasAddress returns true if the address is in the book.
func (a *addrBook) HasAddress(addr *p2p.NetAddress) bool {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	ka := a.addrLookup[addr.ID]
	return ka != nil
}

// NeedMoreAddrs implements AddrBook - returns true if there are not have enough addresses in the book.
func (a *addrBook) NeedMoreAddrs() bool {
	return a.Size() < needAddressThreshold
}

// Empty implements AddrBook - returns true if there are no addresses in the address book.
// Does not count the peer appearing in its own address book, or private peers.
func (a *addrBook) Empty() bool {
	return a.Size() == 0
}

// PickAddress implements AddrBook. It picks an address to connect to.
// The address is picked randomly from an old or new bucket according
// to the biasTowardsNewAddrs argument, which must be between [0, 100] (or else is truncated to that range)
// and determines how biased we are to pick an address from a new bucket.
// PickAddress returns nil if the AddrBook is empty or if we try to pick
// from an empty bucket.
func (a *addrBook) PickAddress(biasTowardsNewAddrs int) *p2p.NetAddress {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	bookSize := a.size()
	if bookSize <= 0 {
		if bookSize < 0 {
			panic(fmt.Sprintf("Addrbook size %d (new: %d + old: %d) is less than 0", a.nNew+a.nOld, a.nNew, a.nOld))
		}
		return nil
	}
	if biasTowardsNewAddrs > 100 {
		biasTowardsNewAddrs = 100
	}
	if biasTowardsNewAddrs < 0 {
		biasTowardsNewAddrs = 0
	}

	// Bias between new and old addresses.
	oldCorrelation := math.Sqrt(float64(a.nOld)) * (100.0 - float64(biasTowardsNewAddrs))
	newCorrelation := math.Sqrt(float64(a.nNew)) * float64(biasTowardsNewAddrs)

	// pick a random peer from a random bucket
	var bucket map[string]*knownAddress
	pickFromOldBucket := (newCorrelation+oldCorrelation)*a.rand.Float64() < oldCorrelation
	if (pickFromOldBucket && a.nOld == 0) ||
		(!pickFromOldBucket && a.nNew == 0) {
		return nil
	}
	// loop until we pick a random non-empty bucket
	for len(bucket) == 0 {
		if pickFromOldBucket {
			bucket = a.bucketsOld[a.rand.Intn(len(a.bucketsOld))]
		} else {
			bucket = a.bucketsNew[a.rand.Intn(len(a.bucketsNew))]
		}
	}
	// pick a random index and loop over the map to return that index
	randIndex := a.rand.Intn(len(bucket))
	for _, ka := range bucket {
		if randIndex == 0 {
			return ka.Addr
		}
		randIndex--
	}
	return nil
}

// MarkGood implements AddrBook - it marks the peer as good and
// moves it into an "old" bucket.
func (a *addrBook) MarkGood(id p2p.ID) {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	ka := a.addrLookup[id]
	if ka == nil {
		return
	}
	ka.markGood()
	if ka.isNew() {
		if err := a.moveToOld(ka); err != nil {
			a.Logger.Error("Error moving address to old", "err", err)
		}
	}
}

// MarkAttempt implements AddrBook - it marks that an attempt was made to connect to the address.
func (a *addrBook) MarkAttempt(addr *p2p.NetAddress) {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	ka := a.addrLookup[addr.ID]
	if ka == nil {
		return
	}
	ka.markAttempt()
}

// MarkBad implements AddrBook. Kicks address out from book, places
// the address in the badPeers pool.
func (a *addrBook) MarkBad(addr *p2p.NetAddress, banTime time.Duration) {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	if a.addBadPeer(addr, banTime) {
		a.removeAddress(addr)
	}
}

// ReinstateBadPeers removes bad peers from ban list and places them into a new
// bucket.
func (a *addrBook) ReinstateBadPeers() {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	for _, ka := range a.badPeers {
		if ka.isBanned() {
			continue
		}

		bucket, err := a.calcNewBucket(ka.Addr, ka.Src)
		if err != nil {
			a.Logger.Error("Failed to calculate new bucket (bad peer won't be reinstantiated)",
				"addr", ka.Addr, "err", err)
			continue
		}

		if err := a.addToNewBucket(ka, bucket); err != nil {
			a.Logger.Error("Error adding peer to new bucket", "err", err)
		}
		delete(a.badPeers, ka.ID())

		a.Logger.Info("Reinstated address", "addr", ka.Addr)
	}
}

// GetSelection implements AddrBook.
// It randomly selects some addresses (old & new). Suitable for peer-exchange protocols.
// Must never return a nil address.
func (a *addrBook) GetSelection() []*p2p.NetAddress {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	bookSize := a.size()
	if bookSize <= 0 {
		if bookSize < 0 {
			panic(fmt.Sprintf("Addrbook size %d (new: %d + old: %d) is less than 0", a.nNew+a.nOld, a.nNew, a.nOld))
		}
		return nil
	}

	numAddresses := cmtmath.MaxInt(
		cmtmath.MinInt(minGetSelection, bookSize),
		bookSize*getSelectionPercent/100)
	numAddresses = cmtmath.MinInt(maxGetSelection, numAddresses)

	// XXX: instead of making a list of all addresses, shuffling, and slicing a random chunk,
	// could we just select a random numAddresses of indexes?
	allAddr := make([]*p2p.NetAddress, bookSize)
	i := 0
	for _, ka := range a.addrLookup {
		allAddr[i] = ka.Addr
		i++
	}

	// Fisher-Yates shuffle the array. We only need to do the first
	// `numAddresses' since we are throwing the rest.
	for i := 0; i < numAddresses; i++ {
		// pick a number between current index and the end
		j := cmtrand.Intn(len(allAddr)-i) + i
		allAddr[i], allAddr[j] = allAddr[j], allAddr[i]
	}

	// slice off the limit we are willing to share.
	return allAddr[:numAddresses]
}

func percentageOfNum(p, n int) int {
	return int(math.Round((float64(p) / float64(100)) * float64(n)))
}

// GetSelectionWithBias implements AddrBook.
// It randomly selects some addresses (old & new). Suitable for peer-exchange protocols.
// Must never return a nil address.
//
// Each address is picked randomly from an old or new bucket according to the
// biasTowardsNewAddrs argument, which must be between [0, 100] (or else is truncated to
// that range) and determines how biased we are to pick an address from a new
// bucket.
func (a *addrBook) GetSelectionWithBias(biasTowardsNewAddrs int) []*p2p.NetAddress {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	bookSize := a.size()
	if bookSize <= 0 {
		if bookSize < 0 {
			panic(fmt.Sprintf("Addrbook size %d (new: %d + old: %d) is less than 0", a.nNew+a.nOld, a.nNew, a.nOld))
		}
		return nil
	}

	if biasTowardsNewAddrs > 100 {
		biasTowardsNewAddrs = 100
	}
	if biasTowardsNewAddrs < 0 {
		biasTowardsNewAddrs = 0
	}

	numAddresses := cmtmath.MaxInt(
		cmtmath.MinInt(minGetSelection, bookSize),
		bookSize*getSelectionPercent/100)
	numAddresses = cmtmath.MinInt(maxGetSelection, numAddresses)

	// number of new addresses that, if possible, should be in the beginning of the selection
	// if there are no enough old addrs, will choose new addr instead.
	numRequiredNewAdd := cmtmath.MaxInt(percentageOfNum(biasTowardsNewAddrs, numAddresses), numAddresses-a.nOld)
	selection := a.randomPickAddresses(bucketTypeNew, numRequiredNewAdd)
	selection = append(selection, a.randomPickAddresses(bucketTypeOld, numAddresses-len(selection))...)
	return selection
}

//------------------------------------------------

// Size returns the number of addresses in the book.
func (a *addrBook) Size() int {
	a.mtx.Lock()
	defer a.mtx.Unlock()

	return a.size()
}

func (a *addrBook) size() int {
	return a.nNew + a.nOld
}

//----------------------------------------------------------

// Save persists the address book to disk.
func (a *addrBook) Save() {
	a.saveToFile(a.filePath) // thread safe
}

func (a *addrBook) saveRoutine() {
	defer a.wg.Done()

	saveFileTicker := time.NewTicker(dumpAddressInterval)
out:
	for {
		select {
		case <-saveFileTicker.C:
			a.saveToFile(a.filePath)
		case <-a.Quit():
			break out
		}
	}
	saveFileTicker.Stop()
	a.saveToFile(a.filePath)
}

//----------------------------------------------------------

func (a *addrBook) getBucket(bucketType byte, bucketIdx int) map[string]*knownAddress {
	switch bucketType {
	case bucketTypeNew:
		return a.bucketsNew[bucketIdx]
	case bucketTypeOld:
		return a.bucketsOld[bucketIdx]
	default:
		panic("Invalid bucket type")
	}
}

// Adds ka to new bucket. Returns false if it couldn't do it cuz buckets full.
// NOTE: currently it always returns true.
func (a *addrBook) addToNewBucket(ka *knownAddress, bucketIdx int) error {
	// Consistency check to ensure we don't add an already known address
	if ka.isOld() {
		return errAddrBookOldAddressNewBucket{ka.Addr, bucketIdx}
	}

	addrStr := ka.Addr.String()
	bucket := a.getBucket(bucketTypeNew, bucketIdx)

	// Already exists?
	if _, ok := bucket[addrStr]; ok {
		return nil
	}

	// Enforce max addresses.
	if len(bucket) > newBucketSize {
		a.Logger.Info("new bucket is full, expiring new")
		a.expireNew(bucketIdx)
	}

	// Add to bucket.
	bucket[addrStr] = ka
	// increment nNew if the peer doesnt already exist in a bucket
	if ka.addBucketRef(bucketIdx) == 1 {
		a.nNew++
	}

	// Add it to addrLookup
	a.addrLookup[ka.ID()] = ka
	return nil
}

// Adds ka to old bucket. Returns false if it couldn't do it cuz buckets full.
func (a *addrBook) addToOldBucket(ka *knownAddress, bucketIdx int) bool {
	// Sanity check
	if ka.isNew() {
		a.Logger.Error(fmt.Sprintf("Cannot add new address to old bucket: %v", ka))
		return false
	}
	if len(ka.Buckets) != 0 {
		a.Logger.Error(fmt.Sprintf("Cannot add already old address to another old bucket: %v", ka))
		return false
	}

	addrStr := ka.Addr.String()
	bucket := a.getBucket(bucketTypeOld, bucketIdx)

	// Already exists?
	if _, ok := bucket[addrStr]; ok {
		return true
	}

	// Enforce max addresses.
	if len(bucket) > oldBucketSize {
		return false
	}

	// Add to bucket.
	bucket[addrStr] = ka
	if ka.addBucketRef(bucketIdx) == 1 {
		a.nOld++
	}

	// Ensure in addrLookup
	a.addrLookup[ka.ID()] = ka

	return true
}

func (a *addrBook) removeFromBucket(ka *knownAddress, bucketType byte, bucketIdx int) {
	if ka.BucketType != bucketType {
		a.Logger.Error(fmt.Sprintf("Bucket type mismatch: %v", ka))
		return
	}
	bucket := a.getBucket(bucketType, bucketIdx)
	delete(bucket, ka.Addr.String())
	if ka.removeBucketRef(bucketIdx) == 0 {
		if bucketType == bucketTypeNew {
			a.nNew--
		} else {
			a.nOld--
		}
		delete(a.addrLookup, ka.ID())
	}
}

func (a *addrBook) removeFromAllBuckets(ka *knownAddress) {
	for _, bucketIdx := range ka.Buckets {
		bucket := a.getBucket(ka.BucketType, bucketIdx)
		delete(bucket, ka.Addr.String())
	}
	ka.Buckets = nil
	if ka.BucketType == bucketTypeNew {
		a.nNew--
	} else {
		a.nOld--
	}
	delete(a.addrLookup, ka.ID())
}

//----------------------------------------------------------

func (a *addrBook) pickOldest(bucketType byte, bucketIdx int) *knownAddress {
	bucket := a.getBucket(bucketType, bucketIdx)
	var oldest *knownAddress
	for _, ka := range bucket {
		if oldest == nil || ka.LastAttempt.Before(oldest.LastAttempt) {
			oldest = ka
		}
	}
	return oldest
}

// adds the address to a "new" bucket. if its already in one,
// it only adds it probabilistically
func (a *addrBook) addAddress(addr, src *p2p.NetAddress) error {
	if addr == nil || src == nil {
		return ErrAddrBookNilAddr{addr, src}
	}

	if err := addr.Valid(); err != nil {
		return ErrAddrBookInvalidAddr{Addr: addr, AddrErr: err}
	}

	if _, ok := a.badPeers[addr.ID]; ok {
		return ErrAddressBanned{addr}
	}

	if _, ok := a.privateIDs[addr.ID]; ok {
		return ErrAddrBookPrivate{addr}
	}

	if _, ok := a.privateIDs[src.ID]; ok {
		return ErrAddrBookPrivateSrc{src}
	}

	// TODO: we should track ourAddrs by ID and by IP:PORT and refuse both.
	if _, ok := a.ourAddrs[addr.String()]; ok {
		return ErrAddrBookSelf{addr}
	}

	if a.routabilityStrict && !addr.Routable() {
		return ErrAddrBookNonRoutable{addr}
	}

	ka := a.addrLookup[addr.ID]
	if ka != nil {
		// If its already old and the address ID's are the same, ignore it.
		// Thereby avoiding issues with a node on the network attempting to change
		// the IP of a known node ID. (Which could yield an eclipse attack on the node)
		if ka.isOld() && ka.Addr.ID == addr.ID {
			return nil
		}
		// Already in max new buckets.
		if len(ka.Buckets) == maxNewBucketsPerAddress {
			return nil
		}
		// The more entries we have, the less likely we are to add more.
		factor := int32(2 * len(ka.Buckets))
		if a.rand.Int31n(factor) != 0 {
			return nil
		}
	} else {
		ka = newKnownAddress(addr, src)
	}

	bucket, err := a.calcNewBucket(addr, src)
	if err != nil {
		return err
	}
	return a.addToNewBucket(ka, bucket)
}

func (a *addrBook) randomPickAddresses(bucketType byte, num int) []*p2p.NetAddress {
	var buckets []map[string]*knownAddress
	switch bucketType {
	case bucketTypeNew:
		buckets = a.bucketsNew
	case bucketTypeOld:
		buckets = a.bucketsOld
	default:
		panic("unexpected bucketType")
	}
	total := 0
	for _, bucket := range buckets {
		total += len(bucket)
	}
	addresses := make([]*knownAddress, 0, total)
	for _, bucket := range buckets {
		for _, ka := range bucket {
			addresses = append(addresses, ka)
		}
	}
	selection := make([]*p2p.NetAddress, 0, num)
	chosenSet := make(map[string]bool, num)
	rand.Shuffle(total, func(i, j int) {
		addresses[i], addresses[j] = addresses[j], addresses[i]
	})
	for _, addr := range addresses {
		if chosenSet[addr.Addr.String()] {
			continue
		}
		chosenSet[addr.Addr.String()] = true
		selection = append(selection, addr.Addr)
		if len(selection) >= num {
			return selection
		}
	}
	return selection
}

// Make space in the new buckets by expiring the really bad entries.
// If no bad entries are available we remove the oldest.
func (a *addrBook) expireNew(bucketIdx int) {
	for addrStr, ka := range a.bucketsNew[bucketIdx] {
		// If an entry is bad, throw it away
		if ka.isBad() {
			a.Logger.Info("expire new", "msg", log.NewLazySprintf("expiring bad address %v", addrStr))
			a.removeFromBucket(ka, bucketTypeNew, bucketIdx)
			return
		}
	}

	// If we haven't thrown out a bad entry, throw out the oldest entry
	oldest := a.pickOldest(bucketTypeNew, bucketIdx)
	a.removeFromBucket(oldest, bucketTypeNew, bucketIdx)
}

// Promotes an address from new to old. If the destination bucket is full,
// demote the oldest one to a "new" bucket.
// TODO: Demote more probabilistically?
func (a *addrBook) moveToOld(ka *knownAddress) error {
	// Sanity check
	if ka.isOld() {
		a.Logger.Error(fmt.Sprintf("Cannot promote address that is already old %v", ka))
		return nil
	}
	if len(ka.Buckets) == 0 {
		a.Logger.Error(fmt.Sprintf("Cannot promote address that isn't in any new buckets %v", ka))
		return nil
	}

	// Remove from all (new) buckets.
	a.removeFromAllBuckets(ka)
	// It's officially old now.
	ka.BucketType = bucketTypeOld

	// Try to add it to its oldBucket destination.
	oldBucketIdx, err := a.calcOldBucket(ka.Addr)
	if err != nil {
		return err
	}
	added := a.addToOldBucket(ka, oldBucketIdx)
	if !added {
		// No room; move the oldest to a new bucket
		oldest := a.pickOldest(bucketTypeOld, oldBucketIdx)
		a.removeFromBucket(oldest, bucketTypeOld, oldBucketIdx)
		newBucketIdx, err := a.calcNewBucket(oldest.Addr, oldest.Src)
		if err != nil {
			return err
		}
		if err := a.addToNewBucket(oldest, newBucketIdx); err != nil {
			a.Logger.Error("Error adding peer to old bucket", "err", err)
		}

		// Finally, add our ka to old bucket again.
		added = a.addToOldBucket(ka, oldBucketIdx)
		if !added {
			a.Logger.Error(fmt.Sprintf("Could not re-add ka %v to oldBucketIdx %v", ka, oldBucketIdx))
		}
	}
	return nil
}

func (a *addrBook) removeAddress(addr *p2p.NetAddress) {
	ka := a.addrLookup[addr.ID]
	if ka == nil {
		return
	}
	a.Logger.Info("Remove address from book", "addr", addr)
	a.removeFromAllBuckets(ka)
}

func (a *addrBook) addBadPeer(addr *p2p.NetAddress, banTime time.Duration) bool {
	// check it exists in addrbook
	ka := a.addrLookup[addr.ID]
	// check address is not already there
	if ka == nil {
		return false
	}

	if _, alreadyBadPeer := a.badPeers[addr.ID]; !alreadyBadPeer {
		// add to bad peer list
		ka.ban(banTime)
		a.badPeers[addr.ID] = ka
		a.Logger.Info("Add address to blacklist", "addr", addr)
	}
	return true
}

//---------------------------------------------------------------------
// calculate bucket placements

// hash(key + sourcegroup + int64(hash(key + group + sourcegroup)) % bucket_per_group) % num_new_buckets
func (a *addrBook) calcNewBucket(addr, src *p2p.NetAddress) (int, error) {
	data1 := []byte{}
	data1 = append(data1, []byte(a.key)...)
	data1 = append(data1, []byte(a.groupKey(addr))...)
	data1 = append(data1, []byte(a.groupKey(src))...)
	hash1, err := a.hash(data1)
	if err != nil {
		return 0, err
	}
	hash64 := binary.BigEndian.Uint64(hash1)
	hash64 %= newBucketsPerGroup
	var hashbuf [8]byte
	binary.BigEndian.PutUint64(hashbuf[:], hash64)
	data2 := []byte{}
	data2 = append(data2, []byte(a.key)...)
	data2 = append(data2, a.groupKey(src)...)
	data2 = append(data2, hashbuf[:]...)

	hash2, err := a.hash(data2)
	if err != nil {
		return 0, err
	}
	result := int(binary.BigEndian.Uint64(hash2) % newBucketCount)
	return result, nil
}

// hash(key + group + int64(hash(key + addr)) % buckets_per_group) % num_old_buckets
func (a *addrBook) calcOldBucket(addr *p2p.NetAddress) (int, error) {
	data1 := []byte{}
	data1 = append(data1, []byte(a.key)...)
	data1 = append(data1, []byte(addr.String())...)
	hash1, err := a.hash(data1)
	if err != nil {
		return 0, err
	}
	hash64 := binary.BigEndian.Uint64(hash1)
	hash64 %= oldBucketsPerGroup
	var hashbuf [8]byte
	binary.BigEndian.PutUint64(hashbuf[:], hash64)
	data2 := []byte{}
	data2 = append(data2, []byte(a.key)...)
	data2 = append(data2, a.groupKey(addr)...)
	data2 = append(data2, hashbuf[:]...)

	hash2, err := a.hash(data2)
	if err != nil {
		return 0, err
	}
	result := int(binary.BigEndian.Uint64(hash2) % oldBucketCount)
	return result, nil
}

// Return a string representing the network group of this address.
// This is the /16 for IPv4 (e.g. 1.2.0.0), the /32 (/36 for he.net) for IPv6, the string
// "local" for a local address and the string "unroutable" for an unroutable
// address.
func (a *addrBook) groupKey(na *p2p.NetAddress) string {
	return groupKeyFor(na, a.routabilityStrict)
}

func groupKeyFor(na *p2p.NetAddress, routabilityStrict bool) string {
	if routabilityStrict && na.Local() {
		return "local"
	}
	if routabilityStrict && !na.Routable() {
		return "unroutable"
	}

	if ipv4 := na.IP.To4(); ipv4 != nil {
		return na.IP.Mask(net.CIDRMask(16, 32)).String()
	}

	if na.RFC6145() || na.RFC6052() {
		// last four bytes are the ip address
		ip := na.IP[12:16]
		return ip.Mask(net.CIDRMask(16, 32)).String()
	}

	if na.RFC3964() {
		ip := na.IP[2:6]
		return ip.Mask(net.CIDRMask(16, 32)).String()
	}

	if na.RFC4380() {
		// teredo tunnels have the last 4 bytes as the v4 address XOR
		// 0xff.
		ip := net.IP(make([]byte, 4))
		for i, byte := range na.IP[12:16] {
			ip[i] = byte ^ 0xff
		}
		return ip.Mask(net.CIDRMask(16, 32)).String()
	}

	if na.OnionCatTor() {
		// group is keyed off the first 4 bits of the actual onion key.
		return fmt.Sprintf("tor:%d", na.IP[6]&((1<<4)-1))
	}

	// OK, so now we know ourselves to be a IPv6 address.
	// bitcoind uses /32 for everything, except for Hurricane Electric's
	// (he.net) IP range, which it uses /36 for.
	bits := 32
	heNet := &net.IPNet{IP: net.ParseIP("2001:470::"), Mask: net.CIDRMask(32, 128)}
	if heNet.Contains(na.IP) {
		bits = 36
	}
	ipv6Mask := net.CIDRMask(bits, 128)
	return na.IP.Mask(ipv6Mask).String()
}

func (a *addrBook) hash(b []byte) ([]byte, error) {
	a.hasher.Reset()
	a.hasher.Write(b)
	return a.hasher.Sum(nil), nil
}
