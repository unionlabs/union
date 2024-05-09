use ibc_vm_rs::{CreateClient, IbcHost, IbcResponse, Status};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract, near_bindgen,
    store::{unordered_map, UnorderedMap},
    AccountId, BorshStorageKey, PanicOnDefault, Promise,
};
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;

#[allow(unused)]
#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    History,
}

// #[event(
//     standard = "x-value-history",
//     version = "1.0.0",
//     serde = "near_sdk::serde"
// )]
// enum ContractEvent {
//     ValueSet { old_value: u32, new_value: u32 },
// }

impl IbcHost for Contract {
    fn next_client_identifier(&mut self, client_type: &String) -> String {
        self.client_index += 1;
        format!("{client_type}-{}", self.client_index)
    }

    fn commit(&mut self, key: String, value: Vec<u8>) {
        self.commitments.insert(key, value);
    }
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize, Owner)]
pub struct Contract {
    commitments: UnorderedMap<String, Vec<u8>>,
    client_index: u64,
    account_ids: UnorderedMap<String, AccountId>,
    // client id -> account id
    clients: UnorderedMap<String, AccountId>,
}

#[near_bindgen]
impl Contract {
    pub fn register_client(&mut self, client_type: String, account: AccountId) {
        match self.account_ids.entry(client_type) {
            unordered_map::Entry::Occupied(_) => panic!("already registered"),
            unordered_map::Entry::Vacant(entry) => {
                entry.insert(account);
            }
        }
    }

    pub fn create_client(
        &mut self,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    ) -> Promise {
        let runnable = ibc_vm_rs::create_client(client_type, client_state, consensus_state);
        fold(self, runnable, IbcResponse::Empty).unwrap()
    }

    #[private]
    pub fn callback_initialize(&mut self, current_state: Vec<u8>) -> Promise {
        let current_state: CreateClient = serde_json::from_slice(&current_state).unwrap();
        fold(self, current_state, IbcResponse::Initialize).unwrap()
    }
}

// TODO(aeryz): i hate naming lol
pub fn fold<'a, T: ibc_vm_rs::Runnable<Contract>>(
    host: &mut Contract,
    runnable: T,
    response: IbcResponse,
) -> Option<Promise> {
    let either = runnable.process(host, response).unwrap();

    let (runnable, ibc_msg) = match either {
        ibc_vm_rs::Either::Left(cont) => cont,
        ibc_vm_rs::Either::Right(_event) => {
            // TODO(aeryz): emit event
            return None;
        }
    };

    match ibc_msg {
        ibc_vm_rs::IbcMsg::Initialize {
            client_id,
            client_type,
            client_state,
            consensus_state,
        } => {
            let account_id = host.account_ids.get(&client_type).unwrap();
            return Some(
                light_client::ext(account_id.clone())
                    .initialize(client_id, client_state, consensus_state)
                    .then(
                        Contract::ext(env::current_account_id())
                            .callback_initialize(serde_json::to_vec(&runnable).unwrap()),
                    ),
            );
        }
        ibc_vm_rs::IbcMsg::Status { client_id } => {
            let account_id = host.clients.get(&client_id).unwrap();
            return Some(light_client::ext(account_id.clone()).status());
        }
        ibc_vm_rs::IbcMsg::LatestHeight { .. } => todo!(),
    }
}

#[ext_contract(light_client)]
pub trait LightClient {
    fn initialize(&self, client_id: String, client_state: Vec<u8>, consensus_state: Vec<u8>);

    fn status(&self) -> Status;
}
