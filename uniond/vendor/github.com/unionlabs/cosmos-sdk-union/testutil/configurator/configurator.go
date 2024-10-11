package configurator

import (
	accountsmodulev1 "cosmossdk.io/api/cosmos/accounts/module/v1"
	runtimev1alpha1 "cosmossdk.io/api/cosmos/app/runtime/v1alpha1"
	appv1alpha1 "cosmossdk.io/api/cosmos/app/v1alpha1"
	authmodulev1 "cosmossdk.io/api/cosmos/auth/module/v1"
	authzmodulev1 "cosmossdk.io/api/cosmos/authz/module/v1"
	bankmodulev1 "cosmossdk.io/api/cosmos/bank/module/v1"
	circuitmodulev1 "cosmossdk.io/api/cosmos/circuit/module/v1"
	consensusmodulev1 "cosmossdk.io/api/cosmos/consensus/module/v1"
	countermodulev1 "cosmossdk.io/api/cosmos/counter/module/v1"
	distrmodulev1 "cosmossdk.io/api/cosmos/distribution/module/v1"
	epochsmodulev1 "cosmossdk.io/api/cosmos/epochs/module/v1"
	evidencemodulev1 "cosmossdk.io/api/cosmos/evidence/module/v1"
	feegrantmodulev1 "cosmossdk.io/api/cosmos/feegrant/module/v1"
	genutilmodulev1 "cosmossdk.io/api/cosmos/genutil/module/v1"
	govmodulev1 "cosmossdk.io/api/cosmos/gov/module/v1"
	groupmodulev1 "cosmossdk.io/api/cosmos/group/module/v1"
	mintmodulev1 "cosmossdk.io/api/cosmos/mint/module/v1"
	nftmodulev1 "cosmossdk.io/api/cosmos/nft/module/v1"
	paramsmodulev1 "cosmossdk.io/api/cosmos/params/module/v1"
	poolmodulev1 "cosmossdk.io/api/cosmos/protocolpool/module/v1"
	slashingmodulev1 "cosmossdk.io/api/cosmos/slashing/module/v1"
	stakingmodulev1 "cosmossdk.io/api/cosmos/staking/module/v1"
	txconfigv1 "cosmossdk.io/api/cosmos/tx/config/v1"
	validatemodulev1 "cosmossdk.io/api/cosmos/validate/module/v1"
	vestingmodulev1 "cosmossdk.io/api/cosmos/vesting/module/v1"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"

	"github.com/cosmos/cosmos-sdk/testutil"
	_ "github.com/cosmos/cosmos-sdk/x/auth"           // import as blank for app wiring
	_ "github.com/cosmos/cosmos-sdk/x/auth/tx/config" // import as blank for app wiring
	_ "github.com/cosmos/cosmos-sdk/x/genutil"        // import as blank for app wiring
	_ "github.com/cosmos/cosmos-sdk/x/validate"       // import as blank for app wiring
)

// Config should never need to be instantiated manually and is solely used for ModuleOption.
type Config struct {
	ModuleConfigs      map[string]*appv1alpha1.ModuleConfig
	PreBlockersOrder   []string
	BeginBlockersOrder []string
	EndBlockersOrder   []string
	InitGenesisOrder   []string
	setInitGenesis     bool
}

func defaultConfig() *Config {
	return &Config{
		ModuleConfigs: make(map[string]*appv1alpha1.ModuleConfig),
		PreBlockersOrder: []string{
			testutil.UpgradeModuleName,
		},
		BeginBlockersOrder: []string{
			testutil.MintModuleName,
			testutil.DistributionModuleName,
			testutil.ProtocolPoolModuleName,
			testutil.SlashingModuleName,
			testutil.EvidenceModuleName,
			testutil.StakingModuleName,
			testutil.AuthModuleName,
			testutil.BankModuleName,
			testutil.GovModuleName,
			"genutil",
			testutil.AuthzModuleName,
			testutil.FeegrantModuleName,
			testutil.NFTModuleName,
			testutil.GroupModuleName,
			"consensus",
			testutil.ParamsModuleName,
			"vesting",
			testutil.CircuitModuleName,
			testutil.EpochsModuleName,
		},
		EndBlockersOrder: []string{
			testutil.GovModuleName,
			testutil.StakingModuleName,
			testutil.AuthModuleName,
			testutil.BankModuleName,
			testutil.DistributionModuleName,
			testutil.SlashingModuleName,
			testutil.MintModuleName,
			"genutil",
			testutil.EvidenceModuleName,
			testutil.AuthzModuleName,
			testutil.FeegrantModuleName,
			testutil.NFTModuleName,
			testutil.GroupModuleName,
			"consensus",
			testutil.UpgradeModuleName,
			"vesting",
			testutil.CircuitModuleName,
			testutil.ProtocolPoolModuleName,
		},
		InitGenesisOrder: []string{
			testutil.AccountsModuleName,
			testutil.AuthModuleName,
			testutil.BankModuleName,
			testutil.DistributionModuleName,
			testutil.StakingModuleName,
			testutil.SlashingModuleName,
			testutil.GovModuleName,
			testutil.MintModuleName,
			"genutil",
			testutil.EvidenceModuleName,
			testutil.AuthzModuleName,
			testutil.FeegrantModuleName,
			testutil.NFTModuleName,
			testutil.GroupModuleName,
			"consensus",
			testutil.UpgradeModuleName,
			"vesting",
			testutil.CircuitModuleName,
			testutil.ProtocolPoolModuleName,
			testutil.EpochsModuleName,
		},
		setInitGenesis: true,
	}
}

