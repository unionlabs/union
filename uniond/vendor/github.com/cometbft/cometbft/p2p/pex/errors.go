package pex

import (
	"errors"
	"fmt"

	"github.com/cometbft/cometbft/p2p"
)

type ErrAddrBookNonRoutable struct {
	Addr *p2p.NetAddress
}

func (err ErrAddrBookNonRoutable) Error() string {
	return fmt.Sprintf("Cannot add non-routable address %v", err.Addr)
}

type errAddrBookOldAddressNewBucket struct {
	Addr     *p2p.NetAddress
	BucketID int
}

func (err errAddrBookOldAddressNewBucket) Error() string {
	return fmt.Sprintf("failed consistency check!"+
		" Cannot add pre-existing address %v into new bucket %v",
		err.Addr, err.BucketID)
}

type ErrAddrBookSelf struct {
	Addr *p2p.NetAddress
}

func (err ErrAddrBookSelf) Error() string {
	return fmt.Sprintf("Cannot add ourselves with address %v", err.Addr)
}

type ErrAddrBookPrivate struct {
	Addr *p2p.NetAddress
}

func (err ErrAddrBookPrivate) Error() string {
	return fmt.Sprintf("Cannot add private peer with address %v", err.Addr)
}

func (err ErrAddrBookPrivate) PrivateAddr() bool {
	return true
}

type ErrAddrBookPrivateSrc struct {
	Src *p2p.NetAddress
}

func (err ErrAddrBookPrivateSrc) Error() string {
	return fmt.Sprintf("Cannot add peer coming from private peer with address %v", err.Src)
}

func (err ErrAddrBookPrivateSrc) PrivateAddr() bool {
	return true
}

type ErrAddrBookNilAddr struct {
	Addr *p2p.NetAddress
	Src  *p2p.NetAddress
}

func (err ErrAddrBookNilAddr) Error() string {
	return fmt.Sprintf("Cannot add a nil address. Got (addr, src) = (%v, %v)", err.Addr, err.Src)
}

type ErrAddrBookInvalidAddr struct {
	Addr    *p2p.NetAddress
	AddrErr error
}

func (err ErrAddrBookInvalidAddr) Error() string {
	return fmt.Sprintf("Cannot add invalid address %v: %v", err.Addr, err.AddrErr)
}

// ErrAddressBanned is thrown when the address has been banned and therefore cannot be used
type ErrAddressBanned struct {
	Addr *p2p.NetAddress
}

func (err ErrAddressBanned) Error() string {
	return fmt.Sprintf("Address: %v is currently banned", err.Addr)
}

// ErrUnsolicitedList is thrown when a peer provides a list of addresses that have not been asked for.
var ErrUnsolicitedList = errors.New("unsolicited pexAddrsMessage")
