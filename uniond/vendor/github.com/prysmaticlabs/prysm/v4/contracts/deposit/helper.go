package deposit

import (
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
)

// NewDepositContractCallerFromBoundContract creates a new instance of DepositContractCaller, bound to
// a specific deployed contract.
func NewDepositContractCallerFromBoundContract(contract *bind.BoundContract) DepositContractCaller {
	return DepositContractCaller{contract: contract}
}

// NewDepositContractTransactorFromBoundContract creates a new instance of
// DepositContractTransactor, bound to a specific deployed contract.
func NewDepositContractTransactorFromBoundContract(contract *bind.BoundContract) DepositContractTransactor {
	return DepositContractTransactor{contract: contract}
}

// NewDepositContractFiltererFromBoundContract creates a new instance of
// DepositContractFilterer, bound to a specific deployed contract.
func NewDepositContractFiltererFromBoundContract(contract *bind.BoundContract) DepositContractFilterer {
	return DepositContractFilterer{contract: contract}
}
