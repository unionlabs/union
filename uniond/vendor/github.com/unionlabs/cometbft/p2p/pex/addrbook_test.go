package pex

import (
	"encoding/hex"
	"fmt"
	"math"
	"net"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/log"
	cmtmath "github.com/cometbft/cometbft/libs/math"
	"github.com/cometbft/cometbft/p2p"
)

// FIXME These tests should not rely on .(*addrBook) assertions

func TestAddrBookPickAddress(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	// 0 addresses
	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	assert.Zero(t, book.Size())

	addr := book.PickAddress(50)
	assert.Nil(t, addr, "expected no address")

	randAddrs := randNetAddressPairs(t, 1)
	addrSrc := randAddrs[0]
	err := book.AddAddress(addrSrc.addr, addrSrc.src)
	require.NoError(t, err)

	// pick an address when we only have new address
	addr = book.PickAddress(0)
	assert.NotNil(t, addr, "expected an address")
	addr = book.PickAddress(50)
	assert.NotNil(t, addr, "expected an address")
	addr = book.PickAddress(100)
	assert.NotNil(t, addr, "expected an address")

	// pick an address when we only have old address
	book.MarkGood(addrSrc.addr.ID)
	addr = book.PickAddress(0)
	assert.NotNil(t, addr, "expected an address")
	addr = book.PickAddress(50)
	assert.NotNil(t, addr, "expected an address")

	// in this case, nNew==0 but we biased 100% to new, so we return nil
	addr = book.PickAddress(100)
	assert.Nil(t, addr, "did not expect an address")
}

func TestAddrBookSaveLoad(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	// 0 addresses
	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	book.Save()

	book = NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	err := book.Start()
	require.NoError(t, err)

	assert.True(t, book.Empty())

	// 100 addresses
	randAddrs := randNetAddressPairs(t, 100)

	for _, addrSrc := range randAddrs {
		err := book.AddAddress(addrSrc.addr, addrSrc.src)
		require.NoError(t, err)
	}

	assert.Equal(t, 100, book.Size())
	book.Save()

	book = NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	err = book.Start()
	require.NoError(t, err)

	assert.Equal(t, 100, book.Size())
}

func TestAddrBookLookup(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	randAddrs := randNetAddressPairs(t, 100)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	for _, addrSrc := range randAddrs {
		addr := addrSrc.addr
		src := addrSrc.src
		err := book.AddAddress(addr, src)
		require.NoError(t, err)

		ka := book.HasAddress(addr)
		assert.True(t, ka, "Expected to find KnownAddress %v but wasn't there.", addr)
	}
}

func TestAddrBookPromoteToOld(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	randAddrs := randNetAddressPairs(t, 100)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	for _, addrSrc := range randAddrs {
		err := book.AddAddress(addrSrc.addr, addrSrc.src)
		require.NoError(t, err)
	}

	// Attempt all addresses.
	for _, addrSrc := range randAddrs {
		book.MarkAttempt(addrSrc.addr)
	}

	// Promote half of them
	for i, addrSrc := range randAddrs {
		if i%2 == 0 {
			book.MarkGood(addrSrc.addr.ID)
		}
	}

	// TODO: do more testing :)

	selection := book.GetSelection()
	t.Logf("selection: %v", selection)

	if len(selection) > book.Size() {
		t.Errorf("selection could not be bigger than the book")
	}

	selection = book.GetSelectionWithBias(30)
	t.Logf("selection: %v", selection)

	if len(selection) > book.Size() {
		t.Errorf("selection with bias could not be bigger than the book")
	}

	assert.Equal(t, 100, book.Size(), "expecting book size to be 100")
}

func TestAddrBookHandlesDuplicates(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())

	randAddrs := randNetAddressPairs(t, 100)

	differentSrc := randIPv4Address(t)
	for _, addrSrc := range randAddrs {
		err := book.AddAddress(addrSrc.addr, addrSrc.src)
		require.NoError(t, err)
		err = book.AddAddress(addrSrc.addr, addrSrc.src) // duplicate
		require.NoError(t, err)
		err = book.AddAddress(addrSrc.addr, differentSrc) // different src
		require.NoError(t, err)
	}

	assert.Equal(t, 100, book.Size())
}

type netAddressPair struct {
	addr *p2p.NetAddress
	src  *p2p.NetAddress
}

