# Coding Guidelines

This document is an extension to [CONTRIBUTING](./CONTRIBUTING.md) and provides more details about the coding guidelines and requirements.

## API & Design

* Code must be well structured:
    * packages must have a limited responsibility (different concerns can go to different packages),
    * types must be easy to compose,
    * think about maintainbility and testability.
* "Depend upon abstractions, [not] concretions".
* Try to limit the number of methods you are exposing. It's easier to expose something later than to hide it.
* Follow agreed-upon design patterns and naming conventions.
* publicly-exposed functions are named logically, have forward-thinking arguments and return types.
* Avoid global variables and global configurators.
* Favor composable and extensible designs.
* Minimize code duplication.
* Limit third-party dependencies.

Performance:

* Avoid unnecessary operations or memory allocations.

Security:

* Pay proper attention to exploits involving:
    * gas usage
    * transaction verification and signatures
    * malleability
    * code must be always deterministic
* Thread safety. If some functionality is not thread-safe, or uses something that is not thread-safe, then clearly indicate the risk on each level.

## Best practices

* Use [goimports](https://pkg.go.dev/golang.org/x/tools/cmd/goimports) as your code formatter.

* Always wrap returned errors. 
    * Doing `if err != nil { return err }` does not include each callers' context. Pushing errors up the stack without context makes it harder to test and debug. Additionally, a short context description makes it easier for the reader to understand the code. Example:
  
        ```go
        if !coins.IsZero() { 
            if err := k.bank.TransferCoins(ctx, caller, contractAddress, coins); err != nil { 
                return nil, err 
            } 
        } 
        ```

    * It would be an improvement to return  `return nil, sdkerror.Wrap(err, "lock contract coins")`
    * Please notice that fmt.Errorf is not used, because the error handling predates fmt.Errorf and errors.Is 
   
* Limit the use of aliases, when not used during the refactoring process.