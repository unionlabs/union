package view

// AppState defines an interface for things that represent application state in schema format.
type AppState interface {
	// GetModule returns the module state for the given module name. If the module does not exist, nil and no error
	// should be returned.
	GetModule(moduleName string) (ModuleState, error)

	// Modules iterates over all the module state instances in the app. If there is an error getting a module state,
	// modState may be nil and err will be non-nil.
	Modules(f func(modState ModuleState, err error) bool)

	// NumModules returns the number of modules in the app.
	NumModules() (int, error)
}
