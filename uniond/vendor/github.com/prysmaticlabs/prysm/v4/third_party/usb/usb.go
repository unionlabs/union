// usb - Self contained USB and HID library for Go
// Copyright 2019 The library Authors
//
// This library is free software: you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// The library is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License along
// with the library. If not, see <http://www.gnu.org/licenses/>.

// Package usb provide interfaces for generic USB devices.
package usb

import "errors"

// ErrDeviceClosed is returned for operations where the device closed before or
// during the execution.
var ErrDeviceClosed = errors.New("usb: device closed")

// ErrUnsupportedPlatform is returned for all operations where the underlying
// operating system is not supported by the library.
var ErrUnsupportedPlatform = errors.New("usb: unsupported platform")

// DeviceInfo contains all the information we know about a USB device. In case of
// HID devices, that might be a lot more extensive (empty fields for raw USB).
type DeviceInfo struct {
	Path         string // Platform-specific device path
	VendorID     uint16 // Device Vendor ID
	ProductID    uint16 // Device Product ID
	Release      uint16 // Device Release Number in binary-coded decimal, also known as Device Version Number
	Serial       string // Serial Number
	Manufacturer string // Manufacturer String
	Product      string // Product string
	UsagePage    uint16 // Usage Page for this Device/Interface (Windows/Mac only)
	Usage        uint16 // Usage for this Device/Interface (Windows/Mac only)

	// The USB interface which this logical device
	// represents. Valid on both Linux implementations
	// in all cases, and valid on the Windows implementation
	// only if the device contains more than one interface.
	Interface int

	// Raw low level libusb endpoint data for simplified communication
	rawDevice interface{}
	rawPort   *uint8 // Pointer to differentiate between unset and port 0
	rawReader *uint8 // Pointer to differentiate between unset and endpoint 0
	rawWriter *uint8 // Pointer to differentiate between unset and endpoint 0
}

// Device is a generic USB device interface. It may either be backed by a USB HID
// device or a low level raw (libusb) device.
type Device interface {
	// Close releases the USB device handle.
	Close() error

	// Write sends a binary blob to a USB device. For HID devices write uses reports,
	// for low level USB write uses interrupt transfers.
	Write(b []byte) (int, error)

	// Read retrieves a binary blob from a USB device. For HID devices read uses
	// reports, for low level USB read uses interrupt transfers.
	Read(b []byte) (int, error)
}
