// hid - Gopher Interface Devices (USB HID)
// Copyright (c) 2017 Péter Szilágyi. All rights reserved.
//
// This file is released under the 3-clause BSD license. Note however that Linux
// support depends on libusb, released under GNU LGPL 2.1 or later.

package hid

import (
	"sync"
	"testing"
)

// Tests that device enumeration can be called concurrently from multiple threads.
func TestThreadedEnumerate(t *testing.T) {
	var pend sync.WaitGroup
	for i := 0; i < 8; i++ {
		pend.Add(1)

		go func(index int) {
			defer pend.Done()
			for j := 0; j < 512; j++ {
				Enumerate(uint16(index), 0)
			}
		}(i)
	}
	pend.Wait()
}
