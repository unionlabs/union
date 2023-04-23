/*
The commands from the SDK are defined with `cobra` and configured with the
`viper` package.

This takes place in the `InterceptConfigsPreRunHandler` function.
Since the `viper` package is used for configuration the precedence is dictated
by that package. That is

1. Command line switches
2. Environment variables
3. Files from configuration values
4. Default values

The global configuration instance exposed by the `viper` package is not
used by Cosmos SDK in this function. A new instance of `viper.Viper` is created
and the following is performed. The environmental variable prefix is set
to the current program name. Environmental variables consider the underscore
to be equivalent to the `.` or `-` character. This means that an configuration
value called `rpc.laddr` would be read from an environmental variable called
`MYTOOL_RPC_LADDR` if the current program name is `mytool`.

Running the `InterceptConfigsPreRunHandler` also reads `app.toml`
and `config.toml` from the home directory under the `config` directory.
If `config.toml` or `app.toml` do not exist then those files are created
and populated with default values. `InterceptConfigsPreRunHandler` takes
two parameters to set/update a custom template to create custom `app.toml`.
If these parameters are empty, the server then creates a default template
provided by the SDK.
*/
package server
