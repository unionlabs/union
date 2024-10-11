package dnssrv

import "net"

// Lookup is a function that resolves a DNS SRV record to multiple addresses.
// It has the same signature as net.LookupSRV.
type Lookup func(service, proto, name string) (cname string, addrs []*net.SRV, err error)
