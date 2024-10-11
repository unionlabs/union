package context

type (
	execModeKey    struct{}
	cometInfoKey   struct{}
	initInfoKey    struct{}
	environmentKey struct{}
)

var (
	// ExecModeKey is the context key for setting the execution mode.
	ExecModeKey = execModeKey{}
	// CometInfoKey is the context key for allowing modules to get CometInfo.
	CometInfoKey = cometInfoKey{}
	// CometParamsInitInfoKey is the context key for setting consensus params from genesis in the consensus module.
	CometParamsInitInfoKey = initInfoKey{}

	// EnvironmentContextKey is the context key for the environment.
	// A caller should not assume the environment is available in each context.
	// ref: https://github.com/cosmos/cosmos-sdk/issues/19640
	EnvironmentContextKey = environmentKey{}
)
