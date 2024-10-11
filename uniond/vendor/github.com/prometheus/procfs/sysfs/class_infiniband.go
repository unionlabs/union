// Copyright 2019 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build linux
// +build linux

package sysfs

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"github.com/prometheus/procfs/internal/util"
)

const infinibandClassPath = "class/infiniband"

// InfiniBandCounters contains counter values from files in
// /sys/class/infiniband/<Name>/ports/<Port>/counters or
// /sys/class/infiniband/<Name>/ports/<Port>/counters_ext
// for a single port of one InfiniBand device.
type InfiniBandCounters struct {
	LegacyPortMulticastRcvPackets  *uint64 // counters_ext/port_multicast_rcv_packets
	LegacyPortMulticastXmitPackets *uint64 // counters_ext/port_multicast_xmit_packets
	LegacyPortRcvData64            *uint64 // counters_ext/port_rcv_data_64
	LegacyPortRcvPackets64         *uint64 // counters_ext/port_rcv_packets_64
	LegacyPortUnicastRcvPackets    *uint64 // counters_ext/port_unicast_rcv_packets
	LegacyPortUnicastXmitPackets   *uint64 // counters_ext/port_unicast_xmit_packets
	LegacyPortXmitData64           *uint64 // counters_ext/port_xmit_data_64
	LegacyPortXmitPackets64        *uint64 // counters_ext/port_xmit_packets_64

	ExcessiveBufferOverrunErrors *uint64 // counters/excessive_buffer_overrun_errors
	LinkDowned                   *uint64 // counters/link_downed
	LinkErrorRecovery            *uint64 // counters/link_error_recovery
	LocalLinkIntegrityErrors     *uint64 // counters/local_link_integrity_errors
	MulticastRcvPackets          *uint64 // counters/multicast_rcv_packets
	MulticastXmitPackets         *uint64 // counters/multicast_xmit_packets
	PortRcvConstraintErrors      *uint64 // counters/port_rcv_constraint_errors
	PortRcvData                  *uint64 // counters/port_rcv_data
	PortRcvDiscards              *uint64 // counters/port_rcv_discards
	PortRcvErrors                *uint64 // counters/port_rcv_errors
	PortRcvPackets               *uint64 // counters/port_rcv_packets
	PortRcvRemotePhysicalErrors  *uint64 // counters/port_rcv_remote_physical_errors
	PortRcvSwitchRelayErrors     *uint64 // counters/port_rcv_switch_relay_errors
	PortXmitConstraintErrors     *uint64 // counters/port_xmit_constraint_errors
	PortXmitData                 *uint64 // counters/port_xmit_data
	PortXmitDiscards             *uint64 // counters/port_xmit_discards
	PortXmitPackets              *uint64 // counters/port_xmit_packets
	PortXmitWait                 *uint64 // counters/port_xmit_wait
	SymbolError                  *uint64 // counters/symbol_error
	UnicastRcvPackets            *uint64 // counters/unicast_rcv_packets
	UnicastXmitPackets           *uint64 // counters/unicast_xmit_packets
	VL15Dropped                  *uint64 // counters/VL15_dropped
}

