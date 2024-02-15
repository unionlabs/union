package params

// UseHoleskyNetworkConfig uses the Holesky beacon chain specific network config.
func UseHoleskyNetworkConfig() {
	cfg := BeaconNetworkConfig().Copy()
	cfg.ContractDeploymentBlock = 0
	cfg.BootstrapNodes = []string{
		// EF
		"enr:-Ku4QFo-9q73SspYI8cac_4kTX7yF800VXqJW4Lj3HkIkb5CMqFLxciNHePmMt4XdJzHvhrCC5ADI4D_GkAsxGJRLnQBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpAhnTT-AQFwAP__________gmlkgnY0gmlwhLKAiOmJc2VjcDI1NmsxoQORcM6e19T1T9gi7jxEZjk_sjVLGFscUNqAY9obgZaxbIN1ZHCCIyk",
		"enr:-Ku4QPG7F72mbKx3gEQEx07wpYYusGDh-ni6SNkLvOS-hhN-BxIggN7tKlmalb0L5JPoAfqD-akTZ-gX06hFeBEz4WoBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpAhnTT-AQFwAP__________gmlkgnY0gmlwhJK-DYCJc2VjcDI1NmsxoQKLVXFOhp2uX6jeT0DvvDpPcU8FWMjQdR4wMuORMhpX24N1ZHCCIyk",
		"enr:-LK4QPxe-mDiSOtEB_Y82ozvxn9aQM07Ui8A-vQHNgYGMMthfsfOabaaTHhhJHFCBQQVRjBww_A5bM1rf8MlkJU_l68Eh2F0dG5ldHOIAADAAAAAAACEZXRoMpBpt9l0BAFwAAABAAAAAAAAgmlkgnY0gmlwhLKAiOmJc2VjcDI1NmsxoQJu6T9pclPObAzEVQ53DpVQqjadmVxdTLL-J3h9NFoCeIN0Y3CCIyiDdWRwgiMo",
		"enr:-Ly4QGbOw4xNel5EhmDsJJ-QhC9XycWtsetnWoZ0uRy381GHdHsNHJiCwDTOkb3S1Ade0SFQkWJX_pgb3g8Jfh93rvMBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpBpt9l0BAFwAAABAAAAAAAAgmlkgnY0gmlwhJK-DYCJc2VjcDI1NmsxoQOxKv9sv3zKF8GDewgFGGHKP5HCZZpPpTrwl9eXKAWGxIhzeW5jbmV0cwCDdGNwgiMog3VkcIIjKA",
		// Teku
		"enr:-LS4QG0uV4qvcpJ-HFDJRGBmnlD3TJo7yc4jwK8iP7iKaTlfQ5kZvIDspLMJhk7j9KapuL9yyHaZmwTEZqr10k9XumyCEcmHYXR0bmV0c4gAAAAABgAAAIRldGgykGm32XQEAXAAAAEAAAAAAACCaWSCdjSCaXCErK4j-YlzZWNwMjU2azGhAgfWRBEJlb7gAhXIB5ePmjj2b8io0UpEenq1Kl9cxStJg3RjcIIjKIN1ZHCCIyg",
		// Sigma Prime
		"enr:-Le4QLoE1wFHSlGcm48a9ZESb_MRLqPPu6G0vHqu4MaUcQNDHS69tsy-zkN0K6pglyzX8m24mkb-LtBcbjAYdP1uxm4BhGV0aDKQabfZdAQBcAAAAQAAAAAAAIJpZIJ2NIJpcIQ5gR6Wg2lwNpAgAUHQBwEQAAAAAAAAADR-iXNlY3AyNTZrMaEDPMSNdcL92uNIyCsS177Z6KTXlbZakQqxv3aQcWawNXeDdWRwgiMohHVkcDaCI4I",
	}
	OverrideBeaconNetworkConfig(cfg)
}

// HoleskyConfig defines the config for the Holesky beacon chain testnet.
func HoleskyConfig() *BeaconChainConfig {
	cfg := MainnetConfig().Copy()
	cfg.MinGenesisTime = 1695902100
	cfg.GenesisDelay = 300
	cfg.ConfigName = HoleskyName
	cfg.GenesisForkVersion = []byte{0x01, 0x01, 0x70, 0x00}
	cfg.SecondsPerETH1Block = 14
	cfg.DepositChainID = 17000
	cfg.DepositNetworkID = 17000
	cfg.AltairForkEpoch = 0
	cfg.AltairForkVersion = []byte{0x2, 0x1, 0x70, 0x0}
	cfg.BellatrixForkEpoch = 0
	cfg.BellatrixForkVersion = []byte{0x3, 0x1, 0x70, 0x0}
	cfg.CapellaForkEpoch = 256
	cfg.CapellaForkVersion = []byte{0x4, 0x1, 0x70, 0x0}
	cfg.DenebForkEpoch = 29696
	cfg.DenebForkVersion = []byte{0x05, 0x1, 0x70, 0x0}
	cfg.TerminalTotalDifficulty = "0"
	cfg.DepositContractAddress = "0x4242424242424242424242424242424242424242"
	cfg.EjectionBalance = 28000000000
	cfg.InitializeForkSchedule()
	return cfg
}
