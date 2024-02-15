package params

// InteropConfig provides a generic config suitable for interop testing.
func InteropConfig() *BeaconChainConfig {
	c := MainnetConfig().Copy()

	// Prysm constants.
	c.ConfigName = InteropName
	c.GenesisForkVersion = []byte{0, 0, 0, 235}
	c.AltairForkVersion = []byte{1, 0, 0, 235}
	c.BellatrixForkVersion = []byte{2, 0, 0, 235}
	c.CapellaForkVersion = []byte{3, 0, 0, 235}
	c.DenebForkVersion = []byte{4, 0, 0, 235}

	c.InitializeForkSchedule()
	return c
}