// InfiniBandHwCounters contains counter value from files in
// /sys/class/infiniband/<Name>/ports/<Port>/hw_counters
// for a single port of one InfiniBand device.
type InfiniBandHwCounters struct {
	DuplicateRequest        *uint64 // hw_counters/duplicate_request
	ImpliedNakSeqErr        *uint64 // hw_counters/implied_nak_seq_err
	Lifespan                *uint64 // hw_counters/lifespan
	LocalAckTimeoutErr      *uint64 // hw_counters/local_ack_timeout_err
	NpCnpSent               *uint64 // hw_counters/np_cnp_sent
	NpEcnMarkedRocePackets  *uint64 // hw_counters/np_ecn_marked_roce_packets
	OutOfBuffer             *uint64 // hw_counters/out_of_buffer
	OutOfSequence           *uint64 // hw_counters/out_of_sequence
	PacketSeqErr            *uint64 // hw_counters/packet_seq_err
	ReqCqeError             *uint64 // hw_counters/req_cqe_error
	ReqCqeFlushError        *uint64 // hw_counters/req_cqe_flush_error
	ReqRemoteAccessErrors   *uint64 // hw_counters/req_remote_access_errors
	ReqRemoteInvalidRequest *uint64 // hw_counters/req_remote_invalid_request
	RespCqeError            *uint64 // hw_counters/resp_cqe_error
	RespCqeFlushError       *uint64 // hw_counters/resp_cqe_flush_error
	RespLocalLengthError    *uint64 // hw_counters/resp_local_length_error
	RespRemoteAccessErrors  *uint64 // hw_counters/resp_remote_access_errors
	RnrNakRetryErr          *uint64 // hw_counters/rnr_nak_retry_err
	RoceAdpRetrans          *uint64 // hw_counters/roce_adp_retrans
	RoceAdpRetransTo        *uint64 // hw_counters/roce_adp_retrans_to
	RoceSlowRestart         *uint64 // hw_counters/roce_slow_restart
	RoceSlowRestartCnps     *uint64 // hw_counters/roce_slow_restart_cnps
	RoceSlowRestartTrans    *uint64 // hw_counters/roce_slow_restart_trans
	RpCnpHandled            *uint64 // hw_counters/rp_cnp_handled
	RpCnpIgnored            *uint64 // hw_counters/rp_cnp_ignored
	RxAtomicRequests        *uint64 // hw_counters/rx_atomic_requests
	RxDctConnect            *uint64 // hw_counters/rx_dct_connect
	RxIcrcEncapsulated      *uint64 // hw_counters/rx_icrc_encapsulated
	RxReadRequests          *uint64 // hw_counters/rx_read_requests
	RxWriteRequests         *uint64 // hw_counters/rx_write_requests
}

// InfiniBandPort contains info from files in
// /sys/class/infiniband/<Name>/ports/<Port>
// for a single port of one InfiniBand device.
type InfiniBandPort struct {
	Name        string
	Port        uint
	State       string // String representation from /sys/class/infiniband/<Name>/ports/<Port>/state
	StateID     uint   // ID from /sys/class/infiniband/<Name>/ports/<Port>/state
	PhysState   string // String representation from /sys/class/infiniband/<Name>/ports/<Port>/phys_state
	PhysStateID uint   // String representation from /sys/class/infiniband/<Name>/ports/<Port>/phys_state
	Rate        uint64 // in bytes/second from /sys/class/infiniband/<Name>/ports/<Port>/rate
	Counters    InfiniBandCounters
	HwCounters  InfiniBandHwCounters
}

// InfiniBandDevice contains info from files in /sys/class/infiniband for a
// single InfiniBand device.
type InfiniBandDevice struct {
	Name            string
	BoardID         string // /sys/class/infiniband/<Name>/board_id
	FirmwareVersion string // /sys/class/infiniband/<Name>/fw_ver
	HCAType         string // /sys/class/infiniband/<Name>/hca_type
	Ports           map[uint]InfiniBandPort
}

// InfiniBandClass is a collection of every InfiniBand device in
// /sys/class/infiniband.
//
// The map keys are the names of the InfiniBand devices.
type InfiniBandClass map[string]InfiniBandDevice

// InfiniBandClass returns info for all InfiniBand devices read from
// /sys/class/infiniband.
func (fs FS) InfiniBandClass() (InfiniBandClass, error) {
	path := fs.sys.Path(infinibandClassPath)

	dirs, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	ibc := make(InfiniBandClass, len(dirs))
	for _, d := range dirs {
		device, err := fs.parseInfiniBandDevice(d.Name())
		if err != nil {
			return nil, err
		}

		ibc[device.Name] = *device
	}

	return ibc, nil
}

