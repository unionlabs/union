package exported

// ProtocolVersionSetter defines the interface fulfilled by BaseApp
// which allows setting it's appVersion field.
type ProtocolVersionSetter interface {
	SetProtocolVersion(uint64)
}
