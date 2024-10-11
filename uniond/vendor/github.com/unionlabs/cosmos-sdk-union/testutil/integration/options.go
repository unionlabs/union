package integration

// Config is the configuration for the integration app.
type Config struct {
	AutomaticFinalizeBlock bool
	AutomaticCommit        bool
}

// Option is a function that can be used to configure the integration app.
type Option func(*Config)

// WithAutomaticFinalizeBlock calls ABCI finalize block.
func WithAutomaticFinalizeBlock() Option {
	return func(cfg *Config) {
		cfg.AutomaticFinalizeBlock = true
	}
}

// WithAutomaticCommit enables automatic commit.
// This means that the integration app will automatically commit the state after each msgs.
func WithAutomaticCommit() Option {
	return func(cfg *Config) {
		cfg.AutomaticCommit = true
	}
}
