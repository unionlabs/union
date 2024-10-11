---
sidebar_position: 1
---

# Hooks

Hooks are functions that are called before and/or after certain events in the module's lifecycle.

## Defining Hooks

1. Define the hook interface and a wrapper implementing `depinject.OnePerModuleType`:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/71c603a2a5a103df00f216d78ec8b108ed64ae28/testutil/x/counter/types/expected_keepers.go#L5-L12
    ```

2. Add a `CounterHooks` field to the keeper:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/71c603a2a5a103df00f216d78ec8b108ed64ae28/testutil/x/counter/keeper/keeper.go#L25
    
    ```

3. Create a `depinject` invoker function

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/71c603a2a5a103df00f216d78ec8b108ed64ae28/testutil/x/counter/depinject.go#L53-L75   
    ```

4. Inject the hooks during app initialization:

    ```go
    appConfig = appconfig.Compose(&appv1alpha1.Config{
        Modules: []*appv1alpha1.ModuleConfig{
            // ....
            {
                Name:   types.ModuleName,
                Config: appconfig.WrapAny(&types.Module{}),
            },
        }
    })
    appConfig = depinject.Configs(
        AppConfig(),
        runtime.DefaultServiceBindings(),
        depinject.Supply(
            logger,
            viper,
            map[string]types.CounterHooksWrapper{
                "counter": types.CounterHooksWrapper{&types.Hooks{}},
            },
    ))
    ```

## Examples in the SDK

For examples of hooks implementation in the Cosmos SDK, refer to the [Epochs Hooks documentation](https://docs.cosmos.network/main/build/modules/epochs#hooks) and [Distribution Hooks Documentation](https://docs.cosmos.network/main/build/modules/distribution#hooks). 

