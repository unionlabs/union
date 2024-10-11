// usb - Self contained USB and HID library for Go
// Copyright 2017 The library Authors
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

// HidDevice is a live HID USB connected device handle. On platforms that this file
// implements, the type lacks the actual HID device and all methods are noop.
type HidDevice struct {
	DeviceInfo // Embed the infos for easier access
}

// Close releases the HID USB device handle. On platforms that this file implements,
// the method is just a noop.
func (dev *HidDevice) Close() error {
	return ErrUnsupportedPlatform
}

// Write sends an output report to a HID device. On platforms that this file
// implements, the method just returns an error.
func (dev *HidDevice) Write(b []byte) (int, error) {
	return 0, ErrUnsupportedPlatform
}

// Read retrieves an input report from a HID device. On platforms that this file
// implements, the method just returns an error.
func (dev *HidDevice) Read(b []byte) (int, error) {
	return 0, ErrUnsupportedPlatform
}
