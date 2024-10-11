package context

type (
	loggerContextKey struct{}
	viperContextKey  struct{}
)

var (
	// LoggerContextKey is the context key where the logger can be found.
	LoggerContextKey loggerContextKey
	// ViperContextKey is the context key where the viper instance can be found.
	ViperContextKey viperContextKey
)
