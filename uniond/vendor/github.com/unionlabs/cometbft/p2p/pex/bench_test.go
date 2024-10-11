package pex

import (
	"testing"

	"github.com/cometbft/cometbft/p2p"
)

func BenchmarkAddrBook_hash(b *testing.B) {
	book := &addrBook{
		ourAddrs:          make(map[string]struct{}),
		privateIDs:        make(map[p2p.ID]struct{}),
		addrLookup:        make(map[p2p.ID]*knownAddress),
		badPeers:          make(map[p2p.ID]*knownAddress),
		filePath:          "",
		routabilityStrict: true,
	}
	book.init()
	msg := []byte(`foobar`)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = book.hash(msg)
	}
}