// Parse one InfiniBand device.
// Refer to https://www.kernel.org/doc/Documentation/ABI/stable/sysfs-class-infiniband
func (fs FS) parseInfiniBandDevice(name string) (*InfiniBandDevice, error) {
	path := fs.sys.Path(infinibandClassPath, name)
	device := InfiniBandDevice{Name: name}

	// fw_ver is exposed by all InfiniBand drivers since kernel version 4.10.
	value, err := util.SysReadFile(filepath.Join(path, "fw_ver"))
	if err != nil {
		return nil, fmt.Errorf("failed to read HCA firmware version: %w", err)
	}
	device.FirmwareVersion = value

	// Not all InfiniBand drivers expose all of these.
	for _, f := range [...]string{"board_id", "hca_type"} {
		name := filepath.Join(path, f)
		value, err := util.SysReadFile(name)
		if err != nil {
			if os.IsNotExist(err) {
				continue
			}
			return nil, fmt.Errorf("failed to read file %q: %w", name, err)
		}

		switch f {
		case "board_id":
			device.BoardID = value
		case "hca_type":
			device.HCAType = value
		}
	}

	portsPath := filepath.Join(path, "ports")
	ports, err := os.ReadDir(portsPath)
	if err != nil {
		return nil, fmt.Errorf("failed to list InfiniBand ports at %q: %w", portsPath, err)
	}

	device.Ports = make(map[uint]InfiniBandPort, len(ports))
	for _, d := range ports {
		port, err := fs.parseInfiniBandPort(name, d.Name())
		if err != nil {
			return nil, err
		}

		device.Ports[port.Port] = *port
	}

	return &device, nil
}

// Parse InfiniBand state. Expected format: "<id>: <string-representation>".
func parseState(s string) (uint, string, error) {
	parts := strings.Split(s, ":")
	if len(parts) != 2 {
		return 0, "", fmt.Errorf("failed to split %s into 'ID: NAME'", s)
	}
	name := strings.TrimSpace(parts[1])
	value, err := strconv.ParseUint(strings.TrimSpace(parts[0]), 10, 32)
	if err != nil {
		return 0, name, fmt.Errorf("failed to convert %s into uint", strings.TrimSpace(parts[0]))
	}
	id := uint(value)
	return id, name, nil
}

// Parse rate (example: "100 Gb/sec (4X EDR)") and return it as bytes/second.
func parseRate(s string) (uint64, error) {
	parts := strings.SplitAfterN(s, " ", 2)
	if len(parts) != 2 {
		return 0, fmt.Errorf("failed to split %q", s)
	}
	value, err := strconv.ParseFloat(strings.TrimSpace(parts[0]), 32)
	if err != nil {
		return 0, fmt.Errorf("failed to convert %s into uint", strings.TrimSpace(parts[0]))
	}
	// Convert Gb/s into bytes/s
	rate := uint64(value * 125000000)
	return rate, nil
}

// parseInfiniBandPort scans predefined files in /sys/class/infiniband/<device>/ports/<port>
// directory and gets their contents.
func (fs FS) parseInfiniBandPort(name string, port string) (*InfiniBandPort, error) {
	portNumber, err := strconv.ParseUint(port, 10, 32)
	if err != nil {
		return nil, fmt.Errorf("failed to convert %s into uint", port)
	}
	ibp := InfiniBandPort{Name: name, Port: uint(portNumber)}

	portPath := fs.sys.Path(infinibandClassPath, name, "ports", port)
	content, err := os.ReadFile(filepath.Join(portPath, "state"))
	if err != nil {
		return nil, err
	}
	id, name, err := parseState(string(content))
	if err != nil {
		return nil, fmt.Errorf("could not parse state file in %q: %w", portPath, err)
	}
	ibp.State = name
	ibp.StateID = id

	content, err = os.ReadFile(filepath.Join(portPath, "phys_state"))
	if err != nil {
		return nil, err
	}
	id, name, err = parseState(string(content))
	if err != nil {
		return nil, fmt.Errorf("could not parse phys_state file in %q: %w", portPath, err)
	}
	ibp.PhysState = name
	ibp.PhysStateID = id

	content, err = os.ReadFile(filepath.Join(portPath, "rate"))
	if err != nil {
		return nil, err
	}
	ibp.Rate, err = parseRate(string(content))
	if err != nil {
		return nil, fmt.Errorf("could not parse rate file in %q: %w", portPath, err)
	}

	// Intel irdma module does not expose /sys/class/infiniband/<device>/ports/<port-num>/counters
	if !strings.HasPrefix(ibp.Name, "irdma") {
		counters, err := parseInfiniBandCounters(portPath)
		if err != nil {
			return nil, err
		}
		ibp.Counters = *counters
	}

	if strings.HasPrefix(ibp.Name, "irdma") || strings.HasPrefix(ibp.Name, "mlx5_") {
		hwCounters, err := parseInfiniBandHwCounters(portPath)
		if err != nil {
			return nil, err
		}
		ibp.HwCounters = *hwCounters
	}

	return &ibp, nil
}

