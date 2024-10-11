// Copyright 2023 The go-ethereum Authors
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

package p2p

import (
	"net"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/p2p/nat"
)

const (
	portMapDuration        = 10 * time.Minute
	portMapRefreshInterval = 8 * time.Minute
	portMapRetryInterval   = 5 * time.Minute
	extipRetryInterval     = 2 * time.Minute
)

type portMapping struct {
	protocol string
	name     string
	port     int

	// for use by the portMappingLoop goroutine:
	extPort  int // the mapped port returned by the NAT interface
	nextTime mclock.AbsTime
}

// setupPortMapping starts the port mapping loop if necessary.
// Note: this needs to be called after the LocalNode instance has been set on the server.
func (srv *Server) setupPortMapping() {
	// portMappingRegister will receive up to two values: one for the TCP port if
	// listening is enabled, and one more for enabling UDP port mapping if discovery is
	// enabled. We make it buffered to avoid blocking setup while a mapping request is in
	// progress.
	srv.portMappingRegister = make(chan *portMapping, 2)

	switch srv.NAT.(type) {
	case nil:
		// No NAT interface configured.
		srv.loopWG.Add(1)
		go srv.consumePortMappingRequests()

	case nat.ExtIP:
		// ExtIP doesn't block, set the IP right away.
		ip, _ := srv.NAT.ExternalIP()
		srv.localnode.SetStaticIP(ip)
		srv.loopWG.Add(1)
		go srv.consumePortMappingRequests()

	default:
		srv.loopWG.Add(1)
		go srv.portMappingLoop()
	}
}

func (srv *Server) consumePortMappingRequests() {
	defer srv.loopWG.Done()
	for {
		select {
		case <-srv.quit:
			return
		case <-srv.portMappingRegister:
		}
	}
}

// portMappingLoop manages port mappings for UDP and TCP.
func (srv *Server) portMappingLoop() {
	defer srv.loopWG.Done()

	newLogger := func(p string, e int, i int) log.Logger {
		return log.New("proto", p, "extport", e, "intport", i, "interface", srv.NAT)
	}

	var (
		mappings  = make(map[string]*portMapping, 2)
		refresh   = mclock.NewAlarm(srv.clock)
		extip     = mclock.NewAlarm(srv.clock)
		lastExtIP net.IP
	)
	extip.Schedule(srv.clock.Now())
	defer func() {
		refresh.Stop()
		extip.Stop()
		for _, m := range mappings {
			if m.extPort != 0 {
				log := newLogger(m.protocol, m.extPort, m.port)
				log.Debug("Deleting port mapping")
				srv.NAT.DeleteMapping(m.protocol, m.extPort, m.port)
			}
		}
	}()

	for {
		// Schedule refresh of existing mappings.
		for _, m := range mappings {
			refresh.Schedule(m.nextTime)
		}

		select {
		case <-srv.quit:
			return

		case <-extip.C():
			extip.Schedule(srv.clock.Now().Add(extipRetryInterval))
			ip, err := srv.NAT.ExternalIP()
			if err != nil {
				log.Debug("Couldn't get external IP", "err", err, "interface", srv.NAT)
			} else if !ip.Equal(lastExtIP) {
				log.Debug("External IP changed", "ip", extip, "interface", srv.NAT)
			} else {
				return
			}
			// Here, we either failed to get the external IP, or it has changed.
			lastExtIP = ip
			srv.localnode.SetStaticIP(ip)
			// Ensure port mappings are refreshed in case we have moved to a new network.
			for _, m := range mappings {
				m.nextTime = srv.clock.Now()
			}

		case m := <-srv.portMappingRegister:
			if m.protocol != "TCP" && m.protocol != "UDP" {
				panic("unknown NAT protocol name: " + m.protocol)
			}
			mappings[m.protocol] = m
			m.nextTime = srv.clock.Now()

		case <-refresh.C():
			for _, m := range mappings {
				if srv.clock.Now() < m.nextTime {
					continue
				}

				external := m.port
				if m.extPort != 0 {
					external = m.extPort
				}
				log := newLogger(m.protocol, external, m.port)

				log.Trace("Attempting port mapping")
				p, err := srv.NAT.AddMapping(m.protocol, external, m.port, m.name, portMapDuration)
				if err != nil {
					log.Debug("Couldn't add port mapping", "err", err)
					m.extPort = 0
					m.nextTime = srv.clock.Now().Add(portMapRetryInterval)
					continue
				}
				// It was mapped!
				m.extPort = int(p)
				m.nextTime = srv.clock.Now().Add(portMapRefreshInterval)
				if external != m.extPort {
					log = newLogger(m.protocol, m.extPort, m.port)
					log.Info("NAT mapped alternative port")
				} else {
					log.Info("NAT mapped port")
				}

				// Update port in local ENR.
				switch m.protocol {
				case "TCP":
					srv.localnode.Set(enr.TCP(m.extPort))
				case "UDP":
					srv.localnode.SetFallbackUDP(m.extPort)
				}
			}
		}
	}
}
