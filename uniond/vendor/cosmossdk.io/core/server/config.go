package server

// DynamicConfig defines an interface for configuration that can be dynamically
// fetched at runtime by an arbitrary key.
type DynamicConfig interface {
	Get(string) any
	GetString(string) string
}
