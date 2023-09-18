---
sidebar_position: 1
---


# AutoCLI

:::note Synopsis
This document details how to build CLI and REST interfaces for a module. Examples from various Cosmos SDK modules are included.
:::

:::note Pre-requisite Readings

* [Building Modules Intro](./01-intro.md)

:::

The `autocli` package is a [Go library](https://pkg.go.dev/cosmossdk.io/client/v2/autocli) for generating CLI (command line interface) interfaces for Cosmos SDK-based applications. It provides a simple way to add CLI commands to your application by generating them automatically based on your gRPC service definitions. Autocli generates CLI commands and flags directly from your protobuf messages, including options, input parameters, and output parameters. This means that you can easily add a CLI interface to your application without having to manually create and manage commands.

## Getting Started

Here are the steps to use the `autocli` package:

1. Define your app's modules that implement the `appmodule.AppModule` interface.
2. (optional) When willing to configure how behave `autocli` command generation, implement the `func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions` method on the module. Learn more [here](#advanced-usage).
3. Use the `autocli.AppOptions` struct to specifies the modules you defined. If you are using the `depinject` package to manage your app's dependencies, it can automatically create an instance of `autocli.AppOptions` based on your app's configuration.
4. Use the `EnhanceRootCommand()` method provided by `autocli` to add the CLI commands for the specified modules to your root command and can also be found in the `client/v2/autocli/app.go` file. Additionally, this method adds the `autocli` functionality to your app's root command. This method is additive only, meaning that it does not create commands if they are already registered for a module. Instead, it adds any missing commands to the root command.

Here's an example of how to use `autocli`:

``` go
// Define your app's modules
testModules := map[string]appmodule.AppModule{
    "testModule": &TestModule{},
}

// Define the autocli AppOptions
autoCliOpts := autocli.AppOptions{
    Modules: testModules,
}

// Get the root command
rootCmd := &cobra.Command{
    Use: "app",
}

// Enhance the root command with autocli
autocli.EnhanceRootCommand(rootCmd, autoCliOpts)

// Run the root command
if err := rootCmd.Execute(); err != nil {
    fmt.Println(err)
}
```

## Flags

`autocli` generates flags for each field in a protobuf message. By default, the names of the flags are generated based on the names of the fields in the message. You can customise the flag names using the `namingOptions` parameter of the `Builder.AddMessageFlags()` method.

To define flags for a message, you can use the `Builder.AddMessageFlags()` method. This method takes the `cobra.Command` instance and the message type as input, and generates flags for each field in the message.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/1ac260cb1c6f05666f47e67f8b2cfd6229a55c3b/client/v2/autocli/common.go#L44-L49
```

The `binder` variable returned by the `AddMessageFlags()` method is used to bind the command-line arguments to the fields in the message.

You can also customise the behavior of the flags using the `namingOptions` parameter of the `Builder.AddMessageFlags()` method. This parameter allows you to specify a custom prefix for the flags, and to specify whether to generate flags for repeated fields and whether to generate flags for fields with default values.

## Commands and Queries

The `autocli` package generates CLI commands and flags for each method defined in your gRPC service. By default, it generates commands for each RPC method that does not return a stream of messages. The commands are named based on the name of the service method.

For example, given the following protobuf definition for a service:

```protobuf
service MyService {
  rpc MyMethod(MyRequest) returns (MyResponse) {}
}
```

`autocli` will generate a command named `my-method` for the `MyMethod` method. The command will have flags for each field in the `MyRequest` message.

If you want to customise the behavior of a command, you can define a custom command by implementing the `autocli.Command` interface. You can then register the command with the `autocli.Builder` instance for your application.

Similarly, you can define a custom query by implementing the `autocli.Query` interface. You can then register the query with the `autocli.Builder` instance for your application.

To add a custom command or query, you can use the `Builder.AddCustomCommand` or `Builder.AddCustomQuery` methods, respectively. These methods take a `cobra.Command` or `cobra.Command` instance, respectively, which can be used to define the behavior of the command or query.

## Advanced Usage

### Specifying Subcommands

By default, `autocli` generates a command for each method in your gRPC service. However, you can specify subcommands to group related commands together. To specify subcommands, you can use the `autocliv1.ServiceCommandDescriptor` struct.

This example shows how to use the `autocliv1.ServiceCommandDescriptor` struct to group related commands together and specify subcommands in your gRPC service by defining an instance of `autocliv1.ModuleOptions` in your `autocli.go` file.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/bcdf81cbaf8d70c4e4fa763f51292d54aed689fd/x/gov/autocli.go#L9-L27
```

The `AutoCLIOptions()` method in the autocli package allows you to specify the services and sub-commands to be mapped for your app. In the example code, an instance of the `autocliv1.ModuleOptions` struct is defined in the `appmodule.AppModule` implementation located in the `x/gov/autocli.go` file. This configuration groups related commands together and specifies subcommands for each service.

### Positional Arguments

Positional arguments are arguments that are passed to a command without being specified as a flag. They are typically used for providing additional context to a command, such as a filename or search query.

To add positional arguments to a command, you can use the `autocliv1.PositionalArgDescriptor` struct, as seen in the example below. You need to specify the `ProtoField` parameter, which is the name of the protobuf field that should be used as the positional argument. In addition, if the parameter is a variable-length argument, you can specify the `Varargs` parameter as `true`. This can only be applied to the last positional parameter, and the `ProtoField` must be a repeated field.

Here's an example of how to define a positional argument for the `Account` method of the `auth` service:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/bcdf81cbaf8d70c4e4fa763f51292d54aed689fd/x/auth/autocli.go#L8-L32
```

Here are some example commands that use the positional arguments we defined above:

To query an account by address:

```bash
<appd> query auth account cosmos1abcd...xyz
```

To query an account address by account number:

```bash
<appd> query auth address-by-acc-num 1
```

In both of these commands, the `auth` service is being queried with the `query` subcommand, followed by the specific method being called (`account` or `address-by-acc-num`). The positional argument is included at the end of the command (`cosmos1abcd...xyz` or `1`) to specify the address or account number, respectively.

### Customising Flag Names

By default, `autocli` generates flag names based on the names of the fields in your protobuf message. However, you can customise the flag names by providing a `FlagOptions` parameter to the `Builder.AddMessageFlags()` method. This parameter allows you to specify custom names for flags based on the names of the message fields. For example, if you have a message with the fields `test` and `test1`, you can use the following naming options to customise the flags

``` go
options := autocliv1.RpcCommandOptions{ 
    FlagOptions: map[string]*autocliv1.FlagOptions{ 
        "test": { Name: "custom_name", }, 
        "test1": { Name: "other_name", }, 
    }, 
}

builder.AddMessageFlags(message, options)
```

Note that `autocliv1.RpcCommandOptions` is a field of the `autocliv1.ServiceCommandDescriptor` struct, which is defined in the `autocliv1` package. To use this option, you can define an instance of `autocliv1.ModuleOptions` in your `appmodule.AppModule` implementation and specify the `FlagOptions` for the relevant service command descriptor.

## Conclusion

`autocli` is a powerful tool for adding CLI interfaces to your Cosmos SDK-based applications. It allows you to easily generate CLI commands and flags from your protobuf messages, and provides many options for customising the behavior of your CLI application.

To further enhance your CLI experience with Cosmos SDK-based blockchains, you can use `Hubl`. `Hubl` is a tool that allows you to query any Cosmos SDK-based blockchain using the new AutoCLI feature of the Cosmos SDK. With hubl, you can easily configure a new chain and query modules with just a few simple commands.

For more information on `Hubl`, including how to configure a new chain and query a module, see the [Hubl documentation](https://docs.cosmos.network/main/tooling/hubl).