// parseInfiniBandCounters parses the counters exposed under
// /sys/class/infiniband/<device>/ports/<port-num>/counters, which first appeared in kernel v2.6.12.
// Prior to kernel v4.5, 64-bit counters were exposed separately under the "counters_ext" directory.
func parseInfiniBandCounters(portPath string) (*InfiniBandCounters, error) {
	var counters InfiniBandCounters

	path := filepath.Join(portPath, "counters")
	files, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	for _, f := range files {
		if !f.Type().IsRegular() {
			continue
		}

		name := filepath.Join(path, f.Name())
		value, err := util.SysReadFile(name)
		if err != nil {
			if os.IsNotExist(err) || os.IsPermission(err) || err.Error() == "operation not supported" || errors.Is(err, os.ErrInvalid) {
				continue
			}
			return nil, fmt.Errorf("failed to read file %q: %w", name, err)
		}

		// According to Mellanox, the metrics port_rcv_data, port_xmit_data,
		// port_rcv_data_64, and port_xmit_data_64 "are divided by 4 unconditionally"
		// as they represent the amount of data being transmitted and received per lane.
		// Mellanox cards have 4 lanes per port, so all values must be multiplied by 4
		// to get the expected value.

		vp := util.NewValueParser(value)

		switch f.Name() {
		case "excessive_buffer_overrun_errors":
			counters.ExcessiveBufferOverrunErrors = vp.PUInt64()
		case "link_downed":
			counters.LinkDowned = vp.PUInt64()
		case "link_error_recovery":
			counters.LinkErrorRecovery = vp.PUInt64()
		case "local_link_integrity_errors":
			counters.LocalLinkIntegrityErrors = vp.PUInt64()
		case "multicast_rcv_packets":
			counters.MulticastRcvPackets = vp.PUInt64()
		case "multicast_xmit_packets":
			counters.MulticastXmitPackets = vp.PUInt64()
		case "port_rcv_constraint_errors":
			counters.PortRcvConstraintErrors = vp.PUInt64()
		case "port_rcv_data":
			counters.PortRcvData = vp.PUInt64()
			if counters.PortRcvData != nil {
				*counters.PortRcvData *= 4
			}
		case "port_rcv_discards":
			counters.PortRcvDiscards = vp.PUInt64()
		case "port_rcv_errors":
			counters.PortRcvErrors = vp.PUInt64()
		case "port_rcv_packets":
			counters.PortRcvPackets = vp.PUInt64()
		case "port_rcv_remote_physical_errors":
			counters.PortRcvRemotePhysicalErrors = vp.PUInt64()
		case "port_rcv_switch_relay_errors":
			counters.PortRcvSwitchRelayErrors = vp.PUInt64()
		case "port_xmit_constraint_errors":
			counters.PortXmitConstraintErrors = vp.PUInt64()
		case "port_xmit_data":
			counters.PortXmitData = vp.PUInt64()
			if counters.PortXmitData != nil {
				*counters.PortXmitData *= 4
			}
		case "port_xmit_discards":
			counters.PortXmitDiscards = vp.PUInt64()
		case "port_xmit_packets":
			counters.PortXmitPackets = vp.PUInt64()
		case "port_xmit_wait":
			counters.PortXmitWait = vp.PUInt64()
		case "symbol_error":
			counters.SymbolError = vp.PUInt64()
		case "unicast_rcv_packets":
			counters.UnicastRcvPackets = vp.PUInt64()
		case "unicast_xmit_packets":
			counters.UnicastXmitPackets = vp.PUInt64()
		case "VL15_dropped":
			counters.VL15Dropped = vp.PUInt64()
		}

		if err := vp.Err(); err != nil {
			// Ugly workaround for handling https://github.com/prometheus/node_exporter/issues/966
			// when counters are `N/A (not available)`.
			// This was already patched and submitted, see
			// https://www.spinics.net/lists/linux-rdma/msg68596.html
			// Remove this as soon as the fix lands in the enterprise distros.
			if strings.Contains(value, "N/A (no PMA)") {
				continue
			}
			return nil, err
		}
	}

	// Parse pre-kernel-v4.5 64-bit counters.
	path = filepath.Join(portPath, "counters_ext")
	files, err = os.ReadDir(path)
	if err != nil && !os.IsNotExist(err) {
		return nil, err
	}

	for _, f := range files {
		if !f.Type().IsRegular() {
			continue
		}

		name := filepath.Join(path, f.Name())
		value, err := util.SysReadFile(name)
		if err != nil {
			if os.IsNotExist(err) || os.IsPermission(err) || err.Error() == "operation not supported" || errors.Is(err, os.ErrInvalid) {
				continue
			}
			return nil, fmt.Errorf("failed to read file %q: %w", name, err)
		}

		vp := util.NewValueParser(value)

		switch f.Name() {
		case "port_multicast_rcv_packets":
			counters.LegacyPortMulticastRcvPackets = vp.PUInt64()
		case "port_multicast_xmit_packets":
			counters.LegacyPortMulticastXmitPackets = vp.PUInt64()
		case "port_rcv_data_64":
			counters.LegacyPortRcvData64 = vp.PUInt64()
			if counters.LegacyPortRcvData64 != nil {
				*counters.LegacyPortRcvData64 *= 4
			}
		case "port_rcv_packets_64":
			counters.LegacyPortRcvPackets64 = vp.PUInt64()
		case "port_unicast_rcv_packets":
			counters.LegacyPortUnicastRcvPackets = vp.PUInt64()
		case "port_unicast_xmit_packets":
			counters.LegacyPortUnicastXmitPackets = vp.PUInt64()
		case "port_xmit_data_64":
			counters.LegacyPortXmitData64 = vp.PUInt64()
			if counters.LegacyPortXmitData64 != nil {
				*counters.LegacyPortXmitData64 *= 4
			}
		case "port_xmit_packets_64":
			counters.LegacyPortXmitPackets64 = vp.PUInt64()
		}

		if err := vp.Err(); err != nil {
			// Ugly workaround for handling https://github.com/prometheus/node_exporter/issues/966
			// when counters are `N/A (not available)`.
			// This was already patched and submitted, see
			// https://www.spinics.net/lists/linux-rdma/msg68596.html
			// Remove this as soon as the fix lands in the enterprise distros.
			if strings.Contains(value, "N/A (no PMA)") {
				continue
			}
			return nil, err
		}
	}

	return &counters, nil
}

