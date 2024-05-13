use ibc_vm_rs::{
    states::{connection_handshake, CreateClient},
    IbcHost, IbcResponse, IbcState, Runnable, Status,
};
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
    ibc::core::{channel, client::height::Height, commitment::merkle_path::MerklePath, connection},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    validated::ValidateT,
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
    fn next_client_identifier(&mut self, client_type: &String) -> Result<ClientId, ()> {
        self.client_index += 1;
        Ok(format!("{client_type}-{}", self.client_index)
            .validate()
            .unwrap())
    }

    fn commit_raw(&mut self, key: String, value: Vec<u8>) {
        self.commitments.insert(key, value);
    }

    fn next_connection_identifier(&mut self) -> Result<ConnectionId, ()> {
        self.connection_index += 1;
        Ok(format!("connection-{}", self.connection_index)
            .validate()
            .unwrap())
    }

    fn client_state(&self, client_id: &str) -> Option<Vec<u8>> {
        self.commitments
            .get(&format!("clients/{client_id}/clientState"))
            .map(|item| item.clone())
    }

    fn read<T: Decode<Proto>>(&self, key: &str) -> Option<T> {
        self.commitments
            .get(key)
            .map(|item| T::decode(item).unwrap())
    }

    fn commit<T: Encode<Proto>>(&mut self, key: String, value: T) {
        self.commitments.insert(key, value.encode());
    }

    fn next_channel_identifier(&mut self) -> Result<ChannelId, ()> {
        self.channel_index += 1;
        Ok(format!("channel-{}", self.channel_index)
            .validate()
            .unwrap())
    }

    fn read_raw(&self, key: &str) -> Option<Vec<u8>> {
        self.commitments.get(key).map(|item| item.clone())
    }

    fn current_height(&self) -> Height {
        Height {
            revision_number: 0,
            revision_height: env::block_height(),
        }
    }

    fn current_timestamp(&self) -> u64 {
        // TODO(aeryz): should this be in ms?
        env::block_timestamp()
    }

    fn sha256(&self, data: Vec<u8>) -> Vec<u8> {
        env::sha256(&data)
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Owner)]
pub struct Contract {
    commitments: UnorderedMap<String, Vec<u8>>,
    client_index: u64,
    connection_index: u64,
    channel_index: u64,
    account_ids: UnorderedMap<String, AccountId>,
    // client id -> account id
    clients: UnorderedMap<String, AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        Contract {
            commitments: UnorderedMap::new(b"commitments".as_slice()),
            client_index: 0,
            channel_index: 0,
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
        client_id: ClientId,
        counterparty: connection_handshake::Counterparty,
        version: connection::version::Version,
        delay_period: u64,
    ) -> Promise {
        let runnable =
            ibc_vm_rs::connection_open_init(client_id, counterparty, version, delay_period);
        fold(self, runnable, IbcResponse::Empty).unwrap()
    }
    pub fn connection_open_try(
        &mut self,
        client_id: ClientId,
        counterparty: connection_handshake::Counterparty,
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

    pub fn channel_open_init(
        &mut self,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    ) -> Promise {
        let runnable =
            ibc_vm_rs::channel_open_init(connection_hops, port_id, counterparty, version);
        fold(self, runnable, IbcResponse::Empty).unwrap()
    }

    pub fn channel_open_ack(
        &mut self,
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: Vec<u8>,
        proof_height: Height,
    ) -> Promise {
        let runnable = ibc_vm_rs::channel_open_ack(
            channel_id,
            port_id,
            counterparty_channel_id,
            counterparty_version,
            proof_try,
            proof_height,
        );
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
                // TODO(aeryz): we want to impl borsh serialize for validate types in unionlabs
                let _ = self
                    .clients
                    .insert(client_id.clone().to_string(), account_id.clone());
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

    #[private]
    pub fn callback_on_chan_open_init(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] err: bool,
    ) -> Option<Promise> {
        fold(self, current_state, IbcResponse::OnChannelOpenInit { err })
    }

    #[private]
    pub fn callback_on_chan_open_ack(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] err: bool,
    ) -> Option<Promise> {
        fold(self, current_state, IbcResponse::OnChannelOpenAck { err })
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
            let client_id = client_id.to_string();
            let account_id = host.clients.get(&client_id).unwrap();
            return Some(
                light_client::ext(account_id.clone()).status().then(
                    Contract::ext(env::current_account_id())
                        .callback_status(serde_json::to_vec(&runnable).unwrap()),
                ),
            );
        }
        ibc_vm_rs::IbcMsg::LatestHeight { client_id } => {
            let client_id = client_id.to_string();
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
            let account_id = host.clients.get(&client_id.to_string()).unwrap();
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
                            .callback_verify_membership(serde_json::to_vec(&runnable).unwrap()),
                    ),
            );
        }
        ibc_vm_rs::IbcMsg::OnChannelOpenInit {
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
            version,
        } => {
            let account_id = AccountId::try_from(port_id.to_string()).unwrap();
            Some(
                ibc_app::ext(account_id)
                    .on_channel_open_init(
                        order,
                        connection_hops,
                        port_id,
                        channel_id,
                        counterparty,
                        version,
                    )
                    .then(
                        Contract::ext(env::current_account_id())
                            .callback_on_chan_open_init(runnable),
                    ),
            )
        }
        ibc_vm_rs::IbcMsg::OnChannelOpenTry { .. } => todo!(),
        ibc_vm_rs::IbcMsg::OnChannelOpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => {
            let account_id = AccountId::try_from(port_id.to_string()).unwrap();
            Some(
                ibc_app::ext(account_id)
                    .on_channel_open_ack(
                        port_id,
                        channel_id,
                        counterparty_channel_id,
                        counterparty_version,
                    )
                    .then(
                        Contract::ext(env::current_account_id())
                            .callback_on_chan_open_ack(runnable),
                    ),
            )
        }
        ibc_vm_rs::IbcMsg::OnChannelOpenConfirm { .. } => todo!(),
        ibc_vm_rs::IbcMsg::OnRecvPacket { .. } => todo!(),
    }
}

// TODO(aeryz): these ext contract api's should be defined under `ibc-vm` by splitted into technologies such as `near/cosmwasm etc`
#[ext_contract(light_client)]
pub trait LightClient {
    fn initialize(client_id: ClientId, client_state: Vec<u8>, consensus_state: Vec<u8>) -> Self;

    fn status(&self) -> Status;

    fn latest_height(&self) -> Height;

    fn verify_membership(
        &self,
        client_id: ClientId,
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    ) -> bool;
}

#[ext_contract(ibc_app)]
pub trait IbcApp {
    fn on_channel_open_init(
        order: channel::order::Order,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    ) -> bool;

    fn on_channel_open_ack(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_channel_id: String,
        counterparty_version: String,
    ) -> bool;
}