type ModuleOption func(config *Config)

func WithCustomPreBlockersOrder(preBlockOrder ...string) ModuleOption {
	return func(config *Config) {
		config.PreBlockersOrder = preBlockOrder
	}
}

func WithCustomBeginBlockersOrder(beginBlockOrder ...string) ModuleOption {
	return func(config *Config) {
		config.BeginBlockersOrder = beginBlockOrder
	}
}

func WithCustomEndBlockersOrder(endBlockersOrder ...string) ModuleOption {
	return func(config *Config) {
		config.EndBlockersOrder = endBlockersOrder
	}
}

func WithCustomInitGenesisOrder(initGenesisOrder ...string) ModuleOption {
	return func(config *Config) {
		config.InitGenesisOrder = initGenesisOrder
	}
}

func BankModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.BankModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.BankModuleName,
			Config: appconfig.WrapAny(&bankmodulev1.Module{}),
		}
	}
}

func AuthModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.AuthModuleName] = &appv1alpha1.ModuleConfig{
			Name: testutil.AuthModuleName,
			Config: appconfig.WrapAny(&authmodulev1.Module{
				Bech32Prefix: "cosmos",
				ModuleAccountPermissions: []*authmodulev1.ModuleAccountPermission{
					{Account: "fee_collector"},
					{Account: testutil.DistributionModuleName},
					{Account: testutil.MintModuleName, Permissions: []string{"minter"}},
					{Account: "bonded_tokens_pool", Permissions: []string{"burner", testutil.StakingModuleName}},
					{Account: "not_bonded_tokens_pool", Permissions: []string{"burner", testutil.StakingModuleName}},
					{Account: testutil.GovModuleName, Permissions: []string{"burner"}},
					{Account: testutil.NFTModuleName},
					{Account: testutil.ProtocolPoolModuleName},
					{Account: "stream_acc"},
					{Account: "protocolpool_distr"},
				},
			}),
		}
	}
}

func ParamsModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.ParamsModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.ParamsModuleName,
			Config: appconfig.WrapAny(&paramsmodulev1.Module{}),
		}
	}
}

func TxModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.AuthTxConfigDepinjectModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.AuthTxConfigDepinjectModuleName,
			Config: appconfig.WrapAny(&txconfigv1.Config{}),
		}
	}
}

func ValidateModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.ValidateModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.ValidateModuleName,
			Config: appconfig.WrapAny(&validatemodulev1.Module{}),
		}
	}
}

func StakingModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.StakingModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.StakingModuleName,
			Config: appconfig.WrapAny(&stakingmodulev1.Module{}),
		}
	}
}

func SlashingModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.SlashingModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.SlashingModuleName,
			Config: appconfig.WrapAny(&slashingmodulev1.Module{}),
		}
	}
}

func GenutilModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs["genutil"] = &appv1alpha1.ModuleConfig{
			Name:   "genutil",
			Config: appconfig.WrapAny(&genutilmodulev1.Module{}),
		}
	}
}

func DistributionModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.DistributionModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.DistributionModuleName,
			Config: appconfig.WrapAny(&distrmodulev1.Module{}),
		}
	}
}

func FeegrantModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.FeegrantModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.FeegrantModuleName,
			Config: appconfig.WrapAny(&feegrantmodulev1.Module{}),
		}
	}
}

func VestingModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs["vesting"] = &appv1alpha1.ModuleConfig{
			Name:   "vesting",
			Config: appconfig.WrapAny(&vestingmodulev1.Module{}),
		}
	}
}

func GovModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.GovModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.GovModuleName,
			Config: appconfig.WrapAny(&govmodulev1.Module{}),
		}
	}
}

func ConsensusModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs["consensus"] = &appv1alpha1.ModuleConfig{
			Name:   "consensus",
			Config: appconfig.WrapAny(&consensusmodulev1.Module{}),
		}
	}
}

func MintModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.MintModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.MintModuleName,
			Config: appconfig.WrapAny(&mintmodulev1.Module{}),
			GolangBindings: []*appv1alpha1.GolangBinding{
				{
					InterfaceType:  "cosmossdk.io/x/mint/types/types.StakingKeeper",
					Implementation: "cosmossdk.io/x/staking/keeper/*keeper.Keeper",
				},
			},
		}
	}
}

func EvidenceModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.EvidenceModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.EvidenceModuleName,
			Config: appconfig.WrapAny(&evidencemodulev1.Module{}),
		}
	}
}

func AuthzModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.AuthzModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.AuthzModuleName,
			Config: appconfig.WrapAny(&authzmodulev1.Module{}),
		}
	}
}

func GroupModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.GroupModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.GroupModuleName,
			Config: appconfig.WrapAny(&groupmodulev1.Module{}),
		}
	}
}

func NFTModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.NFTModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.NFTModuleName,
			Config: appconfig.WrapAny(&nftmodulev1.Module{}),
		}
	}
}

func CircuitModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.CircuitModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.CircuitModuleName,
			Config: appconfig.WrapAny(&circuitmodulev1.Module{}),
		}
	}
}

func ProtocolPoolModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.ProtocolPoolModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.ProtocolPoolModuleName,
			Config: appconfig.WrapAny(&poolmodulev1.Module{}),
		}
	}
}

func AccountsModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.AccountsModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.AccountsModuleName,
			Config: appconfig.WrapAny(&accountsmodulev1.Module{}),
		}
	}
}

func CounterModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs["counter"] = &appv1alpha1.ModuleConfig{
			Name:   "counter",
			Config: appconfig.WrapAny(&countermodulev1.Module{}),
		}
	}
}

func EpochsModule() ModuleOption {
	return func(config *Config) {
		config.ModuleConfigs[testutil.EpochsModuleName] = &appv1alpha1.ModuleConfig{
			Name:   testutil.EpochsModuleName,
			Config: appconfig.WrapAny(&epochsmodulev1.Module{}),
		}
	}
}

func OmitInitGenesis() ModuleOption {
	return func(config *Config) {
		config.setInitGenesis = false
	}
}

func NewAppConfig(opts ...ModuleOption) depinject.Config {
	cfg := defaultConfig()
	for _, opt := range opts {
		opt(cfg)
	}

	preBlockers := make([]string, 0)
	beginBlockers := make([]string, 0)
	endBlockers := make([]string, 0)
	initGenesis := make([]string, 0)
	overrides := make([]*runtimev1alpha1.StoreKeyConfig, 0)

	for _, s := range cfg.PreBlockersOrder {
		if _, ok := cfg.ModuleConfigs[s]; ok {
			preBlockers = append(preBlockers, s)
		}
	}

	for _, s := range cfg.BeginBlockersOrder {
		if _, ok := cfg.ModuleConfigs[s]; ok {
			beginBlockers = append(beginBlockers, s)
		}
	}

	for _, s := range cfg.EndBlockersOrder {
		if _, ok := cfg.ModuleConfigs[s]; ok {
			endBlockers = append(endBlockers, s)
		}
	}

	for _, s := range cfg.InitGenesisOrder {
		if _, ok := cfg.ModuleConfigs[s]; ok {
			initGenesis = append(initGenesis, s)
		}
	}

	if _, ok := cfg.ModuleConfigs[testutil.AuthModuleName]; ok {
		overrides = append(overrides, &runtimev1alpha1.StoreKeyConfig{ModuleName: testutil.AuthModuleName, KvStoreKey: "acc"})
	}

	runtimeConfig := &runtimev1alpha1.Module{
		AppName:           "TestApp",
		PreBlockers:       preBlockers,
		BeginBlockers:     beginBlockers,
		EndBlockers:       endBlockers,
		OverrideStoreKeys: overrides,
	}
	if cfg.setInitGenesis {
		runtimeConfig.InitGenesis = initGenesis
	}

	modules := []*appv1alpha1.ModuleConfig{{
		Name:   "runtime",
		Config: appconfig.WrapAny(runtimeConfig),
	}}

	for _, m := range cfg.ModuleConfigs {
		modules = append(modules, m)
	}

	return appconfig.Compose(&appv1alpha1.Config{Modules: modules})
}
