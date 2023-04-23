package depinject

// Inject builds the container specified by containerConfig and extracts the
// requested outputs from the container or returns an error. It is the single
// entry point for building and running a dependency injection container.
// Each of the values specified as outputs must be pointers to types that
// can be provided by the container.
//
// Ex:
//
//	var x int
//	Inject(Provide(func() int { return 1 }), &x)
//
// Inject uses the debug mode provided by AutoDebug which means there will be
// verbose debugging information if there is an error and nothing upon success.
// Use InjectDebug to configure debug behavior.
func Inject(containerConfig Config, outputs ...interface{}) error {
	loc := LocationFromCaller(1)
	return inject(loc, AutoDebug(), containerConfig, outputs...)
}

// InjectDebug is a version of Inject which takes an optional DebugOption for
// logging and visualization.
func InjectDebug(debugOpt DebugOption, config Config, outputs ...interface{}) error {
	loc := LocationFromCaller(1)
	return inject(loc, debugOpt, config, outputs...)
}

func inject(loc Location, debugOpt DebugOption, config Config, outputs ...interface{}) error {
	cfg, err := newDebugConfig()
	if err != nil {
		return err
	}

	// always generate graph on exit
	defer cfg.generateGraph()

	// debug cleanup
	defer func() {
		for _, f := range cfg.cleanup {
			f()
		}
	}()

	if err = doInject(cfg, loc, debugOpt, config, outputs...); err != nil {
		cfg.logf("Error: %v", err)
		if cfg.onError != nil {
			if err2 := cfg.onError.applyConfig(cfg); err2 != nil {
				return err2
			}
		}

		return err
	}

	if cfg.onSuccess != nil {
		if err2 := cfg.onSuccess.applyConfig(cfg); err2 != nil {
			return err2
		}
	}
	return nil
}

func doInject(cfg *debugConfig, loc Location, debugOpt DebugOption, config Config, outputs ...interface{}) error {
	if debugOpt != nil {
		if err := debugOpt.applyConfig(cfg); err != nil {
			return err
		}
	}

	cfg.logf("Registering providers")
	cfg.indentLogger()
	ctr := newContainer(cfg)
	err := config.apply(ctr)
	if err != nil {
		cfg.logf("Failed registering providers because of: %+v", err)
		return err
	}
	cfg.dedentLogger()

	return ctr.build(loc, outputs...)
}
