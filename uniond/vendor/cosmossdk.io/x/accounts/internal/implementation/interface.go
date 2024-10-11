package implementation

// Account defines a smart account interface.
type Account interface {
	// RegisterInitHandler allows the smart account to register an initialisation handler, using
	// the provided InitBuilder. The handler will be called when the smart account is initialized
	// (deployed).
	RegisterInitHandler(builder *InitBuilder)

	// RegisterExecuteHandlers allows the smart account to register execution handlers.
	// The smart account might also decide to not register any execution handler.
	RegisterExecuteHandlers(builder *ExecuteBuilder)

	// RegisterQueryHandlers allows the smart account to register query handlers. The smart account
	// might also decide to not register any query handler.
	RegisterQueryHandlers(builder *QueryBuilder)
}
