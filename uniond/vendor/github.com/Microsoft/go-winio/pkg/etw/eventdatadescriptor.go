//go:build windows

package etw

import (
	"unsafe"
)

type eventDataDescriptorType uint8

const (
	eventDataDescriptorTypeUserData eventDataDescriptorType = iota
	eventDataDescriptorTypeEventMetadata
	eventDataDescriptorTypeProviderMetadata
)

type eventDataDescriptor struct {
	ptr      ptr64
	size     uint32
	dataType eventDataDescriptorType
	_        uint8
	_        uint16
}

func newEventDataDescriptor(dataType eventDataDescriptorType, buffer []byte) eventDataDescriptor {
	return eventDataDescriptor{
		ptr:      ptr64{ptr: unsafe.Pointer(&buffer[0])},
		size:     uint32(len(buffer)),
		dataType: dataType,
	}
}
