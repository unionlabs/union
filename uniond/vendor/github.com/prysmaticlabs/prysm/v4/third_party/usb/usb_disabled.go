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

package usb

// Supported returns whether this platform is supported by the USB library or not.
// The goal of this method is to allow programmatically handling platforms that do
// not support USB and not having to fall back to build constraints.
func Supported() bool {
	return false
}

// Enumerate returns a list of all the USB devices attached to the system which
// match the vendor and product id. On platforms that this file implements the
// function is a noop and returns an empty list always.
func Enumerate(vendorID uint16, productID uint16) ([]DeviceInfo, error) {
	return nil, nil
}

// EnumerateRaw returns a list of all the USB devices attached to the system which
// match the vendor and product id. On platforms that this file implements the
// function is a noop and returns an empty list always.
func EnumerateRaw(vendorID uint16, productID uint16) ([]DeviceInfo, error) {
	return nil, nil
}

// EnumerateHid returns a list of all the HID devices attached to the system which
// match the vendor and product id. On platforms that this file implements the
// function is a noop and returns an empty list always.
func EnumerateHid(vendorID uint16, productID uint16) ([]DeviceInfo, error) {
	return nil, nil
}

// Open connects to a previously discovered USB device.
func (info DeviceInfo) Open() (Device, error) {
	return nil, ErrUnsupportedPlatform
}