func randNetAddressPairs(t *testing.T, n int) []netAddressPair {
	t.Helper()
	randAddrs := make([]netAddressPair, n)
	for i := 0; i < n; i++ {
		randAddrs[i] = netAddressPair{addr: randIPv4Address(t), src: randIPv4Address(t)}
	}
	return randAddrs
}

func randIPv4Address(t *testing.T) *p2p.NetAddress {
	t.Helper()
	for {
		ip := fmt.Sprintf("%v.%v.%v.%v",
			cmtrand.Intn(254)+1,
			cmtrand.Intn(255),
			cmtrand.Intn(255),
			cmtrand.Intn(255),
		)
		port := cmtrand.Intn(65535-1) + 1
		id := p2p.ID(hex.EncodeToString(cmtrand.Bytes(p2p.IDByteLength)))
		idAddr := p2p.IDAddressString(id, fmt.Sprintf("%v:%v", ip, port))
		addr, err := p2p.NewNetAddressString(idAddr)
		require.NoError(t, err)
		if addr.Routable() {
			return addr
		}
	}
}

func TestAddrBookRemoveAddress(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())

	addr := randIPv4Address(t)
	err := book.AddAddress(addr, addr)
	require.NoError(t, err)
	assert.Equal(t, 1, book.Size())

	book.RemoveAddress(addr)
	assert.Equal(t, 0, book.Size())

	nonExistingAddr := randIPv4Address(t)
	book.RemoveAddress(nonExistingAddr)
	assert.Equal(t, 0, book.Size())
}

func TestAddrBookGetSelectionWithOneMarkedGood(t *testing.T) {
	// create a book with 10 addresses, 1 good/old and 9 new
	book, fname := createAddrBookWithMOldAndNNewAddrs(t, 1, 9)
	defer deleteTempFile(fname)

	addrs := book.GetSelectionWithBias(biasToSelectNewPeers)
	assert.NotNil(t, addrs)
	assertMOldAndNNewAddrsInSelection(t, 1, 9, addrs, book)
}

func TestAddrBookGetSelectionWithOneNotMarkedGood(t *testing.T) {
	// create a book with 10 addresses, 9 good/old and 1 new
	book, fname := createAddrBookWithMOldAndNNewAddrs(t, 9, 1)
	defer deleteTempFile(fname)

	addrs := book.GetSelectionWithBias(biasToSelectNewPeers)
	assert.NotNil(t, addrs)
	assertMOldAndNNewAddrsInSelection(t, 9, 1, addrs, book)
}

func TestAddrBookGetSelectionReturnsNilWhenAddrBookIsEmpty(t *testing.T) {
	book, fname := createAddrBookWithMOldAndNNewAddrs(t, 0, 0)
	defer deleteTempFile(fname)

	addrs := book.GetSelectionWithBias(biasToSelectNewPeers)
	assert.Nil(t, addrs)
}

func TestAddrBookGetSelection(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())

	// 1) empty book
	assert.Empty(t, book.GetSelection())

	// 2) add one address
	addr := randIPv4Address(t)
	err := book.AddAddress(addr, addr)
	require.NoError(t, err)

	assert.Len(t, book.GetSelection(), 1)
	assert.Equal(t, addr, book.GetSelection()[0])

	// 3) add a bunch of addresses
	randAddrs := randNetAddressPairs(t, 100)
	for _, addrSrc := range randAddrs {
		err := book.AddAddress(addrSrc.addr, addrSrc.src)
		require.NoError(t, err)
	}

	// check there is no duplicates
	addrs := make(map[string]*p2p.NetAddress)
	selection := book.GetSelection()
	for _, addr := range selection {
		if dup, ok := addrs[addr.String()]; ok {
			t.Fatalf("selection %v contains duplicates %v", selection, dup)
		}
		addrs[addr.String()] = addr
	}

	if len(selection) > book.Size() {
		t.Errorf("selection %v could not be bigger than the book", selection)
	}
}

