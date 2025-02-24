use anyhow::{anyhow, bail, Context, Result};
use cometbft_rpc::rpc_types::{GrpcAbciQueryError, TxResponse};
use protos::cosmos::base::abci;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tracing::{debug, info};
use unionlabs::{
    cosmos::{
        auth::base_account::BaseAccount,
        base::{abci::gas_info::GasInfo, coin::Coin},
        crypto::{secp256k1, AnyPubKey},
        tx::{
            auth_info::AuthInfo, fee::Fee, mode_info::ModeInfo, sign_doc::SignDoc,
            signer_info::SignerInfo, signing::sign_info::SignMode, tx::Tx, tx_body::TxBody,
            tx_raw::TxRaw,
        },
    },
    encoding::{EncodeAs, Proto},
    google::protobuf::any::Any,
    primitives::H256,
    prost::{Message, Name},
    signer::CosmosSigner,
};

// TODO: Add read and write versions of this
#[derive(Debug, Clone)]
pub struct Ctx {
    signer: CosmosSigner,
    client: cometbft_rpc::Client,
    gas_config: GasConfig,
    chain_id: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GasConfig {
    pub gas_price: f64,
    pub gas_denom: String,
    pub gas_multiplier: f64,
    pub max_gas: u64,
    pub min_gas: u64,
}

impl GasConfig {
    pub fn mk_fee(&self, gas: u64) -> Fee {
        // gas limit = provided gas * multiplier, clamped between min_gas and max_gas
        let gas_limit = u128_saturating_mul_f64(gas.into(), self.gas_multiplier)
            .clamp(self.min_gas.into(), self.max_gas.into());

        let amount = u128_saturating_mul_f64(gas.into(), self.gas_price);

        Fee {
            amount: vec![Coin {
                amount,
                denom: self.gas_denom.clone(),
            }],
            gas_limit: gas_limit.try_into().unwrap_or(u64::MAX),
            payer: String::new(),
            granter: String::new(),
        }
    }
}

fn u128_saturating_mul_f64(u: u128, f: f64) -> u128 {
    (num_rational::BigRational::from_integer(u.into())
        * num_rational::BigRational::from_float(f).expect("finite"))
    .to_integer()
    .try_into()
    .unwrap_or(u128::MAX)
    // .expect("overflow")
}

impl Ctx {
    pub async fn new(rpc_url: String, private_key: H256, gas_config: GasConfig) -> Result<Ctx> {
        let client = cometbft_rpc::Client::new(rpc_url)
            .await
            .context("creating cometbft rpc client")?;

        let prefix = client
            .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
                "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
                None,
                false,
            )
            .await
            .context("querying bech32 prefix")?
            .into_result()?
            .unwrap()
            .bech32_prefix;

        let chain_id = client
            .status()
            .await
            .context("querying node status")?
            .node_info
            .network;

        let ctx = Ctx {
            signer: CosmosSigner::new(
                bip32::secp256k1::ecdsa::SigningKey::from_bytes(&private_key.into())
                    .expect("invalid private key"),
                prefix,
            ),
            client,
            gas_config,
            chain_id,
        };

        Ok(ctx)
    }

    pub fn signer(&self) -> &CosmosSigner {
        &self.signer
    }
    pub fn client(&self) -> &cometbft_rpc::Client {
        &self.client
    }

    pub fn gas_config(&self) -> &GasConfig {
        &self.gas_config
    }

    pub fn chain_id(&self) -> &str {
        &self.chain_id
    }

    pub async fn tx<M: Message + Name, R: Message + Default + Name>(
        &self,
        msg: M,
        memo: impl AsRef<str>,
    ) -> Result<(H256, R)> {
        // dbg!(&msg);

        // panic!();

        let (tx_hash, result) = self
            .broadcast_tx_commit(
                [protos::google::protobuf::Any {
                    type_url: M::type_url(),
                    value: msg.encode_to_vec().into(),
                }],
                memo,
            )
            .await
            .context("broadcast_tx_commit")?;

        let response =
            <abci::v1beta1::TxMsgData as Message>::decode(&*result.tx_result.data.unwrap())
                .unwrap();

        assert_eq!(&*response.msg_responses[0].type_url, R::type_url());

        let response =
            R::decode(&*response.msg_responses[0].value).context("parsing returned address")?;

        Ok((tx_hash, response))
    }

