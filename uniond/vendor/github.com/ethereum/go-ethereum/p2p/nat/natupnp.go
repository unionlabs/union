// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package nat

import (
	"errors"
	"fmt"
	"math"
	"math/rand"
	"net"
	"strings"
	"sync"
	"time"

	"github.com/huin/goupnp"
	"github.com/huin/goupnp/dcps/internetgateway1"
	"github.com/huin/goupnp/dcps/internetgateway2"
)

const (
	soapRequestTimeout = 3 * time.Second
	rateLimit          = 200 * time.Millisecond
)

type upnp struct {
	dev         *goupnp.RootDevice
	service     string
	client      upnpClient
	mu          sync.Mutex
	lastReqTime time.Time
	rand        *rand.Rand
}

type upnpClient interface {
	GetExternalIPAddress() (string, error)
	AddPortMapping(string, uint16, string, uint16, string, bool, string, uint32) error
	DeletePortMapping(string, uint16, string) error
	GetNATRSIPStatus() (sip bool, nat bool, err error)
}

func (n *upnp) natEnabled() bool {
	var ok bool
	var err error
	n.withRateLimit(func() error {
		_, ok, err = n.client.GetNATRSIPStatus()
		return err
	})
	return err == nil && ok
}

func (n *upnp) ExternalIP() (addr net.IP, err error) {
	var ipString string
	n.withRateLimit(func() error {
		ipString, err = n.client.GetExternalIPAddress()
		return err
	})

	if err != nil {
		return nil, err
	}
	ip := net.ParseIP(ipString)
	if ip == nil {
		return nil, errors.New("bad IP in response")
	}
	return ip, nil
}

func (n *upnp) AddMapping(protocol string, extport, intport int, desc string, lifetime time.Duration) (uint16, error) {
	ip, err := n.internalAddress()
	if err != nil {
		return 0, nil // TODO: Shouldn't we return the error?
	}
	protocol = strings.ToUpper(protocol)
	lifetimeS := uint32(lifetime / time.Second)
	n.DeleteMapping(protocol, extport, intport)

	err = n.withRateLimit(func() error {
		return n.client.AddPortMapping("", uint16(extport), protocol, uint16(intport), ip.String(), true, desc, lifetimeS)
	})
	if err == nil {
		return uint16(extport), nil
	}

	return uint16(extport), n.withRateLimit(func() error {
		p, err := n.addAnyPortMapping(protocol, extport, intport, ip, desc, lifetimeS)
		if err == nil {
			extport = int(p)
		}
		return err
	})
}

func (n *upnp) addAnyPortMapping(protocol string, extport, intport int, ip net.IP, desc string, lifetimeS uint32) (uint16, error) {
	if client, ok := n.client.(*internetgateway2.WANIPConnection2); ok {
		return client.AddAnyPortMapping("", uint16(extport), protocol, uint16(intport), ip.String(), true, desc, lifetimeS)
	}
	// It will retry with a random port number if the client does
	// not support AddAnyPortMapping.
	extport = n.randomPort()
	err := n.client.AddPortMapping("", uint16(extport), protocol, uint16(intport), ip.String(), true, desc, lifetimeS)
	if err != nil {
		return 0, err
	}
	return uint16(extport), nil
}

func (n *upnp) randomPort() int {
	if n.rand == nil {
		n.rand = rand.New(rand.NewSource(time.Now().UnixNano()))
	}
	return n.rand.Intn(math.MaxUint16-10000) + 10000
}

func (n *upnp) internalAddress() (net.IP, error) {
	devaddr, err := net.ResolveUDPAddr("udp4", n.dev.URLBase.Host)
	if err != nil {
		return nil, err
	}
	ifaces, err := net.Interfaces()
	if err != nil {
		return nil, err
	}
	for _, iface := range ifaces {
		addrs, err := iface.Addrs()
		if err != nil {
			return nil, err
		}
		for _, addr := range addrs {
			if x, ok := addr.(*net.IPNet); ok && x.Contains(devaddr.IP) {
				return x.IP, nil
			}
		}
	}
	return nil, fmt.Errorf("could not find local address in same net as %v", devaddr)
}

func (n *upnp) DeleteMapping(protocol string, extport, intport int) error {
	return n.withRateLimit(func() error {
		return n.client.DeletePortMapping("", uint16(extport), strings.ToUpper(protocol))
	})
}

func (n *upnp) String() string {
	return "UPNP " + n.service
}

func (n *upnp) withRateLimit(fn func() error) error {
	n.mu.Lock()
	defer n.mu.Unlock()

	lastreq := time.Since(n.lastReqTime)
	if lastreq < rateLimit {
		time.Sleep(rateLimit - lastreq)
	}
	err := fn()
	n.lastReqTime = time.Now()
	return err
}

// discoverUPnP searches for Internet Gateway Devices
// and returns the first one it can find on the local network.
func discoverUPnP() Interface {
	found := make(chan *upnp, 2)
	// IGDv1
	go discover(found, internetgateway1.URN_WANConnectionDevice_1, func(sc goupnp.ServiceClient) *upnp {
		switch sc.Service.ServiceType {
		case internetgateway1.URN_WANIPConnection_1:
			return &upnp{service: "IGDv1-IP1", client: &internetgateway1.WANIPConnection1{ServiceClient: sc}}
		case internetgateway1.URN_WANPPPConnection_1:
			return &upnp{service: "IGDv1-PPP1", client: &internetgateway1.WANPPPConnection1{ServiceClient: sc}}
		}
		return nil
	})
	// IGDv2
	go discover(found, internetgateway2.URN_WANConnectionDevice_2, func(sc goupnp.ServiceClient) *upnp {
		switch sc.Service.ServiceType {
		case internetgateway2.URN_WANIPConnection_1:
			return &upnp{service: "IGDv2-IP1", client: &internetgateway2.WANIPConnection1{ServiceClient: sc}}
		case internetgateway2.URN_WANIPConnection_2:
			return &upnp{service: "IGDv2-IP2", client: &internetgateway2.WANIPConnection2{ServiceClient: sc}}
		case internetgateway2.URN_WANPPPConnection_1:
			return &upnp{service: "IGDv2-PPP1", client: &internetgateway2.WANPPPConnection1{ServiceClient: sc}}
		}
		return nil
	})
	for i := 0; i < cap(found); i++ {
		if c := <-found; c != nil {
			return c
		}
	}
	return nil
}

// finds devices matching the given target and calls matcher for all
// advertised services of each device. The first non-nil service found
// is sent into out. If no service matched, nil is sent.
func discover(out chan<- *upnp, target string, matcher func(goupnp.ServiceClient) *upnp) {
	devs, err := goupnp.DiscoverDevices(target)
	if err != nil {
		out <- nil
		return
	}
	found := false
	for i := 0; i < len(devs) && !found; i++ {
		if devs[i].Root == nil {
			continue
		}
		devs[i].Root.Device.VisitServices(func(service *goupnp.Service) {
			if found {
				return
			}
			// check for a matching IGD service
			sc := goupnp.ServiceClient{
				SOAPClient: service.NewSOAPClient(),
				RootDevice: devs[i].Root,
				Location:   devs[i].Location,
				Service:    service,
			}
			sc.SOAPClient.HTTPClient.Timeout = soapRequestTimeout
			upnp := matcher(sc)
			if upnp == nil {
				return
			}
			upnp.dev = devs[i].Root

			// check whether port mapping is enabled
			if upnp.natEnabled() {
				out <- upnp
				found = true
			}
		})
	}
	if !found {
		out <- nil
	}
}