func TestAddrBookGetSelectionWithBias(t *testing.T) {
	const biasTowardsNewAddrs = 30

	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())

	// 1) empty book
	selection := book.GetSelectionWithBias(biasTowardsNewAddrs)
	assert.Empty(t, selection)

	// 2) add one address
	addr := randIPv4Address(t)
	err := book.AddAddress(addr, addr)
	require.NoError(t, err)

	selection = book.GetSelectionWithBias(biasTowardsNewAddrs)
	assert.Len(t, selection, 1)
	assert.Equal(t, addr, selection[0])

	// 3) add a bunch of addresses
	randAddrs := randNetAddressPairs(t, 100)
	for _, addrSrc := range randAddrs {
		err := book.AddAddress(addrSrc.addr, addrSrc.src)
		require.NoError(t, err)
	}

	// check there is no duplicates
	addrs := make(map[string]*p2p.NetAddress)
	selection = book.GetSelectionWithBias(biasTowardsNewAddrs)
	for _, addr := range selection {
		if dup, ok := addrs[addr.String()]; ok {
			t.Fatalf("selection %v contains duplicates %v", selection, dup)
		}
		addrs[addr.String()] = addr
	}

	if len(selection) > book.Size() {
		t.Fatalf("selection %v could not be bigger than the book", selection)
	}

	// 4) mark 80% of the addresses as good
	randAddrsLen := len(randAddrs)
	for i, addrSrc := range randAddrs {
		if int((float64(i)/float64(randAddrsLen))*100) >= 20 {
			book.MarkGood(addrSrc.addr.ID)
		}
	}

	selection = book.GetSelectionWithBias(biasTowardsNewAddrs)

	// check that ~70% of addresses returned are good
	good := 0
	for _, addr := range selection {
		if book.IsGood(addr) {
			good++
		}
	}

	got, expected := int((float64(good)/float64(len(selection)))*100), 100-biasTowardsNewAddrs

	// compute some slack to protect against small differences due to rounding:
	slack := int(math.Round(float64(100) / float64(len(selection))))
	if got > expected+slack {
		t.Fatalf(
			"got more good peers (%% got: %d, %% expected: %d, number of good addrs: %d, total: %d)",
			got,
			expected,
			good,
			len(selection),
		)
	}
	if got < expected-slack {
		t.Fatalf(
			"got fewer good peers (%% got: %d, %% expected: %d, number of good addrs: %d, total: %d)",
			got,
			expected,
			good,
			len(selection),
		)
	}
}

func TestAddrBookHasAddress(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	addr := randIPv4Address(t)
	err := book.AddAddress(addr, addr)
	require.NoError(t, err)

	assert.True(t, book.HasAddress(addr))

	book.RemoveAddress(addr)

	assert.False(t, book.HasAddress(addr))
}

func testCreatePrivateAddrs(t *testing.T, numAddrs int) ([]*p2p.NetAddress, []string) {
	t.Helper()
	addrs := make([]*p2p.NetAddress, numAddrs)
	for i := 0; i < numAddrs; i++ {
		addrs[i] = randIPv4Address(t)
	}

	private := make([]string, numAddrs)
	for i, addr := range addrs {
		private[i] = string(addr.ID)
	}
	return addrs, private
}

func TestBanBadPeers(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())

	addr := randIPv4Address(t)
	_ = book.AddAddress(addr, addr)

	book.MarkBad(addr, 1*time.Second)
	// addr should not reachable
	assert.False(t, book.HasAddress(addr))
	assert.True(t, book.IsBanned(addr))

	err := book.AddAddress(addr, addr)
	// book should not add address from the blacklist
	require.Error(t, err)

	time.Sleep(1 * time.Second)
	book.ReinstateBadPeers()
	// address should be reinstated in the new bucket
	assert.EqualValues(t, 1, book.Size())
	assert.True(t, book.HasAddress(addr))
	assert.False(t, book.IsGood(addr))
}

func TestAddrBookEmpty(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	// Check that empty book is empty
	require.True(t, book.Empty())
	// Check that book with our address is empty
	book.AddOurAddress(randIPv4Address(t))
	require.True(t, book.Empty())
	// Check that book with private addrs is empty
	_, privateIDs := testCreatePrivateAddrs(t, 5)
	book.AddPrivateIDs(privateIDs)
	require.True(t, book.Empty())

	// Check that book with address is not empty
	err := book.AddAddress(randIPv4Address(t), randIPv4Address(t))
	require.NoError(t, err)
	require.False(t, book.Empty())
}

func TestPrivatePeers(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())

	addrs, private := testCreatePrivateAddrs(t, 10)
	book.AddPrivateIDs(private)

	// private addrs must not be added
	for _, addr := range addrs {
		err := book.AddAddress(addr, addr)
		if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
			_, ok := err.(ErrAddrBookPrivate)
			assert.True(t, ok)
		}
	}

	// addrs coming from private peers must not be added
	err := book.AddAddress(randIPv4Address(t), addrs[0])
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		_, ok := err.(ErrAddrBookPrivateSrc)
		assert.True(t, ok)
	}
}

