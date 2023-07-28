package p2p

import (
	"fmt"
	"net"
)

// ErrFilterTimeout indicates that a filter operation timed out.
type ErrFilterTimeout struct{}

func (e ErrFilterTimeout) Error() string {
	return "filter timed out"
}

// ErrRejected indicates that a Peer was rejected carrying additional
// information as to the reason.
type ErrRejected struct {
	addr              NetAddress
	conn              net.Conn
	err               error
	id                ID
	isAuthFailure     bool
	isDuplicate       bool
	isFiltered        bool
	isIncompatible    bool
	isNodeInfoInvalid bool
	isSelf            bool
}

// Addr returns the NetAddress for the rejected Peer.
func (e ErrRejected) Addr() NetAddress {
	return e.addr
}

func (e ErrRejected) Error() string {
	if e.isAuthFailure {
		return fmt.Sprintf("auth failure: %s", e.err)
	}

	if e.isDuplicate {
		if e.conn != nil {
			return fmt.Sprintf(
				"duplicate CONN<%s>",
				e.conn.RemoteAddr().String(),
			)
		}
		if e.id != "" {
			return fmt.Sprintf("duplicate ID<%v>", e.id)
		}
	}

	if e.isFiltered {
		if e.conn != nil {
			return fmt.Sprintf(
				"filtered CONN<%s>: %s",
				e.conn.RemoteAddr().String(),
				e.err,
			)
		}

		if e.id != "" {
			return fmt.Sprintf("filtered ID<%v>: %s", e.id, e.err)
		}
	}

	if e.isIncompatible {
		return fmt.Sprintf("incompatible: %s", e.err)
	}

	if e.isNodeInfoInvalid {
		return fmt.Sprintf("invalid NodeInfo: %s", e.err)
	}

	if e.isSelf {
		return fmt.Sprintf("self ID<%v>", e.id)
	}

	return fmt.Sprintf("%s", e.err)
}

// IsAuthFailure when Peer authentication was unsuccessful.
func (e ErrRejected) IsAuthFailure() bool { return e.isAuthFailure }

// IsDuplicate when Peer ID or IP are present already.
func (e ErrRejected) IsDuplicate() bool { return e.isDuplicate }

// IsFiltered when Peer ID or IP was filtered.
func (e ErrRejected) IsFiltered() bool { return e.isFiltered }

// IsIncompatible when Peer NodeInfo is not compatible with our own.
func (e ErrRejected) IsIncompatible() bool { return e.isIncompatible }

// IsNodeInfoInvalid when the sent NodeInfo is not valid.
func (e ErrRejected) IsNodeInfoInvalid() bool { return e.isNodeInfoInvalid }

// IsSelf when Peer is our own node.
func (e ErrRejected) IsSelf() bool { return e.isSelf }

// ErrSwitchDuplicatePeerID to be raised when a peer is connecting with a known
// ID.
type ErrSwitchDuplicatePeerID struct {
	ID ID
}

func (e ErrSwitchDuplicatePeerID) Error() string {
	return fmt.Sprintf("duplicate peer ID %v", e.ID)
}

// ErrSwitchDuplicatePeerIP to be raised whena a peer is connecting with a known
// IP.
type ErrSwitchDuplicatePeerIP struct {
	IP net.IP
}

func (e ErrSwitchDuplicatePeerIP) Error() string {
	return fmt.Sprintf("duplicate peer IP %v", e.IP.String())
}

// ErrSwitchConnectToSelf to be raised when trying to connect to itself.
type ErrSwitchConnectToSelf struct {
	Addr *NetAddress
}

func (e ErrSwitchConnectToSelf) Error() string {
	return fmt.Sprintf("connect to self: %v", e.Addr)
}

type ErrSwitchAuthenticationFailure struct {
	Dialed *NetAddress
	Got    ID
}

func (e ErrSwitchAuthenticationFailure) Error() string {
	return fmt.Sprintf(
		"failed to authenticate peer. Dialed %v, but got peer with ID %s",
		e.Dialed,
		e.Got,
	)
}

// ErrTransportClosed is raised when the Transport has been closed.
type ErrTransportClosed struct{}

func (e ErrTransportClosed) Error() string {
	return "transport has been closed"
}

// ErrPeerRemoval is raised when attempting to remove a peer results in an error.
type ErrPeerRemoval struct{}

func (e ErrPeerRemoval) Error() string {
	return "peer removal failed"
}

//-------------------------------------------------------------------

type ErrNetAddressNoID struct {
	Addr string
}

func (e ErrNetAddressNoID) Error() string {
	return fmt.Sprintf("address (%s) does not contain ID", e.Addr)
}

type ErrNetAddressInvalid struct {
	Addr string
	Err  error
}

func (e ErrNetAddressInvalid) Error() string {
	return fmt.Sprintf("invalid address (%s): %v", e.Addr, e.Err)
}

type ErrNetAddressLookup struct {
	Addr string
	Err  error
}

func (e ErrNetAddressLookup) Error() string {
	return fmt.Sprintf("error looking up host (%s): %v", e.Addr, e.Err)
}

// ErrCurrentlyDialingOrExistingAddress indicates that we're currently
// dialing this address or it belongs to an existing peer.
type ErrCurrentlyDialingOrExistingAddress struct {
	Addr string
}

func (e ErrCurrentlyDialingOrExistingAddress) Error() string {
	return fmt.Sprintf("connection with %s has been established or dialed", e.Addr)
}
