package p2p

import (
	"bytes"
	"fmt"
	"net/http"
	"strings"

	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
)

// InfoHandler is a handler to serve /p2p page in metrics.
func (s *Service) InfoHandler(w http.ResponseWriter, _ *http.Request) {
	buf := new(bytes.Buffer)
	if _, err := fmt.Fprintf(buf, `bootnode=%s
self=%s

%d peers
%v
`,
		s.cfg.BootstrapNodeAddr,
		s.selfAddresses(),
		len(s.host.Network().Peers()),
		formatPeers(s.host), // Must be last. Writes one entry per row.
	); err != nil {
		log.WithError(err).Error("Failed to render p2p info page")
		return
	}

	w.WriteHeader(http.StatusOK)
	if _, err := w.Write(buf.Bytes()); err != nil {
		log.WithError(err).Error("Failed to render p2p info page")
	}
}

// selfAddresses formats the host data into dialable strings, comma separated.
func (s *Service) selfAddresses() string {
	var addresses []string
	if s.dv5Listener != nil {
		addresses = append(addresses, s.dv5Listener.Self().String())
	}
	for _, addr := range s.host.Addrs() {
		addresses = append(addresses, addr.String()+"/p2p/"+s.host.ID().String())
	}
	return strings.Join(addresses, ",")
}

// Format peer list to dialable addresses, separated by new line.
func formatPeers(h host.Host) string {
	var addresses []string

	for _, pid := range h.Network().Peers() {
		addresses = append(addresses, formatPeer(pid, h.Peerstore().PeerInfo(pid).Addrs))
	}
	return strings.Join(addresses, "\n")
}

// Format single peer info to dialable addresses, comma separated.
func formatPeer(pid peer.ID, ma []ma.Multiaddr) string {
	var addresses []string
	for _, a := range ma {
		addresses = append(addresses, a.String()+"/p2p/"+pid.String())
	}
	return strings.Join(addresses, ",")
}
