package p2p

import (
	"net"
	"runtime"

	"github.com/libp2p/go-libp2p/core/control"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/multiformats/go-multiaddr"
	manet "github.com/multiformats/go-multiaddr/net"
	"github.com/sirupsen/logrus"
)

const (
	// Limit for rate limiter when processing new inbound dials.
	ipLimit = 4

	// Burst limit for inbound dials.
	ipBurst = 8

	// High watermark buffer signifies the buffer till which
	// we will handle inbound requests.
	highWatermarkBuffer = 20
)

// InterceptPeerDial tests whether we're permitted to Dial the specified peer.
func (_ *Service) InterceptPeerDial(_ peer.ID) (allow bool) {
	return true
}

// InterceptAddrDial tests whether we're permitted to dial the specified
// multiaddr for the given peer.
func (s *Service) InterceptAddrDial(pid peer.ID, m multiaddr.Multiaddr) (allow bool) {
	// Disallow bad peers from dialing in.
	if s.peers.IsBad(pid) {
		return false
	}
	return filterConnections(s.addrFilter, m)
}

// InterceptAccept checks whether the incidental inbound connection is allowed.
func (s *Service) InterceptAccept(n network.ConnMultiaddrs) (allow bool) {
	// Deny all incoming connections before we are ready
	if !s.started {
		return false
	}
	if !s.validateDial(n.RemoteMultiaddr()) {
		// Allow other go-routines to run in the event
		// we receive a large amount of junk connections.
		runtime.Gosched()
		log.WithFields(logrus.Fields{"peer": n.RemoteMultiaddr(),
			"reason": "exceeded dial limit"}).Trace("Not accepting inbound dial from ip address")
		return false
	}
	if s.isPeerAtLimit(true /* inbound */) {
		log.WithFields(logrus.Fields{"peer": n.RemoteMultiaddr(),
			"reason": "at peer limit"}).Trace("Not accepting inbound dial")
		return false
	}
	return filterConnections(s.addrFilter, n.RemoteMultiaddr())
}

// InterceptSecured tests whether a given connection, now authenticated,
// is allowed.
func (_ *Service) InterceptSecured(_ network.Direction, _ peer.ID, _ network.ConnMultiaddrs) (allow bool) {
	return true
}

// InterceptUpgraded tests whether a fully capable connection is allowed.
func (_ *Service) InterceptUpgraded(_ network.Conn) (allow bool, reason control.DisconnectReason) {
	return true, 0
}

func (s *Service) validateDial(addr multiaddr.Multiaddr) bool {
	ip, err := manet.ToIP(addr)
	if err != nil {
		return false
	}
	remaining := s.ipLimiter.Remaining(ip.String())
	if remaining <= 0 {
		return false
	}
	s.ipLimiter.Add(ip.String(), 1)
	return true
}

var privateCIDRList = []string{
	// Private ip addresses specified by rfc-1918.
	// See: https://tools.ietf.org/html/rfc1918
	"10.0.0.0/8",
	"172.16.0.0/12",
	"192.168.0.0/16",
	// Reserved address space for CGN devices, specified by rfc-6598
	// See: https://tools.ietf.org/html/rfc6598
	"100.64.0.0/10",
	// IPv4 Link-Local addresses, specified by rfc-3926
	// See: https://tools.ietf.org/html/rfc3927
	"169.254.0.0/16",
}

// configureFilter looks at the provided allow lists and
// deny lists to appropriately create a filter.
func configureFilter(cfg *Config) (*multiaddr.Filters, error) {
	addrFilter := multiaddr.NewFilters()
	var privErr error
	switch {
	case cfg.AllowListCIDR == "public":
		cfg.DenyListCIDR = append(cfg.DenyListCIDR, privateCIDRList...)
	case cfg.AllowListCIDR == "private":
		addrFilter, privErr = privateCIDRFilter(addrFilter, multiaddr.ActionAccept)
		if privErr != nil {
			return nil, privErr
		}
	case cfg.AllowListCIDR != "":
		_, ipnet, err := net.ParseCIDR(cfg.AllowListCIDR)
		if err != nil {
			return nil, err
		}
		addrFilter.AddFilter(*ipnet, multiaddr.ActionAccept)
	}

	// Configure from provided deny list in the config.
	if len(cfg.DenyListCIDR) > 0 {
		for _, cidr := range cfg.DenyListCIDR {
			// If an entry in the deny list is "private", we iterate through the
			// private addresses and add them to the filter. Likewise, if the deny
			// list is "public", then we add all private address to the accept filter,
			switch {
			case cidr == "private":
				addrFilter, privErr = privateCIDRFilter(addrFilter, multiaddr.ActionDeny)
				if privErr != nil {
					return nil, privErr
				}
				continue
			case cidr == "public":
				addrFilter, privErr = privateCIDRFilter(addrFilter, multiaddr.ActionAccept)
				if privErr != nil {
					return nil, privErr
				}
				continue
			}
			_, ipnet, err := net.ParseCIDR(cidr)
			if err != nil {
				return nil, err
			}
			// Check if the address already has an action associated with it
			// If this address was previously accepted, log a warning before placing
			// it in the deny filter
			action, _ := addrFilter.ActionForFilter(*ipnet)
			if action == multiaddr.ActionAccept {
				log.Warnf("Address %s is in conflict with previous rule.", ipnet.String())
			}
			addrFilter.AddFilter(*ipnet, multiaddr.ActionDeny)
		}
	}
	return addrFilter, nil
}

// helper function to either accept or deny all private addresses
// if a new rule for a private address is in conflict with a previous one, log a warning
func privateCIDRFilter(addrFilter *multiaddr.Filters, action multiaddr.Action) (*multiaddr.Filters, error) {
	for _, privCidr := range privateCIDRList {
		_, ipnet, err := net.ParseCIDR(privCidr)
		if err != nil {
			return nil, err
		}
		// Get the current filter action for the address
		// If it conflicts with the action given by the function call,
		// log a warning
		curAction, _ := addrFilter.ActionForFilter(*ipnet)
		switch {
		case action == multiaddr.ActionAccept:
			if curAction == multiaddr.ActionDeny {
				log.Warnf("Address %s is in conflict with previous rule.", ipnet.String())
			}
		case action == multiaddr.ActionDeny:
			if curAction == multiaddr.ActionAccept {
				log.Warnf("Address %s is in conflict with previous rule.", ipnet.String())
			}
		}
		addrFilter.AddFilter(*ipnet, action)
	}
	return addrFilter, nil
}

// filterConnections checks the appropriate ip subnets from our
// filter and decides what to do with them. By default libp2p
// accepts all incoming dials, so if we have an allow list
// we will reject all inbound dials except for those in the
// appropriate ip subnets.
func filterConnections(f *multiaddr.Filters, a multiaddr.Multiaddr) bool {
	acceptedNets := f.FiltersForAction(multiaddr.ActionAccept)
	restrictConns := len(acceptedNets) != 0

	// If we have an allow list added in, we by default reject all
	// connection attempts except for those coming in from the
	// appropriate ip subnets.
	if restrictConns {
		ip, err := manet.ToIP(a)
		if err != nil {
			log.Tracef("Multiaddress has invalid ip: %v", err)
			return false
		}
		found := false
		for _, ipnet := range acceptedNets {
			if ipnet.Contains(ip) {
				found = true
				break
			}
		}
		return found
	}
	return !f.AddrBlocked(a)
}
