package v7

import "context"

// ConnectionKeeper expected IBC connection keeper
type ConnectionKeeper interface {
	CreateSentinelLocalhostConnection(ctx context.Context)
}
