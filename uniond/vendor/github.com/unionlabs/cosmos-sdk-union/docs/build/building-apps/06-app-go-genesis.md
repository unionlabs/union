---
sidebar_position: 1
---

### Modifying the `DefaultGenesis`

It is possible to modify the DefaultGenesis parameters for modules by wrapping the module, providing it to the `*module.Manager` and injecting it with `depinject`.

Example ( staking ) :

```go
type CustomStakingModule struct {
    staking.AppModule
    cdc codec.Codec
}

// DefaultGenesis will override the Staking module DefaultGenesis AppModuleBasic method.
func (cm CustomStakingModule) DefaultGenesis() json.RawMessage {
    params := stakingtypes.DefaultParams()
    params.BondDenom = "mydenom"

    return cm.cdc.MustMarshalJSON(&stakingtypes.GenesisState{
        Params: params,
    })
}

// option 1 ( for depinject users ): override previous module manager
depinject.Inject(
// ... provider/invoker/supplier
&moduleManager,
)

oldStakingModule,_ := moduleManager.Modules()[stakingtypes.ModuleName].(staking.AppModule)
moduleManager.Modules()[stakingtypes.ModuleName] = CustomStakingModule{
	AppModule: oldStakingModule,
	cdc: appCodec,
}

// option 2 ( for non depinject users ): use new module manager
moduleManager := module.NewManagerFromMap(map[string]appmodule.AppModule{
stakingtypes.ModuleName: CustomStakingModule{cdc: appCodec, AppModule: staking.NewAppModule(...)},
// other modules ...
})

// set the module manager
app.ModuleManager = moduleManager
```
