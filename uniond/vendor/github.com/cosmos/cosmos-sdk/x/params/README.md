---
sidebar_position: 1
---

# `x/params`

> Note: The Params module has been depreacted in favour of each module housing its own parameters. 

## Abstract

Package params provides a globally available parameter store.

There are two main types, Keeper and Subspace. Subspace is an isolated namespace for a
paramstore, where keys are prefixed by preconfigured spacename. Keeper has a
permission to access all existing spaces.

Subspace can be used by the individual keepers, which need a private parameter store
that the other keepers cannot modify. The params Keeper can be used to add a route to `x/gov` router in order to modify any parameter in case a proposal passes.

The following contents explains how to use params module for master and user modules.

## Contents

* [Keeper](#keeper)
* [Subspace](#subspace)
    * [Key](#key)
    * [KeyTable](#keytable)
    * [ParamSet](#paramset)

## Keeper

In the app initialization stage, [subspaces](#subspace) can be allocated for other modules' keeper using `Keeper.Subspace` and are stored in `Keeper.spaces`. Then, those modules can have a reference to their specific parameter store through `Keeper.GetSubspace`.

Example:

```go
type ExampleKeeper struct {
	paramSpace paramtypes.Subspace
}

func (k ExampleKeeper) SetParams(ctx sdk.Context, params types.Params) {
	k.paramSpace.SetParamSet(ctx, &params)
}
```

## Subspace

`Subspace` is a prefixed subspace of the parameter store. Each module which uses the
parameter store will take a `Subspace` to isolate permission to access.

### Key

Parameter keys are human readable alphanumeric strings. A parameter for the key
`"ExampleParameter"` is stored under `[]byte("SubspaceName" + "/" + "ExampleParameter")`,
	where `"SubspaceName"` is the name of the subspace.

Subkeys are secondary parameter keys those are used along with a primary parameter key.
Subkeys can be used for grouping or dynamic parameter key generation during runtime.

### KeyTable

All of the parameter keys that will be used should be registered at the compile
time. `KeyTable` is essentially a `map[string]attribute`, where the `string` is a parameter key.

Currently, `attribute` consists of a `reflect.Type`, which indicates the parameter
type to check that provided key and value are compatible and registered, as well as a function `ValueValidatorFn` to validate values.

Only primary keys have to be registered on the `KeyTable`. Subkeys inherit the
attribute of the primary key.

### ParamSet

Modules often define parameters as a proto message. The generated struct can implement
`ParamSet` interface to be used with the following methods:

* `KeyTable.RegisterParamSet()`: registers all parameters in the struct
* `Subspace.{Get, Set}ParamSet()`: Get to & Set from the struct

The implementor should be a pointer in order to use `GetParamSet()`.