// parseInfiniBandHwCounters parses the optional counters exposed under
// /sys/class/infiniband/<device>/ports/<port-num>/hw_counters, which first appeared in kernel v4.6.
func parseInfiniBandHwCounters(portPath string) (*InfiniBandHwCounters, error) {
	var hwCounters InfiniBandHwCounters

	path := filepath.Join(portPath, "hw_counters")
	files, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	for _, f := range files {
		if !f.Type().IsRegular() {
			continue
		}

		name := filepath.Join(path, f.Name())
		value, err := util.SysReadFile(name)
		if err != nil {
			if os.IsNotExist(err) || os.IsPermission(err) || err.Error() == "operation not supported" || errors.Is(err, os.ErrInvalid) {
				continue
			}
			return nil, fmt.Errorf("failed to read file %q: %w", name, err)
		}

		vp := util.NewValueParser(value)

		switch f.Name() {
		case "duplicate_request":
			hwCounters.DuplicateRequest = vp.PUInt64()
		case "implied_nak_seq_err":
			hwCounters.ImpliedNakSeqErr = vp.PUInt64()
		case "lifespan":
			hwCounters.Lifespan = vp.PUInt64()
		case "local_ack_timeout_err":
			hwCounters.LocalAckTimeoutErr = vp.PUInt64()
		case "np_cnp_sent":
			hwCounters.NpCnpSent = vp.PUInt64()
		case "np_ecn_marked_roce_packets":
			hwCounters.NpEcnMarkedRocePackets = vp.PUInt64()
		case "out_of_buffer":
			hwCounters.OutOfBuffer = vp.PUInt64()
		case "out_of_sequence":
			hwCounters.OutOfSequence = vp.PUInt64()
		case "packet_seq_err":
			hwCounters.PacketSeqErr = vp.PUInt64()
		case "req_cqe_error":
			hwCounters.ReqCqeError = vp.PUInt64()
		case "req_cqe_flush_error":
			hwCounters.ReqCqeFlushError = vp.PUInt64()
		case "req_remote_access_errors":
			hwCounters.ReqRemoteAccessErrors = vp.PUInt64()
		case "req_remote_invalid_request":
			hwCounters.ReqRemoteInvalidRequest = vp.PUInt64()
		case "resp_cqe_error":
			hwCounters.RespCqeError = vp.PUInt64()
		case "resp_cqe_flush_error":
			hwCounters.RespCqeFlushError = vp.PUInt64()
		case "resp_local_length_error":
			hwCounters.RespLocalLengthError = vp.PUInt64()
		case "resp_remote_access_errors":
			hwCounters.RespRemoteAccessErrors = vp.PUInt64()
		case "rnr_nak_retry_err":
			hwCounters.RnrNakRetryErr = vp.PUInt64()
		case "roce_adp_retrans":
			hwCounters.RoceAdpRetrans = vp.PUInt64()
		case "roce_adp_retrans_to":
			hwCounters.RoceAdpRetransTo = vp.PUInt64()
		case "roce_slow_restart":
			hwCounters.RoceSlowRestart = vp.PUInt64()
		case "roce_slow_restart_cnps":
			hwCounters.RoceSlowRestartCnps = vp.PUInt64()
		case "roce_slow_restart_trans":
			hwCounters.RoceSlowRestartTrans = vp.PUInt64()
		case "rp_cnp_handled":
			hwCounters.RpCnpHandled = vp.PUInt64()
		case "rp_cnp_ignored":
			hwCounters.RpCnpIgnored = vp.PUInt64()
		case "rx_atomic_requests":
			hwCounters.RxAtomicRequests = vp.PUInt64()
		case "rx_dct_connect":
			hwCounters.RxDctConnect = vp.PUInt64()
		case "rx_icrc_encapsulated":
			hwCounters.RxIcrcEncapsulated = vp.PUInt64()
		case "rx_read_requests":
			hwCounters.RxReadRequests = vp.PUInt64()
		case "rx_write_requests":
			hwCounters.RxWriteRequests = vp.PUInt64()
		}

		if err := vp.Err(); err != nil {
			// Ugly workaround for handling https://github.com/prometheus/node_exporter/issues/966
			// when counters are `N/A (not available)`.
			// This was already patched and submitted, see
			// https://www.spinics.net/lists/linux-rdma/msg68596.html
			// Remove this as soon as the fix lands in the enterprise distros.
			if strings.Contains(value, "N/A (no PMA)") {
				continue
			}
			return nil, err
		}
	}
	return &hwCounters, nil
}
