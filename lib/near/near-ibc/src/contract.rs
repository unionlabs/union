use ibc_vm_rs::{
    states::{
        channel_handshake::{ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry},
        client_state::UpdateClient,
        connection_handshake::{
            ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
        },
        packet::{Acknowledgement, RecvPacket, SendPacket},
        CreateClient,
    },
    CallbackError, IbcHost, IbcQuery, IbcResponse, IbcState, IbcVmResponse, Runnable, Status,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
    env, ext_contract, near_bindgen, AccountId, BorshStorageKey, PromiseOrValue,
};
#[allow(unused)]
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;
use unionlabs::{
    encoding::{Decode, Encode, Proto},
    ibc::core::{
        channel::{self, packet::Packet},
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection,
    },
    ics24::Path,
    id::{ChannelId, ClientId, ConnectionId, PortId},
    validated::ValidateT,
};

use crate::error::Error;

#[allow(unused)]
#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    History,
}

impl IbcHost for Contract {
    type Error = Error;

    fn next_client_identifier(&mut self, client_type: &str) -> Result<ClientId, Error> {
        self.client_index += 1;
        Ok(format!("{client_type}-{}", self.client_index)
            .validate()
            .unwrap())
    }

    fn commit_raw(&mut self, key: Path, value: Vec<u8>) -> Result<(), Error> {
        self.commitments.insert(&key.to_string(), &value);
        Ok(())
    }

    fn next_connection_identifier(&mut self) -> Result<ConnectionId, Error> {
        self.connection_index += 1;
        Ok(format!("connection-{}", self.connection_index)
            .validate()
            .unwrap())
    }

    fn client_state(&self, client_id: &str) -> Option<Vec<u8>> {
        self.commitments
            .get(&format!("clients/{client_id}/clientState"))
    }

    fn read<T: Decode<Proto>>(&self, key: &Path) -> Option<T> {
        self.commitments
            .get(&key.to_string())
            .map(|item| T::decode(&item).unwrap())
    }

    fn commit<T: Encode<Proto>>(&mut self, key: Path, value: T) -> Result<(), Error> {
        self.commitments.insert(&key.to_string(), &value.encode());
        Ok(())
    }

    fn next_channel_identifier(&mut self) -> Result<ChannelId, Error> {
        self.channel_index += 1;
        Ok(format!("channel-{}", self.channel_index)
            .validate()
            .unwrap())
    }

    fn read_raw(&self, key: &Path) -> Option<Vec<u8>> {
        self.commitments.get(&key.to_string())
    }

    fn current_height(&self) -> Height {
        Height::new(env::block_height())
    }

    fn current_timestamp(&self) -> u64 {
        // TODO(aeryz): should this be in ms?
        env::block_timestamp()
    }

    fn sha256(&self, data: Vec<u8>) -> Vec<u8> {
        env::sha256(&data)
    }

    fn delete(&mut self, key: &Path) -> Result<(), Self::Error> {
        let _ = self.commitments.remove(&key.to_string());
        Ok(())
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Owner)]
pub struct Contract {
    commitments: LookupMap<String, Vec<u8>>,
    client_index: u64,
    connection_index: u64,
    channel_index: u64,
    account_ids: LookupMap<String, AccountId>,

    // client id -> account id
    clients: LookupMap<String, AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        Contract {
            commitments: LookupMap::new(b"commitments".as_slice()),
            client_index: 0,
            channel_index: 0,
            account_ids: LookupMap::new(b"account_ids".as_slice()),
            clients: LookupMap::new(b"clients".as_slice()),
            connection_index: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn register_client(&mut self, client_type: String, account: String) {
        let account_id: AccountId = account.try_into().unwrap();
        assert!(
            self.account_ids.insert(&client_type, &account_id).is_none(),
            "already registered"
        );
    }

    pub fn create_client(
        &mut self,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            CreateClient::Init {
                client_type,
                client_state,
                consensus_state,
            }
            .into(),
        )
    }

