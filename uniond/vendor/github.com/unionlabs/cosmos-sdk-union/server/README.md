# Server

The `server` package is responsible for providing the mechanisms necessary to
start an ABCI CometBFT application and provides the CLI framework (based on [cobra](https://github.com/spf13/cobra))
necessary to fully bootstrap an application. The package exposes two core functions: `StartCmd`
and `ExportCmd` which creates commands to start the application and export state respectively.

## Preliminary

The root command of an application typically is constructed with:

* command to start an application binary
* three meta commands: `query`, `tx`, and a few auxiliary commands such as `genesis`.
utilities.

It is vital that the root command of an application uses `PersistentPreRun()` cobra command
property for executing the command, so all child commands have access to the server and client contexts.
These contexts are set as their default values initially and may be modified,
scoped to the command, in their respective `PersistentPreRun()` functions. Note that
the `client.Context` is typically pre-populated with "default" values that may be
useful for all commands to inherit and override if necessary.

Example:

```go
var (
	initClientCtx  = client.Context{...}

	rootCmd = &cobra.Command{
		Use:   "simd",
		Short: "simulation app",
		PersistentPreRunE: func(cmd *cobra.Command, _ []string) error {
			if err := client.SetCmdClientContextHandler(initClientCtx, cmd); err != nil {
				return err
			}

			return server.InterceptConfigsPreRunHandler(cmd)
		},
	}
    // add root sub-commands ...
)
```

The `SetCmdClientContextHandler` call reads persistent flags via `ReadPersistentCommandFlags`
which creates a `client.Context` and sets that on the root command's `Context`.

The `InterceptConfigsPreRunHandler` call creates a viper literal, default `server.Context`,
and a logger and sets that on the root command's `Context`. The `server.Context`
will be modified and saved to disk via the internal `interceptConfigs` call, which
either reads or creates a CometBFT configuration based on the home path provided.
In addition, `interceptConfigs` also reads and loads the application configuration,
`app.toml`, and binds that to the `server.Context` viper literal. This is vital
so the application can get access to not only the CLI flags, but also to the
application configuration values provided by this file.

## `StartCmd`

The `StartCmd` accepts an `AppCreator` function which returns an `Application`.
The `AppCreator` is responsible for constructing the application based on the
options provided to it via `AppOptions`. The `AppOptions` interface type defines
a single method, `Get() interface{}`, and is implemented as a [viper](https://github.com/spf13/viper)
literal that exists in the `server.Context`. All the possible options an application
may use and provide to the construction process are defined by the `StartCmd`
and by the application's config file, `app.toml`.

The application can either be started in-process or as an external process. The
former creates a CometBFT service and the latter creates a CometBFT Node.

Under the hood, `StartCmd` will call `GetServerContextFromCmd`, which provides
the command access to a `server.Context`. This context provides access to the
viper literal, the CometBFT config and logger. This allows flags to be bound
the viper literal and passed to the application construction.

Example:

```go
func newApp(logger log.Logger, db corestore.KVStoreWithBatch, traceStore io.Writer, appOpts servertypes.AppOptions) servertypes.Application {
	baseappOptions := server.DefaultBaseappOptions(appOpts)
	return simapp.NewSimApp(
		logger, db, traceStore, true,
		appOpts,
		baseappOptions...,
	)
}
```

Note, some of the options provided are exposed via CLI flags in the start command
and some are also allowed to be set in the application's `app.toml`. It is recommend
to use the `cast` package for type safety guarantees and due to the limitations of
CLI flag types.