    pub async fn contract_info(
        &self,
        address: String,
    ) -> Result<Option<protos::cosmwasm::wasm::v1::ContractInfo>> {
        let result = self
            .client
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryContractInfoResponse>(
                "/cosmwasm.wasm.v1.Query/ContractInfo",
                &(protos::cosmwasm::wasm::v1::QueryContractInfoRequest { address }),
                None,
                false,
            )
            .await?
            .into_result();

        match result {
            Ok(ok) => Ok(Some(ok.unwrap().contract_info.unwrap())),
            Err(err) => {
                if err.error_code.get() == 6 && err.codespace == "sdk" {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    pub async fn code_info(&self, code_id: u64) -> Result<Option<H256>> {
        let result = self
            .client
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryCodeInfoResponse>(
                "/cosmwasm.wasm.v1.Query/CodeInfo",
                &protos::cosmwasm::wasm::v1::QueryCodeInfoRequest { code_id },
                None,
                false,
            )
            .await?
            .into_result();

        match result {
            Ok(ok) => Ok(Some(ok.unwrap().checksum.try_into().unwrap())),
            Err(err) => {
                // if err.error_code.get() == 6 && err.codespace == "sdk" {
                //     Ok(None)
                // } else {
                Err(err.into())
                // }
            }
        }
    }

    pub async fn instantiate_code_id_of_contract(&self, address: String) -> Result<Option<u64>> {
        let result = self.contract_history(address).await?;

        match result {
            Ok(ok) => {
                let contract_code_history_entry = &ok.unwrap().entries[0];

                if contract_code_history_entry.operation
                    != protos::cosmwasm::wasm::v1::ContractCodeHistoryOperationType::Init as i32
                {
                    bail!(
                        "invalid state {} for first history entry",
                        contract_code_history_entry.operation
                    )
                }

                Ok(Some(contract_code_history_entry.code_id))
            }
            Err(err) => {
                // if err.error_code.get() == 6 && err.codespace == "sdk" {
                //     Ok(None)
                // } else {
                Err(err.into())
                // }
            }
        }
    }

    pub async fn contract_history(
        &self,
        address: String,
    ) -> Result<
        Result<
            Option<protos::cosmwasm::wasm::v1::QueryContractHistoryResponse>,
            GrpcAbciQueryError,
        >,
    > {
        Ok(self
            .client
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryContractHistoryResponse>(
                "/cosmwasm.wasm.v1.Query/ContractHistory",
                &protos::cosmwasm::wasm::v1::QueryContractHistoryRequest {
                    address,
                    ..Default::default()
                },
                None,
                false,
            )
            .await?
            .into_result())
    }

    /// - simulate tx
    /// - submit tx
    /// - wait for inclusion
    /// - return (tx_hash, gas_used)
    pub async fn broadcast_tx_commit(
        &self,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any> + Clone,
        memo: impl AsRef<str>,
    ) -> Result<(H256, TxResponse)> {
        let account = self
            .account_info(&self.signer.to_string())
            .await
            .context("fetching account info")?;

        let (tx_body, mut auth_info, simulation_gas_info) = self
            .simulate_tx(messages, memo)
            .await
            .context("simulate_tx")?;

        info!(
            gas_used = %simulation_gas_info.gas_used,
            gas_wanted = %simulation_gas_info.gas_wanted,
            "tx simulation successful"
        );

        auth_info.fee = self.gas_config.mk_fee(simulation_gas_info.gas_used);

        info!(
            fee = %auth_info.fee.amount[0].amount,
            gas_multiplier = %self.gas_config.gas_multiplier,
            "submitting transaction with gas"
        );

        // re-sign the new auth info with the simulated gas
        let signature = self
            .signer
            .try_sign(
                &SignDoc {
                    body_bytes: tx_body.clone().encode_as::<Proto>(),
                    auth_info_bytes: auth_info.clone().encode_as::<Proto>(),
                    chain_id: self.chain_id.to_string(),
                    account_number: account.account_number,
                }
                .encode_as::<Proto>(),
            )
            .expect("signing failed")
            .to_bytes()
            .to_vec();

        let tx_raw_bytes = TxRaw {
            body_bytes: tx_body.clone().encode_as::<Proto>(),
            auth_info_bytes: auth_info.clone().encode_as::<Proto>(),
            signatures: [signature].to_vec(),
        }
        .encode_as::<Proto>();

        let tx_hash: H256 = sha2::Sha256::new()
            .chain_update(&tx_raw_bytes)
            .finalize()
            .into();

        if let Ok(tx) = self.client.tx(tx_hash, false).await {
            debug!(%tx_hash, "tx already included");
            return Ok((tx_hash, tx));
        }

        let response = self
            .client
            .broadcast_tx_sync(&tx_raw_bytes)
            .await
            .context("broadcast_tx_sync")?;

        assert_eq!(tx_hash, response.hash, "tx hash calculated incorrectly");

        info!(%tx_hash);

        info!(
            check_tx_code = %response.code,
            codespace = %response.codespace,
            check_tx_log = %response.log
        );

        if response.code > 0 {
            bail!(
                "cosmos tx failed: {}, {}: {}",
                response.code,
                response.codespace,
                response.log
            );
        };

        let mut target_height = self
            .client
            .block(None)
            .await
            .context("querying latest block")?
            .block
            .header
            .height;

        let mut i = 0;
        loop {
            let reached_height = 'l: loop {
                let current_height = self
                    .client
                    .block(None)
                    .await
                    .context("querying latest block for tx inclusion")?
                    .block
                    .header
                    .height;

                if current_height >= target_height {
                    break 'l current_height;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            };

            let tx_inclusion = self.client.tx(tx_hash, false).await;

            // debug!(?tx_inclusion);

            match tx_inclusion {
                Ok(tx) => {
                    if tx.tx_result.code == 0 {
                        break Ok((tx_hash, tx));
                    } else {
                        bail!(
                            "cosmos tx failed: {}, {}: {}",
                            response.code,
                            response.codespace,
                            response.log
                        );
                    }
                }
                Err(err) if i > 5 => {
                    return Err(anyhow!(
                        "tx inclusion couldn't be retrieved after {i} attempt(s) (tx hash: {tx_hash})"
                    )
                    .context(err));
                }
                Err(_) => {
                    debug!("unable to retrieve tx inclusion, trying again");
                    target_height = reached_height.add(&1);
                    i += 1;
                    continue;
                }
            }
        }
    }

    pub async fn simulate_tx(
        &self,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any> + Clone,
        memo: impl AsRef<str>,
    ) -> Result<(TxBody, AuthInfo, GasInfo)> {
        use protos::cosmos::tx;

        let account = self
            .account_info(&self.signer.to_string())
            .await
            .context("querying account info")?;

        let tx_body = TxBody {
            // TODO: Use RawAny here
            messages: messages.clone().into_iter().map(Into::into).collect(),
            memo: memo.as_ref().to_owned(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
            unordered: false,
            timeout_timestamp: None,
        };

        let auth_info = AuthInfo {
            signer_infos: [SignerInfo {
                public_key: Some(AnyPubKey::Secp256k1(secp256k1::PubKey {
                    key: self.signer.public_key().into(),
                })),
                mode_info: ModeInfo::Single {
                    mode: SignMode::Direct,
                },
                sequence: account.sequence,
            }]
            .to_vec(),
            fee: self.gas_config.mk_fee(self.gas_config.max_gas).clone(),
        };

        let simulation_signature = self
            .signer
            .try_sign(
                &SignDoc {
                    body_bytes: tx_body.clone().encode_as::<Proto>(),
                    auth_info_bytes: auth_info.clone().encode_as::<Proto>(),
                    chain_id: self.chain_id.to_string(),
                    account_number: account.account_number,
                }
                .encode_as::<Proto>(),
            )
            .expect("signing failed")
            .to_bytes()
            .to_vec();

        let simulate_response = self
            .client
            .grpc_abci_query::<_, tx::v1beta1::SimulateResponse>(
                "/cosmos.tx.v1beta1.Service/Simulate",
                &tx::v1beta1::SimulateRequest {
                    tx_bytes: Tx {
                        body: tx_body.clone(),
                        auth_info: auth_info.clone(),
                        signatures: [simulation_signature.clone()].to_vec(),
                    }
                    .encode_as::<Proto>(),
                    ..Default::default()
                },
                None,
                false,
            )
            .await
            .context("submitting SimulateRequest")?
            .into_result()?;

        let result = simulate_response.unwrap();

        Ok((
            tx_body,
            auth_info,
            result
                .gas_info
                .expect("gas info is present on successful simulation result")
                .into(),
        ))
    }

    pub async fn account_info(&self, account: &str) -> Result<BaseAccount> {
        debug!(%account, "fetching account");

        Ok(self
            .client
            .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::QueryAccountResponse>(
                "/cosmos.auth.v1beta1.Query/Account",
                &protos::cosmos::auth::v1beta1::QueryAccountRequest {
                    address: account.to_string(),
                },
                None,
                false,
            )
            .await
            .context("querying account info")?
            .into_result()?
            .unwrap()
            .account
            .map(<Any<BaseAccount>>::try_from)
            .context("decoding account info")??
            .0)
    }
}