    pub fn connection_open_init(
        &mut self,
        client_id: ClientId,
        counterparty: connection::counterparty::Counterparty,
        version: connection::version::Version,
        delay_period: u64,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ConnectionOpenInit::Init {
                client_id,
                counterparty,
                version,
                delay_period,
            }
            .into(),
        )
    }

    pub fn connection_open_try(
        &mut self,
        client_id: ClientId,
        counterparty: connection::counterparty::Counterparty,
        counterparty_versions: Vec<connection::version::Version>,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
        delay_period: u64,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ConnectionOpenTry::Init {
                client_id,
                counterparty,
                counterparty_versions,
                connection_end_proof,
                proof_height,
                delay_period,
            }
            .into(),
        )
    }

    pub fn connection_open_ack(
        &mut self,
        connection_id: String,
        version: connection::version::Version,
        counterparty_connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ConnectionOpenAck::Init {
                connection_id,
                version,
                counterparty_connection_id,
                connection_end_proof,
                proof_height,
            }
            .into(),
        )
    }

    pub fn connection_open_confirm(
        &mut self,
        connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ConnectionOpenConfirm::Init {
                connection_id,
                connection_end_proof,
                proof_height,
            }
            .into(),
        )
    }

    pub fn channel_open_init(
        &mut self,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ChannelOpenInit::Init {
                connection_hops,
                port_id,
                counterparty,
                version,
            }
            .into(),
        )
    }

    pub fn channel_open_try(
        &mut self,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
        version: String,
        proof_init: Vec<u8>,
        proof_height: Height,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ChannelOpenTry::Init {
                connection_hops,
                port_id,
                counterparty,
                counterparty_version,
                version,
                proof_init,
                proof_height,
            }
            .into(),
        )
    }

    pub fn channel_open_ack(
        &mut self,
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: Vec<u8>,
        proof_height: Height,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ChannelOpenAck::Init {
                channel_id,
                port_id,
                counterparty_channel_id,
                counterparty_version,
                proof_try,
                proof_height,
            }
            .into(),
        )
    }

    pub fn channel_open_confirm(
        &mut self,
        channel_id: ChannelId,
        port_id: PortId,
        proof_ack: Vec<u8>,
        proof_height: Height,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            ChannelOpenConfirm::Init {
                channel_id,
                port_id,
                proof_ack,
                proof_height,
            }
            .into(),
        )
    }

    pub fn update_client(
        &mut self,
        client_id: ClientId,
        client_msg: Vec<u8>,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            UpdateClient::Init {
                client_id,
                client_msg,
            }
            .into(),
        )
    }

    pub fn send_packet(
        &mut self,
        source_port: PortId,
        source_channel: ChannelId,
        timeout_height: Height,
        timeout_timestamp: u64,
        // TODO(aeryz): enforce this to be non-empty at type level
        data: Vec<u8>,
    ) -> PromiseOrValue<IbcVmResponse> {
        assert!(
            &*source_port == env::predecessor_account_id().as_str(),
            "expected sender to own the capability"
        );

        self.init(
            SendPacket::Init {
                source_port,
                source_channel,
                timeout_height,
                timeout_timestamp,
                data,
            }
            .into(),
        )
    }

    pub fn recv_packet(
        &mut self,
        packet: Packet,
        proof_commitment: Vec<u8>,
        proof_height: Height,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            RecvPacket::Init {
                packet,
                proof_commitment,
                proof_height,
            }
            .into(),
        )
    }

    pub fn acknowledgement(
        &mut self,
        packet: Packet,
        ack: Vec<u8>,
        proof_ack: Vec<u8>,
        proof_height: Height,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.init(
            Acknowledgement::Init {
                packet,
                ack,
                proof_ack,
                proof_height,
            }
            .into(),
        )
    }

    #[private]
    pub fn callback_query(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] responses: Vec<IbcResponse>,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &responses)
    }

    #[private]
    pub fn callback_initialize(
        &mut self,
        current_state: IbcState,
    ) -> PromiseOrValue<IbcVmResponse> {
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
                    .insert(&client_id.clone().to_string(), &account_id);
            }
            _ => panic!("wut?"),
        };
        self.step(current_state, &[IbcResponse::Initialize])
    }

    #[private]
    pub fn callback_update_client_on_misbehaviour(
        &mut self,
        current_state: IbcState,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &[IbcResponse::UpdateStateOnMisbehaviour])
    }

    #[private]
    pub fn callback_update_client(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] client_updated: (Vec<u8>, Vec<(Height, Vec<u8>)>),
    ) -> PromiseOrValue<IbcVmResponse> {
        let (client_state, consensus_states) = client_updated;
        self.step(
            current_state,
            &[IbcResponse::UpdateState {
                consensus_states,
                client_state,
            }],
        )
    }

    #[private]
    pub fn callback_on_chan_open_init(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] err: CallbackError,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &[IbcResponse::OnChannelOpenInit { err }])
    }

    #[private]
    pub fn callback_on_chan_open_try(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] err: CallbackError,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &[IbcResponse::OnChannelOpenTry { err }])
    }

    #[private]
    pub fn callback_on_chan_open_ack(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] err: CallbackError,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &[IbcResponse::OnChannelOpenAck { err }])
    }

    #[private]
    pub fn callback_on_chan_open_confirm(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] err: CallbackError,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &[IbcResponse::OnChannelOpenConfirm { err }])
    }

    #[private]
    pub fn callback_on_recv_packet(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] ack: Vec<u8>,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &[IbcResponse::OnRecvPacket { ack }])
    }

    #[private]
    pub fn callback_on_acknowledge_packet(
        &mut self,
        current_state: IbcState,
        #[callback_unwrap] err: CallbackError,
    ) -> PromiseOrValue<IbcVmResponse> {
        self.step(current_state, &[IbcResponse::OnAcknowledgePacket { err }])
    }

    fn init(&mut self, runnable: IbcState) -> PromiseOrValue<IbcVmResponse> {
        self.step(runnable, &[IbcResponse::Empty])
    }

    #[allow(clippy::too_many_lines)]
    fn step(
        &mut self,
        runnable: IbcState,
        response: &[IbcResponse],
    ) -> PromiseOrValue<IbcVmResponse> {
        let either = runnable.process(self, response).unwrap();

        let (runnable, ibc_action) = match either {
            ibc_vm_rs::Either::Left(cont) => cont,
            ibc_vm_rs::Either::Right((events, val)) => {
                for event in events {
                    env::log_str(&serde_json::to_string(&event).unwrap());
                }
                return PromiseOrValue::Value(val);
            }
        };

        match ibc_action {
            ibc_vm_rs::IbcAction::Query((client_id, ibc_queries)) => {
                let account_id = self.clients.get(&client_id.to_string()).unwrap();
                PromiseOrValue::Promise(
                    ext_light_client::ext(account_id.clone())
                        .query(ibc_queries)
                        .then(Contract::ext(env::current_account_id()).callback_query(runnable)),
                )
            }
            ibc_vm_rs::IbcAction::Write(ibc_msg) => match ibc_msg {
                ibc_vm_rs::IbcMsg::Initialize {
                    client_id,
                    client_type,
                    client_state,
                    consensus_state,
                } => {
                    let account_id = self.account_ids.get(&client_type).unwrap();
                    PromiseOrValue::Promise(
                        ext_light_client::ext(account_id.clone())
                            .initialize(client_id, client_state, consensus_state)
                            .then(
                                Contract::ext(env::current_account_id())
                                    .callback_initialize(runnable),
                            ),
                    )
                }
                ibc_vm_rs::IbcMsg::UpdateStateOnMisbehaviour {
                    client_id,
                    client_msg,
                } => {
                    let account_id = self.clients.get(&client_id.to_string()).unwrap();
                    PromiseOrValue::Promise(
                        ext_light_client::ext(account_id.clone())
                            .update_client_on_misbehaviour(client_msg)
                            .then(
                                Contract::ext(env::current_account_id())
                                    .callback_update_client_on_misbehaviour(runnable),
                            ),
                    )
                }
                ibc_vm_rs::IbcMsg::UpdateState {
                    client_id,
                    client_msg,
                } => {
                    let account_id = self.clients.get(&client_id.to_string()).unwrap();
                    PromiseOrValue::Promise(
                        ext_light_client::ext(account_id.clone())
                            .update_client(client_msg)
                            .then(
                                Contract::ext(env::current_account_id())
                                    .callback_update_client(runnable),
                            ),
                    )
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
                    PromiseOrValue::Promise(
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
                ibc_vm_rs::IbcMsg::OnChannelOpenTry {
                    order,
                    connection_hops,
                    port_id,
                    channel_id,
                    counterparty,
                    counterparty_version,
                } => {
                    let account_id = AccountId::try_from(port_id.to_string()).unwrap();
                    PromiseOrValue::Promise(
                        ibc_app::ext(account_id)
                            .on_channel_open_try(
                                order,
                                connection_hops,
                                port_id,
                                channel_id,
                                counterparty,
                                counterparty_version,
                            )
                            .then(
                                Contract::ext(env::current_account_id())
                                    .callback_on_chan_open_try(runnable),
                            ),
                    )
                }
                ibc_vm_rs::IbcMsg::OnChannelOpenAck {
                    port_id,
                    channel_id,
                    counterparty_channel_id,
                    counterparty_version,
                } => {
                    let account_id = AccountId::try_from(port_id.to_string()).unwrap();
                    PromiseOrValue::Promise(
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
                ibc_vm_rs::IbcMsg::OnChannelOpenConfirm {
                    port_id,
                    channel_id,
                } => {
                    let account_id = AccountId::try_from(port_id.to_string()).unwrap();
                    PromiseOrValue::Promise(
                        ibc_app::ext(account_id)
                            .on_channel_open_confirm(port_id, channel_id)
                            .then(
                                Contract::ext(env::current_account_id())
                                    .callback_on_chan_open_confirm(runnable),
                            ),
                    )
                }
                ibc_vm_rs::IbcMsg::OnRecvPacket { packet } => {
                    let account_id =
                        AccountId::try_from(packet.destination_port.clone().to_string()).unwrap();
                    PromiseOrValue::Promise(ibc_app::ext(account_id).recv_packet(packet).then(
                        Contract::ext(env::current_account_id()).callback_on_recv_packet(runnable),
                    ))
                }
                ibc_vm_rs::IbcMsg::OnAcknowledgePacket { packet, ack } => {
                    let account_id =
                        AccountId::try_from(packet.source_port.clone().to_string()).unwrap();
                    PromiseOrValue::Promise(
                        ibc_app::ext(account_id)
                            .on_acknowledge_packet(packet, ack)
                            .then(
                                Contract::ext(env::current_account_id())
                                    .callback_on_acknowledge_packet(runnable),
                            ),
                    )
                }
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum LightClientQuery {
    Status,
    LatestHeight,
    VerifyClientMessage(Vec<u8>),
    CheckForMisbehaviour(Vec<u8>),
    VerifyMembership {
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    },
}

#[ext_contract(ext_light_client)]
pub trait LightClient {
    fn initialize(client_id: ClientId, client_state: Vec<u8>, consensus_state: Vec<u8>);

    fn query(&self, query: Vec<IbcQuery>) -> Vec<IbcResponse>;

    fn status(&self) -> Status;

    fn latest_height(&self) -> Height;

    fn verify_membership(
        &self,
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    ) -> bool;

    fn verify_client_message(&self, client_msg: Vec<u8>) -> bool;

    fn check_for_misbehaviour(&self, client_msg: Vec<u8>) -> bool;

    fn update_client(&mut self, client_msg: Vec<u8>) -> (Vec<u8>, Vec<(Height, Vec<u8>)>);

    fn update_client_on_misbehaviour(&mut self, client_msg: Vec<u8>);
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

    fn on_channel_open_try(
        order: channel::order::Order,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
    ) -> bool;

    fn on_channel_open_ack(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_channel_id: String,
        counterparty_version: String,
    ) -> bool;

    fn on_channel_open_confirm(port_id: PortId, channel_id: ChannelId) -> bool;

    fn on_acknowledge_packet(packet: Packet, ack: Vec<u8>) -> bool;

    fn recv_packet(packet: Packet) -> Vec<u8>;
}
