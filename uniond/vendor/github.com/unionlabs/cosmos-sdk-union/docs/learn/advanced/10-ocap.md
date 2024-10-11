---
sidebar_position: 1
---

# Object-Capability Model

## Intro

When thinking about security, it is good to start with a specific threat model. Our threat model is the following:

> We assume that a thriving ecosystem of Cosmos SDK modules that are easy to compose into a blockchain application will contain faulty or malicious modules.

The Cosmos SDK is designed to address this threat by being the
foundation of an object capability system.

> The structural properties of object capability systems favor
> modularity in code design and ensure reliable encapsulation in
> code implementation.
>
> These structural properties facilitate the analysis of some
> security properties of an object-capability program or operating
> system. Some of these — in particular, information flow properties
> — can be analyzed at the level of object references and
> connectivity, independent of any knowledge or analysis of the code
> that determines the behavior of the objects.
>
> As a consequence, these security properties can be established
> and maintained in the presence of new objects that contain unknown
> and possibly malicious code.
>
> These structural properties stem from the two rules governing
> access to existing objects:
>
> 1. An object A can send a message to B only if object A holds a
>     reference to B.
> 2. An object A can obtain a reference to C only
>     if object A receives a message containing a reference to C. As a
>     consequence of these two rules, an object can obtain a reference
>     to another object only through a preexisting chain of references.
>     In short, "Only connectivity begets connectivity."

For an introduction to object-capabilities, see this [Wikipedia article](https://en.wikipedia.org/wiki/Object-capability_model).

## Ocaps in practice

The idea is to only reveal what is necessary to get the work done.

For example, the following code snippet violates the object capabilities
principle:

```go
type AppAccount struct {...}
account := &AppAccount{
    Address: pub.Address(),
    Coins: sdk.Coins{sdk.NewInt64Coin("ATM", 100)},
}
sumValue := externalModule.ComputeSumValue(account)
```

The method `ComputeSumValue` implies a pure function, yet the implied
capability of accepting a pointer value is the capability to modify that
value. The preferred method signature should take a copy instead.

```go
sumValue := externalModule.ComputeSumValue(*account)
```

In the Cosmos SDK, you can see the application of this principle in simapp.

https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app.go

The following diagram shows the current dependencies between keepers.

![Keeper dependencies](https://raw.githubusercontent.com/cosmos/cosmos-sdk/release/v0.46.x/docs/uml/svg/keeper_dependencies.svg)
