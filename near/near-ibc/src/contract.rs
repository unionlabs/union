use ibc_vm_rs::{states::CreateClient, IbcHost, IbcResponse, IbcState, Runnable, Status};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract, near_bindgen,
    store::{unordered_map, UnorderedMap},
    AccountId, BorshStorageKey, PanicOnDefault, Promise,
};
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;
use unionlabs::{
    encoding::{Decode, Encode, Proto},
    ibc::core::{client::height::Height, commitment::merkle_path::MerklePath, connection},
};

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

    fn commit_raw(&mut self, key: String, value: Vec<u8>) {
        self.commitments.insert(key, value);
    }

    fn next_connection_identifier(&mut self) -> String {
        self.connection_index += 1;
        format!("connection-{}", self.connection_index)
    }

    fn client_state(&self, client_id: &str) -> Option<Vec<u8>> {
        self.commitments
            .get(&format!("clients/{client_id}/clientState"))
            .map(|item| item.clone())
    }

    fn read<T: Decode<Proto>>(&self, _key: &str) -> Option<T> {
        todo!()
    }

    fn commit<T: Encode<Proto>>(&mut self, key: String, value: T) {
        self.commitments.insert(key, value.encode());
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Owner)]
pub struct Contract {
    commitments: UnorderedMap<String, Vec<u8>>,
    client_index: u64,
    connection_index: u64,
    account_ids: UnorderedMap<String, AccountId>,
    // client id -> account id
    clients: UnorderedMap<String, AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        Contract {
            commitments: UnorderedMap::new(b"commitments".as_slice()),
            client_index: 0,
            account_ids: UnorderedMap::new(b"account_ids".as_slice()),
            clients: UnorderedMap::new(b"clients".as_slice()),
            connection_index: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn register_client(&mut self, client_type: String, account: String) {
        let account_id: AccountId = account.try_into().unwrap();
        match self.account_ids.entry(client_type) {
            unordered_map::Entry::Occupied(_) => panic!("already registered"),
            unordered_map::Entry::Vacant(entry) => {
                entry.insert(account_id);
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

    pub fn connection_open_init(
        &mut self,
        client_id: String,
        counterparty: connection::counterparty::Counterparty<String, String>,
        version: connection::version::Version,
        delay_period: u64,
    ) -> Promise {
        let runnable =
            ibc_vm_rs::connection_open_init(client_id, counterparty, version, delay_period);
        fold(self, runnable, IbcResponse::Empty).unwrap()
    }
    pub fn connection_open_try(
        &mut self,
        client_id: String,
        counterparty: connection::counterparty::Counterparty<String, String>,
        counterparty_versions: Vec<connection::version::Version>,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
        delay_period: u64,
    ) -> Promise {
        let runnable = ibc_vm_rs::connection_open_try(
            client_id,
            counterparty,
            counterparty_versions,
            connection_end_proof,
            proof_height,
            delay_period,
        );
        fold(self, runnable, IbcResponse::Empty).unwrap()
    }

    pub fn connection_open_ack(
        &mut self,
        connection_id: String,
        version: connection::version::Version,
        counterparty_connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    ) -> Promise {
        let runnable = ibc_vm_rs::connection_open_ack(
            connection_id,
            version,
            counterparty_connection_id,
            connection_end_proof,
            proof_height,
        );
        fold(self, runnable, IbcResponse::Empty).unwrap()
    }

    pub fn connection_open_confirm(
        &mut self,
        connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    ) -> Promise {
        let runnable =
            ibc_vm_rs::connection_open_confirm(connection_id, connection_end_proof, proof_height);
        fold(self, runnable, IbcResponse::Empty).unwrap()
    }

    // TODO(aeryz): these getter functions are temporary since for some reason `view_state` won't work
    // when I try to fetch the contract state
    pub fn get_account_id(&self, client_type: String) -> Option<AccountId> {
        self.account_ids.get(&client_type).map(|item| item.clone())
    }

    pub fn get_commitment(&self, key: String) -> Option<Vec<u8>> {
        self.commitments.get(&key).map(|item| item.clone())
    }

    #[private]
    pub fn callback_initialize(&mut self, current_state: Vec<u8>) -> Promise {
        let current_state: IbcState = serde_json::from_slice(&current_state).unwrap();
        match &current_state {
            IbcState::CreateClient(CreateClient::Initialize {
                client_id,
                client_type,
                ..
            }) => {
                let account_id = self.account_ids.get(client_type).unwrap();
                let _ = self.clients.insert(client_id.clone(), account_id.clone());
            }
            _ => panic!("wut?"),
        };
        fold(self, current_state, IbcResponse::Initialize).unwrap()
    }

    #[private]
    pub fn callback_status(
        &mut self,
        current_state: Vec<u8>,
        #[callback_unwrap] status: Status,
    ) -> Option<Promise> {
        let current_state: IbcState = serde_json::from_slice(&current_state).unwrap();
        fold(self, current_state, IbcResponse::Status { status })
    }

    #[private]
    pub fn callback_height(
        &mut self,
        current_state: Vec<u8>,
        #[callback_unwrap] height: Height,
    ) -> Option<Promise> {
        let current_state: IbcState = serde_json::from_slice(&current_state).unwrap();
        fold(self, current_state, IbcResponse::LatestHeight { height })
    }

    #[private]
    pub fn callback_verify_membership(
        &mut self,
        current_state: Vec<u8>,
        #[callback_unwrap] valid: bool,
    ) -> Option<Promise> {
        let current_state: IbcState = serde_json::from_slice(&current_state).unwrap();
        fold(self, current_state, IbcResponse::VerifyMembership { valid })
    }
}

// TODO(aeryz): i hate naming lol
pub fn fold(host: &mut Contract, runnable: IbcState, response: IbcResponse) -> Option<Promise> {
    let either = runnable.process(host, response).unwrap();

    let (runnable, ibc_msg) = match either {
        ibc_vm_rs::Either::Left(cont) => cont,
        ibc_vm_rs::Either::Right(event) => {
            // TODO(aeryz): emit event
            env::log_str(&serde_json::to_string(&event).unwrap());
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
            return Some(
                light_client::ext(account_id.clone()).status().then(
                    Contract::ext(env::current_account_id())
                        .callback_status(serde_json::to_vec(&runnable).unwrap()),
                ),
            );
        }
        ibc_vm_rs::IbcMsg::LatestHeight { client_id } => {
            let account_id = host.clients.get(&client_id).unwrap();
            return Some(
                light_client::ext(account_id.clone()).latest_height().then(
                    Contract::ext(env::current_account_id())
                        .callback_height(serde_json::to_vec(&runnable).unwrap()),
                ),
            );
        }
        ibc_vm_rs::IbcMsg::VerifyMembership {
            client_id,
            height,
            delay_time_period,
            delay_block_period,
            proof,
            path,
            value,
        } => {
            let account_id = host.clients.get(&client_id).unwrap();
            return Some(
                light_client::ext(account_id.clone())
                    .verify_membership(
                        client_id,
                        height,
                        delay_time_period,
                        delay_block_period,
                        proof,
                        path,
                        value,
                    )
                    .then(
                        Contract::ext(env::current_account_id())
                            .callback_height(serde_json::to_vec(&runnable).unwrap()),
                    ),
            );
        }
    }
}

#[ext_contract(light_client)]
pub trait LightClient {
    fn initialize(client_id: String, client_state: Vec<u8>, consensus_state: Vec<u8>) -> Self;

    fn status(&self) -> Status;

    fn latest_height(&self) -> Height;

    fn verify_membership(
        &self,
        client_id: String,
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    ) -> bool;
}
