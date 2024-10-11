package params

const (
	// Every EL component has an offset that manages which port it is assigned. The miner always gets offset=0.
	MinerComponentOffset = 0
	Eth1StaticFilesPath  = "/testing/endtoend/static-files/eth1"
	minerKeyFilename     = "UTC--2021-12-22T19-14-08.590377700Z--878705ba3f8bc32fcf7f4caa1a35e72af65cf766"
	baseELHost           = "127.0.0.1"
	baseELScheme         = "http"
	// DepositGasLimit is the gas limit used for all deposit transactions. The exact value probably isn't important
	// since these are the only transactions in the e2e run.
	DepositGasLimit = 4000000
	// SpamTxGasLimit is used for the spam transactions (to/from miner address)
	// which WaitForBlocks generates in order to advance the EL chain.
	SpamTxGasLimit = 21000
)
