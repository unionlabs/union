use std::{collections::HashMap, convert::Infallible};

use alloy_primitives::Log;
use alloy_sol_macro::sol;
use alloy_sol_types::{
    SolCall, SolConstructor, SolError, SolEvent, SolEventInterface, SolInterface,
};
use reth_primitives::{Address, StorageKey, B256, U256};
use reth_revm::{
    database::{EvmStateProvider, StateProviderDatabase},
    test_utils::StateProviderTest,
};
// use reth_rpc_eth_api::core::EthApi;
use revm::primitives::{
    db::DatabaseCommit, keccak256, ruint::Uint, Account, AccountInfo, AccountStatus, Bytecode,
    ExecutionResult, TxKind, KECCAK_EMPTY,
};
use revm::{primitives::hex, Database};

use crate::Counter::{
    constructorCall, incCall, CounterErrors, CounterEvents, ErrIncTooManyTimes, Inc,
};

#[derive(Debug, Default)]
pub struct InMemoryDb {
    accounts: HashMap<Address, Account>,
    contracts: HashMap<B256, Address>,
    block_hash: HashMap<u64, B256>,
}

impl Database for InMemoryDb {
    type Error = Infallible;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self.accounts.get(&address).map(|acc| acc.info.clone()))
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(self
            .contracts
            .get(&code_hash)
            .and_then(|address| self.accounts.get(address))
            .and_then(|acc| acc.info.code.clone())
            .unwrap_or_default())
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .accounts
            .get(&address)
            .and_then(|x| x.storage.get(&index))
            .map(|x| x.present_value)
            .unwrap_or_default())
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        Ok(*self.block_hash.get(&number).unwrap_or_default())
    }
}

impl DatabaseCommit for InMemoryDb {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        for (address, account) in changes {
            self.accounts
                .entry(address)
                .and_modify(|existing_account| {
                    if account.info.code.is_some()
                        && AccountStatus::Created.contains(account.status)
                    {
                        self.contracts.insert(account.info.code_hash, address);
                    }
                    existing_account.info = account.info.clone();
                    for (slot, value) in account.changed_storage_slots() {
                        existing_account.storage.insert(*slot, value.clone());
                    }
                })
                .or_insert(account);
        }
    }
}

#[test]
fn test() {
    do_test();
}

fn do_test() {
    let alice = Address::from_word(keccak256("alice"));

    let mut evm = Evm::new([(alice, U256::MAX)]);

    let counter_address = evm
        .deploy_contract(
            alice,
            BYTECODE.to_vec(),
            constructorCall {
                init: Uint::from(100),
            },
        )
        .unwrap();

    dbg!(counter_address);

    assert_eq!(
        evm.contract_call::<_, CounterEvents, CounterErrors>(
            alice,
            counter_address,
            incCall { times: 3 },
        )
        .unwrap()
        .0,
        vec![
            Log {
                address: counter_address,
                data: CounterEvents::Inc(Inc {
                    newValue: Uint::from(101)
                })
            },
            Log {
                address: counter_address,
                data: CounterEvents::Inc(Inc {
                    newValue: Uint::from(102)
                })
            },
            Log {
                address: counter_address,
                data: CounterEvents::Inc(Inc {
                    newValue: Uint::from(103)
                })
            },
        ],
    );

    assert_eq!(
        evm.contract_call::<_, CounterEvents, CounterErrors>(
            alice,
            counter_address,
            incCall { times: 20 },
        )
        .unwrap_err(),
        CounterErrors::ErrIncTooManyTimes(ErrIncTooManyTimes { times: 20 }),
    );
}

pub struct Evm<'a> {
    evm: revm::Evm<'a, (), InMemoryDb>,
}

