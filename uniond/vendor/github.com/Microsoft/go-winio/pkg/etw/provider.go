//go:build windows
// +build windows

package etw

import (
	"crypto/sha1" //nolint:gosec // not used for secure application
	"encoding/binary"
	"strings"
	"unicode/utf16"

	"github.com/Microsoft/go-winio/pkg/guid"
	"golang.org/x/sys/windows"
)

// Provider represents an ETW event provider. It is identified by a provider
// name and ID (GUID), which should always have a 1:1 mapping to each other
// (e.g. don't use multiple provider names with the same ID, or vice versa).
type Provider struct {
	ID         guid.GUID
	handle     providerHandle
	metadata   []byte
	callback   EnableCallback
	index      uint
	enabled    bool
	level      Level
	keywordAny uint64
	keywordAll uint64
}

// String returns the `provider`.ID as a string.
func (provider *Provider) String() string {
	if provider == nil {
		return "<nil>"
	}

	return provider.ID.String()
}

type providerHandle uint64

// ProviderState informs the provider EnableCallback what action is being
// performed.
type ProviderState uint32

const (
	// ProviderStateDisable indicates the provider is being disabled.
	ProviderStateDisable ProviderState = iota
	// ProviderStateEnable indicates the provider is being enabled.
	ProviderStateEnable
	// ProviderStateCaptureState indicates the provider is having its current
	// state snap-shotted.
	ProviderStateCaptureState
)

type eventInfoClass uint32

//nolint:deadcode,varcheck // keep unused constants for potential future use
const (
	eventInfoClassProviderBinaryTrackInfo eventInfoClass = iota
	eventInfoClassProviderSetReserved1
	eventInfoClassProviderSetTraits
	eventInfoClassProviderUseDescriptorType
)

// EnableCallback is the form of the callback function that receives provider
// enable/disable notifications from ETW.
type EnableCallback func(guid.GUID, ProviderState, Level, uint64, uint64, uintptr)

func providerCallback(
	sourceID guid.GUID,
	state ProviderState,
	level Level,
	matchAnyKeyword uint64,
	matchAllKeyword uint64,
	filterData uintptr,
	i uintptr,
) {
	provider := providers.getProvider(uint(i))

	switch state {
	case ProviderStateCaptureState:
	case ProviderStateDisable:
		provider.enabled = false
	case ProviderStateEnable:
		provider.enabled = true
		provider.level = level
		provider.keywordAny = matchAnyKeyword
		provider.keywordAll = matchAllKeyword
	}

	if provider.callback != nil {
		provider.callback(sourceID, state, level, matchAnyKeyword, matchAllKeyword, filterData)
	}
}

// providerIDFromName generates a provider ID based on the provider name. It
// uses the same algorithm as used by .NET's EventSource class, which is based
// on RFC 4122. More information on the algorithm can be found here:
// https://blogs.msdn.microsoft.com/dcook/2015/09/08/etw-provider-names-and-guids/
//
// The algorithm is roughly the RFC 4122 algorithm for a V5 UUID, but differs in
// the following ways:
//   - The input name is first upper-cased, UTF16-encoded, and converted to
//     big-endian.
//   - No variant is set on the result UUID.
//   - The result UUID is treated as being in little-endian format, rather than
//     big-endian.
func providerIDFromName(name string) guid.GUID {
	buffer := sha1.New() //nolint:gosec // not used for secure application
	namespace := guid.GUID{
		Data1: 0x482C2DB2,
		Data2: 0xC390,
		Data3: 0x47C8,
		Data4: [8]byte{0x87, 0xF8, 0x1A, 0x15, 0xBF, 0xC1, 0x30, 0xFB},
	}
	namespaceBytes := namespace.ToArray()
	buffer.Write(namespaceBytes[:])
	_ = binary.Write(buffer, binary.BigEndian, utf16.Encode([]rune(strings.ToUpper(name))))

	sum := buffer.Sum(nil)
	sum[7] = (sum[7] & 0xf) | 0x50

	a := [16]byte{}
	copy(a[:], sum)
	return guid.FromWindowsArray(a)
}

type providerOpts struct {
	callback EnableCallback
	id       guid.GUID
	group    guid.GUID
}

// ProviderOpt allows the caller to specify provider options to
// NewProviderWithOptions.
type ProviderOpt func(*providerOpts)

// WithCallback is used to provide a callback option to NewProviderWithOptions.
func WithCallback(callback EnableCallback) ProviderOpt {
	return func(opts *providerOpts) {
		opts.callback = callback
	}
}

// WithID is used to provide a provider ID option to NewProviderWithOptions.
func WithID(id guid.GUID) ProviderOpt {
	return func(opts *providerOpts) {
		opts.id = id
	}
}

// WithGroup is used to provide a provider group option to NewProviderWithOptions.
func WithGroup(group guid.GUID) ProviderOpt {
	return func(opts *providerOpts) {
		opts.group = group
	}
}

