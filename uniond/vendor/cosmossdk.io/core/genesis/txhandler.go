package genesis

// TxHandler is an interface that modules can implement to provide genesis state transitions
type TxHandler interface {
	ExecuteGenesisTx([]byte) error
}