func testAddrBookAddressSelection(t *testing.T, bookSize int) {
	t.Helper()
	// generate all combinations of old (m) and new addresses
	for nBookOld := 0; nBookOld <= bookSize; nBookOld++ {
		nBookNew := bookSize - nBookOld
		dbgStr := fmt.Sprintf("book of size %d (new %d, old %d)", bookSize, nBookNew, nBookOld)

		// create book and get selection
		book, fname := createAddrBookWithMOldAndNNewAddrs(t, nBookOld, nBookNew)
		defer deleteTempFile(fname)
		addrs := book.GetSelectionWithBias(biasToSelectNewPeers)
		assert.NotNil(t, addrs, "%s - expected a non-nil selection", dbgStr)
		nAddrs := len(addrs)
		assert.NotZero(t, nAddrs, "%s - expected at least one address in selection", dbgStr)

		// check there's no nil addresses
		for _, addr := range addrs {
			if addr == nil {
				t.Fatalf("%s - got nil address in selection %v", dbgStr, addrs)
			}
		}

		// XXX: shadowing
		nOld, nNew := countOldAndNewAddrsInSelection(addrs, book)

		// Given:
		// n - num new addrs, m - num old addrs
		// k - num new addrs expected in the beginning (based on bias %)
		// i=min(n, max(k,r-m)), aka expNew
		// j=min(m, r-i), aka expOld
		//
		// We expect this layout:
		// indices:      0...i-1   i...i+j-1
		// addresses:    N0..Ni-1  O0..Oj-1
		//
		// There is at least one partition and at most three.
		var (
			k      = percentageOfNum(biasToSelectNewPeers, nAddrs)
			expNew = cmtmath.MinInt(nNew, cmtmath.MaxInt(k, nAddrs-nBookOld))
			expOld = cmtmath.MinInt(nOld, nAddrs-expNew)
		)

		// Verify that the number of old and new addresses are as expected
		if nNew != expNew {
			t.Fatalf("%s - expected new addrs %d, got %d", dbgStr, expNew, nNew)
		}
		if nOld != expOld {
			t.Fatalf("%s - expected old addrs %d, got %d", dbgStr, expOld, nOld)
		}

		// Verify that the order of addresses is as expected
		// Get the sequence types and lengths of the selection
		seqLens, seqTypes := analyseSelectionLayout(book, addrs)

		// Build a list with the expected lengths of partitions and another with the expected types, e.g.:
		// expSeqLens = [10, 22], expSeqTypes = [1, 2]
		// means we expect 10 new (type 1) addresses followed by 22 old (type 2) addresses.
		var expSeqLens []int
		var expSeqTypes []int

		switch {
		case expOld == 0: // all new addresses
			expSeqLens = []int{nAddrs}
			expSeqTypes = []int{1}
		case expNew == 0: // all old addresses
			expSeqLens = []int{nAddrs}
			expSeqTypes = []int{2}
		case nAddrs-expNew-expOld == 0: // new addresses, old addresses
			expSeqLens = []int{expNew, expOld}
			expSeqTypes = []int{1, 2}
		}

		assert.Equal(t, expSeqLens, seqLens,
			"%s - expected sequence lengths of old/new %v, got %v",
			dbgStr, expSeqLens, seqLens)
		assert.Equal(t, expSeqTypes, seqTypes,
			"%s - expected sequence types (1-new, 2-old) was %v, got %v",
			dbgStr, expSeqTypes, seqTypes)
	}
}

func TestMultipleAddrBookAddressSelection(t *testing.T) {
	// test books with smaller size, < N
	const n = 32
	for bookSize := 1; bookSize < n; bookSize++ {
		testAddrBookAddressSelection(t, bookSize)
	}

	// Test for two books with sizes from following ranges
	ranges := [...][]int{{33, 100}, {100, 175}}
	bookSizes := make([]int, 0, len(ranges))
	for _, r := range ranges {
		bookSizes = append(bookSizes, cmtrand.Intn(r[1]-r[0])+r[0])
	}
	t.Logf("Testing address selection for the following book sizes %v\n", bookSizes)
	for _, bookSize := range bookSizes {
		testAddrBookAddressSelection(t, bookSize)
	}
}