// NewProviderWithID creates and registers a new ETW provider, allowing the
// provider ID to be manually specified. This is most useful when there is an
// existing provider ID that must be used to conform to existing diagnostic
// infrastructure.
func NewProviderWithID(name string, id guid.GUID, callback EnableCallback) (provider *Provider, err error) {
	return NewProviderWithOptions(name, WithID(id), WithCallback(callback))
}

// NewProvider creates and registers a new ETW provider. The provider ID is
// generated based on the provider name.
func NewProvider(name string, callback EnableCallback) (provider *Provider, err error) {
	return NewProviderWithOptions(name, WithCallback(callback))
}

// Close unregisters the provider.
func (provider *Provider) Close() error {
	if provider == nil {
		return nil
	}

	providers.removeProvider(provider)
	return eventUnregister(provider.handle)
}

// IsEnabled calls IsEnabledForLevelAndKeywords with LevelAlways and all
// keywords set.
func (provider *Provider) IsEnabled() bool {
	return provider.IsEnabledForLevelAndKeywords(LevelAlways, ^uint64(0))
}

// IsEnabledForLevel calls IsEnabledForLevelAndKeywords with the specified level
// and all keywords set.
func (provider *Provider) IsEnabledForLevel(level Level) bool {
	return provider.IsEnabledForLevelAndKeywords(level, ^uint64(0))
}

// IsEnabledForLevelAndKeywords allows event producer code to check if there are
// any event sessions that are interested in an event, based on the event level
// and keywords. Although this check happens automatically in the ETW
// infrastructure, it can be useful to check if an event will actually be
// consumed before doing expensive work to build the event data.
func (provider *Provider) IsEnabledForLevelAndKeywords(level Level, keywords uint64) bool {
	if provider == nil {
		return false
	}

	if !provider.enabled {
		return false
	}

	// ETW automatically sets the level to 255 if it is specified as 0, so we
	// don't need to worry about the level=0 (all events) case.
	if level > provider.level {
		return false
	}

	if keywords != 0 && (keywords&provider.keywordAny == 0 || keywords&provider.keywordAll != provider.keywordAll) {
		return false
	}

	return true
}

// WriteEvent writes a single ETW event from the provider. The event is
// constructed based on the EventOpt and FieldOpt values that are passed as
// opts.
func (provider *Provider) WriteEvent(name string, eventOpts []EventOpt, fieldOpts []FieldOpt) error {
	if provider == nil {
		return nil
	}

	options := eventOptions{descriptor: newEventDescriptor()}
	em := &eventMetadata{}
	ed := &eventData{}

	// We need to evaluate the EventOpts first since they might change tags, and
	// we write out the tags before evaluating FieldOpts.
	for _, opt := range eventOpts {
		opt(&options)
	}

	if !provider.IsEnabledForLevelAndKeywords(options.descriptor.level, options.descriptor.keyword) {
		return nil
	}

	em.writeEventHeader(name, options.tags)

	for _, opt := range fieldOpts {
		opt(em, ed)
	}

	// Don't pass a data blob if there is no event data. There will always be
	// event metadata (e.g. for the name) so we don't need to do this check for
	// the metadata.
	dataBlobs := [][]byte{}
	if len(ed.toBytes()) > 0 {
		dataBlobs = [][]byte{ed.toBytes()}
	}

	return provider.writeEventRaw(
		options.descriptor,
		options.activityID,
		options.relatedActivityID,
		[][]byte{em.toBytes()},
		dataBlobs,
	)
}

// writeEventRaw writes a single ETW event from the provider. This function is
// less abstracted than WriteEvent, and presents a fairly direct interface to
// the event writing functionality. It expects a series of event metadata and
// event data blobs to be passed in, which must conform to the TraceLogging
// schema. The functions on EventMetadata and EventData can help with creating
// these blobs. The blobs of each type are effectively concatenated together by
// the ETW infrastructure.
func (provider *Provider) writeEventRaw(
	descriptor *eventDescriptor,
	activityID guid.GUID,
	relatedActivityID guid.GUID,
	metadataBlobs [][]byte,
	dataBlobs [][]byte) error {
	dataDescriptorCount := uint32(1 + len(metadataBlobs) + len(dataBlobs))
	dataDescriptors := make([]eventDataDescriptor, 0, dataDescriptorCount)

	dataDescriptors = append(dataDescriptors,
		newEventDataDescriptor(eventDataDescriptorTypeProviderMetadata, provider.metadata))
	for _, blob := range metadataBlobs {
		dataDescriptors = append(dataDescriptors,
			newEventDataDescriptor(eventDataDescriptorTypeEventMetadata, blob))
	}
	for _, blob := range dataBlobs {
		dataDescriptors = append(dataDescriptors,
			newEventDataDescriptor(eventDataDescriptorTypeUserData, blob))
	}

	return eventWriteTransfer(provider.handle,
		descriptor,
		(*windows.GUID)(&activityID),
		(*windows.GUID)(&relatedActivityID),
		dataDescriptorCount,
		&dataDescriptors[0])
}
