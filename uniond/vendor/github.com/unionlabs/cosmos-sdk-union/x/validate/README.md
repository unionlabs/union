# x/validate

:::tip
This module is only required when using runtime and runtime v2 and you want to make use of the pre-defined ante/poste handlers or tx validators.
:::

The `x/validate` is an app module solely there to setup ante/post handlers on a runtime app (via baseapp options) and the global tx validators on a runtime/v2 app (via app module). Depinject will automatically inject the ante/post handlers and tx validators into the app. Module specific tx validators should be registered on their own modules.

## Extra TxValidators

It is possible to add extra tx validators to the app. This is useful when you want to add extra tx validators that do not belong to one specific module. For example, you can add a tx validator that checks if the tx is signed by a specific address.

In your `app.go`, when using runtime/v2, supply the extra tx validators using `depinject`:

```go
appConfig = depinject.Configs(
	AppConfig(),
	depinject.Supply(
        []appmodulev2.TxValidator[transaction.Tx]{
            // Add extra tx validators here
        }
    ),
)
```

## Storage

This module has no store key. Do not forget to add the module name in the `SkipStoreKeys` runtime config present in the app config.

```go
SkipStoreKeys: []string{
	authtxconfig.DepinjectModuleName,
	validate.ModuleName,
},
```