func TestAddrBookAddDoesNotOverwriteOldIP(t *testing.T) {
	fname := createTempFileName()
	defer deleteTempFile(fname)

	// This test creates adds a peer to the address book and marks it good
	// It then attempts to override the peer's IP, by adding a peer with the same ID
	// but different IP. We distinguish the IP's by "RealIP" and "OverrideAttemptIP"
	peerID := "678503e6c8f50db7279c7da3cb9b072aac4bc0d5"
	peerRealIP := "1.1.1.1:26656"
	peerOverrideAttemptIP := "2.2.2.2:26656"
	srcAddr := "b0dd378c3fbc4c156cd6d302a799f0d2e4227201@159.89.121.174:26656"

	// There is a chance that AddAddress will ignore the new peer its given.
	// So we repeat trying to override the peer several times,
	// to ensure we aren't in a case that got probabilistically ignored
	numOverrideAttempts := 10

	peerRealAddr, err := p2p.NewNetAddressString(peerID + "@" + peerRealIP)
	require.NoError(t, err)

	peerOverrideAttemptAddr, err := p2p.NewNetAddressString(peerID + "@" + peerOverrideAttemptIP)
	require.NoError(t, err)

	src, err := p2p.NewNetAddressString(srcAddr)
	require.NoError(t, err)

	book := NewAddrBook(fname, true)
	book.SetLogger(log.TestingLogger())
	err = book.AddAddress(peerRealAddr, src)
	require.NoError(t, err)
	book.MarkAttempt(peerRealAddr)
	book.MarkGood(peerRealAddr.ID)

	// Double check that adding a peer again doesn't error
	err = book.AddAddress(peerRealAddr, src)
	require.NoError(t, err)

	// Try changing ip but keeping the same node id. (change 1.1.1.1 to 2.2.2.2)
	// This should just be ignored, and not error.
	for i := 0; i < numOverrideAttempts; i++ {
		err = book.AddAddress(peerOverrideAttemptAddr, src)
		require.NoError(t, err)
	}
	// Now check that the IP was not overridden.
	// This is done by sampling several peers from addr book
	// and ensuring they all have the correct IP.
	// In the expected functionality, this test should only have 1 Peer, hence will pass.
	for i := 0; i < numOverrideAttempts; i++ {
		selection := book.GetSelection()
		for _, addr := range selection {
			require.Equal(t, addr.IP, peerRealAddr.IP)
		}
	}
}

func TestAddrBookGroupKey(t *testing.T) {
	// non-strict routability
	testCases := []struct {
		name   string
		ip     string
		expKey string
	}{
		// IPv4 normal.
		{"ipv4 normal class a", "12.1.2.3", "12.1.0.0"},
		{"ipv4 normal class b", "173.1.2.3", "173.1.0.0"},
		{"ipv4 normal class c", "196.1.2.3", "196.1.0.0"},

		// IPv6/IPv4 translations.
		{"ipv6 rfc3964 with ipv4 encap", "2002:0c01:0203::", "12.1.0.0"},
		{"ipv6 rfc4380 toredo ipv4", "2001:0:1234::f3fe:fdfc", "12.1.0.0"},
		{"ipv6 rfc6052 well-known prefix with ipv4", "64:ff9b::0c01:0203", "12.1.0.0"},
		{"ipv6 rfc6145 translated ipv4", "::ffff:0:0c01:0203", "12.1.0.0"},

		// Tor.
		{"ipv6 tor onioncat", "fd87:d87e:eb43:1234::5678", "tor:2"},
		{"ipv6 tor onioncat 2", "fd87:d87e:eb43:1245::6789", "tor:2"},
		{"ipv6 tor onioncat 3", "fd87:d87e:eb43:1345::6789", "tor:3"},

		// IPv6 normal.
		{"ipv6 normal", "2602:100::1", "2602:100::"},
		{"ipv6 normal 2", "2602:0100::1234", "2602:100::"},
		{"ipv6 hurricane electric", "2001:470:1f10:a1::2", "2001:470:1000::"},
		{"ipv6 hurricane electric 2", "2001:0470:1f10:a1::2", "2001:470:1000::"},
	}

	for i, tc := range testCases {
		nip := net.ParseIP(tc.ip)
		key := groupKeyFor(p2p.NewNetAddressIPPort(nip, 26656), false)
		assert.Equal(t, tc.expKey, key, "#%d", i)
	}

	// strict routability
	testCases = []struct {
		name   string
		ip     string
		expKey string
	}{
		// Local addresses.
		{"ipv4 localhost", "127.0.0.1", "local"},
		{"ipv6 localhost", "::1", "local"},
		{"ipv4 zero", "0.0.0.0", "local"},
		{"ipv4 first octet zero", "0.1.2.3", "local"},

		// Unroutable addresses.
		{"ipv4 invalid bcast", "255.255.255.255", "unroutable"},
		{"ipv4 rfc1918 10/8", "10.1.2.3", "unroutable"},
		{"ipv4 rfc1918 172.16/12", "172.16.1.2", "unroutable"},
		{"ipv4 rfc1918 192.168/16", "192.168.1.2", "unroutable"},
		{"ipv6 rfc3849 2001:db8::/32", "2001:db8::1234", "unroutable"},
		{"ipv4 rfc3927 169.254/16", "169.254.1.2", "unroutable"},
		{"ipv6 rfc4193 fc00::/7", "fc00::1234", "unroutable"},
		{"ipv6 rfc4843 2001:10::/28", "2001:10::1234", "unroutable"},
		{"ipv6 rfc4862 fe80::/64", "fe80::1234", "unroutable"},
	}

	for i, tc := range testCases {
		nip := net.ParseIP(tc.ip)
		key := groupKeyFor(p2p.NewNetAddressIPPort(nip, 26656), true)
		assert.Equal(t, tc.expKey, key, "#%d", i)
	}
}