impl Evm<'_> {
    pub fn new(accounts: impl IntoIterator<Item = (Address, U256)>) -> Self {
        let mut db = InMemoryDb::default();

        for (address, balance) in accounts {
            db.accounts.insert(
                address,
                Account {
                    info: AccountInfo {
                        nonce: 1,
                        balance,
                        code_hash: KECCAK_EMPTY,
                        code: None,
                    },
                    storage: Default::default(),
                    status: AccountStatus::Loaded,
                },
            );
        }

        let evm = revm::Evm::builder().with_db(db).build();

        Self { evm }
    }

    fn deploy_contract<Constructor: SolConstructor>(
        &mut self,
        from: Address,
        mut bytecode: Vec<u8>,
        constructor: Constructor,
    ) -> Result<Address, Vec<u8>> {
        bytecode.extend(constructor.abi_encode());

        self.evm.tx_mut().caller = from;
        self.evm.tx_mut().transact_to = TxKind::Create;
        self.evm.tx_mut().data = bytecode.into();

        let res = self.evm.transact().unwrap();

        match res.result {
            ExecutionResult::Success {
                reason,
                gas_used,
                gas_refunded,
                logs,
                output,
            } => {
                self.evm.db_mut().commit(res.state);
                Ok(*output
                    .address()
                    .expect("code deployment returns an address"))
            }
            ExecutionResult::Revert { gas_used, output } => Err(output.into()),
            ExecutionResult::Halt { reason, gas_used } => Ok(todo!()),
        }
    }

    fn contract_call<Data: SolCall, Event: SolEventInterface, Revert: SolInterface>(
        &mut self,
        from: Address,
        contract_address: Address,
        data: Data,
    ) -> Result<(Vec<Log<Event>>, Vec<u8>), Revert> {
        self.evm.tx_mut().caller = from;
        self.evm.tx_mut().transact_to = TxKind::Call(contract_address);
        self.evm.tx_mut().data = data.abi_encode().into();

        let res = self.evm.transact().unwrap();

        // dbg!(self.evm.db());

        match res.result {
            ExecutionResult::Success {
                reason,
                gas_used,
                gas_refunded,
                logs,
                output,
            } => {
                self.evm.db_mut().commit(res.state);
                Ok((
                    logs.into_iter()
                        .map(|log| Event::decode_log(&log, true).expect("unable to decode log"))
                        .collect(),
                    match output {
                        revm::primitives::Output::Call(output) => output.into(),
                        revm::primitives::Output::Create(_, _) => {
                            panic!("calling a contract deployed code???")
                        }
                    },
                ))
            }
            ExecutionResult::Revert { gas_used, output } => {
                Err(Revert::abi_decode(&output, true).expect("unknown revert"))
            }
            ExecutionResult::Halt { reason, gas_used } => Ok(todo!()),
        }
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    contract Counter {
        event Inc(uint);
        event Dec(uint);

        error ErrIncTooManyTimes(uint8 times);

        uint public count;

        constructor(uint init) {
            count = init;
        }

        // Function to get the current count
        function get() public view returns (uint) {
            return count;
        }

        // Function to increment count by 1
        function inc(uint8 times) public {
            if (times > 10) {
                revert ErrIncTooManyTimes(times);
            } else {
                for (;times > 0; times--) {
                    count += 1;
                    emit Inc(count);
                }
            }
        }

        // Function to decrement count by 1
        function dec() public {
            count -= 1;
            emit Dec(count);
        }
    }
}

const BYTECODE: &[u8] = &hex!("6080604052348015600e575f80fd5b506040516104163803806104168339818101604052810190602e9190606b565b805f81905550506091565b5f80fd5b5f819050919050565b604d81603d565b81146056575f80fd5b50565b5f815190506065816046565b92915050565b5f60208284031215607d57607c6039565b5b5f6088848285016059565b91505092915050565b6103788061009e5f395ff3fe608060405234801561000f575f80fd5b506004361061004a575f3560e01c806306661abd1461004e5780636d4ce63c1461006c578063b3bcfa821461008a578063fc5842bd14610094575b5f80fd5b6100566100b0565b60405161006391906101e2565b60405180910390f35b6100746100b5565b60405161008191906101e2565b60405180910390f35b6100926100bd565b005b6100ae60048036038101906100a99190610235565b61010f565b005b5f5481565b5f8054905090565b60015f808282546100ce919061028d565b925050819055507f757fff3e831f63e329ee929d928e44a48df56c5abd902d2414c60211a993e37e5f5460405161010591906101e2565b60405180910390a1565b600a8160ff16111561015857806040517fe74246a900000000000000000000000000000000000000000000000000000000815260040161014f91906102cf565b60405180910390fd5b5b5f8160ff1611156101c75760015f8082825461017591906102e8565b925050819055507f3443590b7333fb7cfd5e65585c8a4c4100c345929865db522919623bf37e58085f546040516101ac91906101e2565b60405180910390a180806101bf9061031b565b915050610159565b50565b5f819050919050565b6101dc816101ca565b82525050565b5f6020820190506101f55f8301846101d3565b92915050565b5f80fd5b5f60ff82169050919050565b610214816101ff565b811461021e575f80fd5b50565b5f8135905061022f8161020b565b92915050565b5f6020828403121561024a576102496101fb565b5b5f61025784828501610221565b91505092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f610297826101ca565b91506102a2836101ca565b92508282039050818111156102ba576102b9610260565b5b92915050565b6102c9816101ff565b82525050565b5f6020820190506102e25f8301846102c0565b92915050565b5f6102f2826101ca565b91506102fd836101ca565b925082820190508082111561031557610314610260565b5b92915050565b5f610325826101ff565b91505f820361033757610336610260565b5b60018203905091905056fea2646970667358221220756fc4b018cad6c146571e79e451bd4b6acc78da96ede40416af47a594d271f064736f6c634300081a0033");
