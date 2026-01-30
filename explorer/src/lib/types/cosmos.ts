// Core Cosmos types based on REST API responses

export interface Coin {
  denom: string
  amount: string
}

export interface Block {
  block_id: {
    hash: string
    part_set_header: {
      total: number
      hash: string
    }
  }
  block: {
    header: {
      version: { block: string; app: string }
      chain_id: string
      height: string
      time: string
      last_block_id: {
        hash: string
        part_set_header: { total: number; hash: string }
      }
      last_commit_hash: string
      data_hash: string
      validators_hash: string
      next_validators_hash: string
      consensus_hash: string
      app_hash: string
      last_results_hash: string
      evidence_hash: string
      proposer_address: string
    }
    data: {
      txs: string[]
    }
    evidence: {
      evidence: unknown[]
    }
    last_commit: {
      height: string
      round: number
      block_id: {
        hash: string
        part_set_header: { total: number; hash: string }
      }
      signatures: Array<{
        block_id_flag: string
        validator_address: string
        timestamp: string
        signature: string | null
      }>
    }
  }
}

export interface TxResponse {
  height: string
  txhash: string
  codespace: string
  code: number
  data: string
  raw_log: string
  logs: Array<{
    msg_index: number
    log: string
    events: Array<{
      type: string
      attributes: Array<{ key: string; value: string }>
    }>
  }>
  info: string
  gas_wanted: string
  gas_used: string
  tx: {
    "@type": string
    body: {
      messages: Array<{ "@type": string; [key: string]: unknown }>
      memo: string
      timeout_height: string
      extension_options: unknown[]
      non_critical_extension_options: unknown[]
    }
    auth_info: {
      signer_infos: Array<{
        public_key: { "@type": string; key: string }
        mode_info: unknown
        sequence: string
      }>
      fee: {
        amount: Coin[]
        gas_limit: string
        payer: string
        granter: string
      }
    }
    signatures: string[]
  }
  timestamp: string
  events: Array<{
    type: string
    attributes: Array<{ key: string; value: string; index?: boolean }>
  }>
}

export interface Validator {
  operator_address: string
  consensus_pubkey: {
    "@type": string
    key: string
  }
  jailed: boolean
  status: string
  tokens: string
  delegator_shares: string
  description: {
    moniker: string
    identity: string
    website: string
    security_contact: string
    details: string
  }
  unbonding_height: string
  unbonding_time: string
  commission: {
    commission_rates: {
      rate: string
      max_rate: string
      max_change_rate: string
    }
    update_time: string
  }
  min_self_delegation: string
}

export interface Delegation {
  delegation: {
    delegator_address: string
    validator_address: string
    shares: string
  }
  balance: Coin
}

export interface UnbondingDelegation {
  delegator_address: string
  validator_address: string
  entries: Array<{
    creation_height: string
    completion_time: string
    initial_balance: string
    balance: string
  }>
}

export interface Proposal {
  id: string
  messages: Array<{ "@type": string; [key: string]: unknown }>
  status: string
  final_tally_result: {
    yes_count: string
    abstain_count: string
    no_count: string
    no_with_veto_count: string
  }
  submit_time: string
  deposit_end_time: string
  total_deposit: Coin[]
  voting_start_time: string
  voting_end_time: string
  metadata: string
  title: string
  summary: string
  proposer: string
}

export interface Account {
  "@type": string
  address: string
  pub_key: { "@type": string; key: string } | null
  account_number: string
  sequence: string
}

export interface NodeInfo {
  default_node_info: {
    protocol_version: {
      p2p: string
      block: string
      app: string
    }
    default_node_id: string
    listen_addr: string
    network: string
    version: string
    channels: string
    moniker: string
    other: {
      tx_index: string
      rpc_address: string
    }
  }
  application_version: {
    name: string
    app_name: string
    version: string
    git_commit: string
    build_tags: string
    go_version: string
    build_deps: Array<{ path: string; version: string; sum: string }>
    cosmos_sdk_version: string
  }
}

export interface PaginationResponse {
  next_key: string | null
  total: string
}

export interface StakingParams {
  unbonding_time: string
  max_validators: number
  max_entries: number
  historical_entries: number
  bond_denom: string
  min_commission_rate: string
}

export interface StakingPool {
  not_bonded_tokens: string
  bonded_tokens: string
}

export interface Supply {
  amount: Coin
}

export interface IBCChannel {
  state: string
  ordering: string
  counterparty: {
    port_id: string
    channel_id: string
  }
  connection_hops: string[]
  version: string
  port_id: string
  channel_id: string
}

export interface IBCConnection {
  id: string
  client_id: string
  versions: Array<{ identifier: string; features: string[] }>
  state: string
  counterparty: {
    client_id: string
    connection_id: string
    prefix: { key_prefix: string }
  }
  delay_period: string
}

export interface SlashingParams {
  signed_blocks_window: string
  min_signed_per_window: string
  downtime_jail_duration: string
  slash_fraction_double_sign: string
  slash_fraction_downtime: string
}

export interface DistributionParams {
  community_tax: string
  base_proposer_reward: string
  bonus_proposer_reward: string
  withdraw_addr_enabled: boolean
}

export interface GovParams {
  min_deposit: Coin[]
  max_deposit_period: string
  voting_period: string
  quorum: string
  threshold: string
  veto_threshold: string
  min_initial_deposit_ratio: string
  proposal_cancel_ratio: string
  proposal_cancel_dest: string
  expedited_voting_period: string
  expedited_threshold: string
  expedited_min_deposit: Coin[]
  burn_vote_quorum: boolean
  burn_proposal_deposit_prevote: boolean
  burn_vote_veto: boolean
  min_deposit_ratio: string
}

export interface MintParams {
  mint_denom: string
  inflation_rate_change: string
  inflation_max: string
  inflation_min: string
  goal_bonded: string
  blocks_per_year: string
}