func assertMOldAndNNewAddrsInSelection(t *testing.T, m, n int, addrs []*p2p.NetAddress, book *addrBook) {
	t.Helper()
	nOld, nNew := countOldAndNewAddrsInSelection(addrs, book)
	assert.Equal(t, m, nOld, "old addresses")
	assert.Equal(t, n, nNew, "new addresses")
}

func createTempFileName() string {
	prefix := "addrbook_test"
	f, err := os.CreateTemp("", prefix)
	if err != nil {
		panic(err)
	}
	fname := f.Name()
	err = f.Close()
	if err != nil {
		panic(err)
	}
	return fname
}

func deleteTempFile(fname string) {
	err := os.Remove(fname)
	if err != nil {
		panic(err)
	}
}

func createAddrBookWithMOldAndNNewAddrs(t *testing.T, nOld, nNew int) (book *addrBook, fname string) {
	t.Helper()
	fname = createTempFileName()

	book = NewAddrBook(fname, true).(*addrBook)
	book.SetLogger(log.TestingLogger())
	assert.Zero(t, book.Size())

	randAddrs := randNetAddressPairs(t, nOld)
	for _, addr := range randAddrs {
		err := book.AddAddress(addr.addr, addr.src)
		require.NoError(t, err)
		book.MarkGood(addr.addr.ID)
	}

	randAddrs = randNetAddressPairs(t, nNew)
	for _, addr := range randAddrs {
		err := book.AddAddress(addr.addr, addr.src)
		require.NoError(t, err)
	}

	return book, fname
}

func countOldAndNewAddrsInSelection(addrs []*p2p.NetAddress, book *addrBook) (nOld, nNew int) {
	for _, addr := range addrs {
		if book.IsGood(addr) {
			nOld++
		} else {
			nNew++
		}
	}
	return nOld, nNew
}

// Analyze the layout of the selection specified by 'addrs'
// Returns:
// - seqLens - the lengths of the sequences of addresses of same type
// - seqTypes - the types of sequences in selection.
func analyseSelectionLayout(book *addrBook, addrs []*p2p.NetAddress) (seqLens, seqTypes []int) {
	// address types are: 0 - nil, 1 - new, 2 - old
	var (
		prevType      = 0
		currentSeqLen = 0
	)

	for _, addr := range addrs {
		var addrType int
		if book.IsGood(addr) {
			addrType = 2
		} else {
			addrType = 1
		}
		if addrType != prevType && prevType != 0 {
			seqLens = append(seqLens, currentSeqLen)
			seqTypes = append(seqTypes, prevType)
			currentSeqLen = 0
		}
		currentSeqLen++
		prevType = addrType
	}

	seqLens = append(seqLens, currentSeqLen)
	seqTypes = append(seqTypes, prevType)

	return seqLens, seqTypes
}
