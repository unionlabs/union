import type {
  Account,
  Block,
  Coin,
  Delegation,
  DistributionParams,
  GovParams,
  IBCChannel,
  IBCConnection,
  MintParams,
  NodeInfo,
  PaginationResponse,
  Proposal,
  SlashingParams,
  StakingParams,
  StakingPool,
  Supply,
  TxResponse,
  UnbondingDelegation,
  Validator,
} from "$lib/types/cosmos"
import { Context, Effect, Layer } from "effect"

export class CosmosClientError extends Error {
  readonly _tag = "CosmosClientError"
  constructor(
    message: string,
    public readonly status?: number,
  ) {
    super(message)
  }
}

export interface CosmosClientConfig {
  restEndpoint: string
  restEndpoints?: string[] // Multiple REST endpoints for racing
  rpcEndpoint?: string
  rpcEndpoints?: string[] // Multiple RPC endpoints for racing
  chainName?: string // Chain name for proxy header
}

export class CosmosClient extends Context.Tag("CosmosClient")<
  CosmosClient,
  {
    readonly config: CosmosClientConfig

    // Base / Tendermint
    readonly getLatestBlock: () => Effect.Effect<Block, CosmosClientError>
    readonly getBlockByHeight: (height: string) => Effect.Effect<Block, CosmosClientError>
    readonly getBlockRange: (minHeight: number, maxHeight: number) => Effect.Effect<
      Array<{ height: string; time: string; hash: string; proposer: string; txCount: number }>,
      CosmosClientError
    >
    readonly getNodeInfo: () => Effect.Effect<NodeInfo, CosmosClientError>
    readonly getValidatorSet: (height?: string) => Effect.Effect<
      {
        block_height: string
        validators: Array<
          {
            address: string
            pub_key: { "@type": string; "key": string }
            voting_power: string
            proposer_priority: string
          }
        >
      },
      CosmosClientError
    >

    // Transactions
    readonly getTx: (hash: string) => Effect.Effect<{ tx_response: TxResponse }, CosmosClientError>
    readonly getTxsByHeight: (
      height: string,
    ) => Effect.Effect<
      { txs: unknown[]; tx_responses: TxResponse[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly searchTxs: (
      query: string,
      page?: number,
      limit?: number,
    ) => Effect.Effect<
      { txs: unknown[]; tx_responses: TxResponse[]; pagination: PaginationResponse },
      CosmosClientError
    >

    // Bank
    readonly getBalances: (
      address: string,
    ) => Effect.Effect<{ balances: Coin[]; pagination: PaginationResponse }, CosmosClientError>
    readonly getTotalSupply: () => Effect.Effect<
      { supply: Coin[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly getSupplyByDenom: (denom: string) => Effect.Effect<Supply, CosmosClientError>

    // Staking
    readonly getValidators: (
      status?: string,
    ) => Effect.Effect<
      { validators: Validator[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly getValidator: (
      address: string,
    ) => Effect.Effect<{ validator: Validator }, CosmosClientError>
    readonly getDelegations: (
      delegatorAddress: string,
    ) => Effect.Effect<
      { delegation_responses: Delegation[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly getValidatorDelegations: (
      validatorAddress: string,
    ) => Effect.Effect<
      { delegation_responses: Delegation[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly getUnbondingDelegations: (
      delegatorAddress: string,
    ) => Effect.Effect<
      { unbonding_responses: UnbondingDelegation[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly getStakingParams: () => Effect.Effect<{ params: StakingParams }, CosmosClientError>
    readonly getStakingPool: () => Effect.Effect<{ pool: StakingPool }, CosmosClientError>
    readonly getSlashingParams: () => Effect.Effect<{ params: SlashingParams }, CosmosClientError>
    readonly getDistributionParams: () => Effect.Effect<
      { params: DistributionParams },
      CosmosClientError
    >
    readonly getGovParams: () => Effect.Effect<{ params: GovParams }, CosmosClientError>
    readonly getMintParams: () => Effect.Effect<{ params: MintParams }, CosmosClientError>

    // Governance
    readonly getProposals: (
      status?: string,
    ) => Effect.Effect<{ proposals: Proposal[]; pagination: PaginationResponse }, CosmosClientError>
    readonly getProposal: (id: string) => Effect.Effect<{ proposal: Proposal }, CosmosClientError>
    readonly getProposalVotes: (
      id: string,
    ) => Effect.Effect<
      {
        votes: Array<
          { proposal_id: string; voter: string; options: Array<{ option: string; weight: string }> }
        >
        pagination: PaginationResponse
      },
      CosmosClientError
    >
    readonly getProposalTally: (
      id: string,
    ) => Effect.Effect<
      {
        tally: {
          yes_count: string
          abstain_count: string
          no_count: string
          no_with_veto_count: string
        }
      },
      CosmosClientError
    >

    // Auth
    readonly getAccount: (address: string) => Effect.Effect<{ account: Account }, CosmosClientError>

    // Distribution
    readonly getDelegationRewards: (
      delegatorAddress: string,
      validatorAddress: string,
    ) => Effect.Effect<{ rewards: Coin[] }, CosmosClientError>
    readonly getDelegatorTotalRewards: (
      delegatorAddress: string,
    ) => Effect.Effect<
      { rewards: Array<{ validator_address: string; reward: Coin[] }>; total: Coin[] },
      CosmosClientError
    >

    // IBC
    readonly getIBCChannels: () => Effect.Effect<
      { channels: IBCChannel[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly getIBCConnections: () => Effect.Effect<
      { connections: IBCConnection[]; pagination: PaginationResponse },
      CosmosClientError
    >
    readonly getIBCClientStates: () => Effect.Effect<
      {
        client_states: Array<{ client_id: string; client_state: unknown }>
        pagination: PaginationResponse
      },
      CosmosClientError
    >
  }
>() {}

const FETCH_TIMEOUT = 15_000 // 15 second timeout per request

const fetchJson = <T>(
  url: string,
  headers?: Record<string, string>,
): Effect.Effect<T, CosmosClientError> =>
  Effect.tryPromise({
    try: async () => {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT)

      try {
        const response = await fetch(url, {
          headers,
          signal: controller.signal,
        })
        if (!response.ok) {
          throw new CosmosClientError(
            `HTTP ${response.status}: ${response.statusText}`,
            response.status,
          )
        }
        return response.json() as Promise<T>
      } finally {
        clearTimeout(timeoutId)
      }
    },
    catch: (error) => {
      if (error instanceof CosmosClientError) {
        return error
      }
      if (error instanceof Error && error.name === "AbortError") {
        return new CosmosClientError(`Request timeout after ${FETCH_TIMEOUT}ms`)
      }
      return new CosmosClientError(error instanceof Error ? error.message : String(error))
    },
  })

// Race all endpoints - first successful response wins
const raceEndpoints = <T>(
  urls: string[],
  headers?: Record<string, string>,
): Effect.Effect<T, CosmosClientError> => {
  if (urls.length === 0) {
    return Effect.fail(new CosmosClientError("No endpoints configured"))
  }
  if (urls.length === 1) {
    return fetchJson<T>(urls[0], headers)
  }

  // Race all endpoints using Promise.any - first success wins
  return Effect.tryPromise({
    try: async () => {
      const controllers = urls.map(() => new AbortController())

      const promises = urls.map(async (url, index) => {
        const controller = controllers[index]
        const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT)

        try {
          const response = await fetch(url, {
            headers,
            signal: controller.signal,
          })

          clearTimeout(timeoutId)

          if (!response.ok) {
            throw new CosmosClientError(
              `HTTP ${response.status}: ${response.statusText}`,
              response.status,
            )
          }

          const data = await response.json() as T
          // Abort other requests
          controllers.forEach((c, i) => i !== index && c.abort())
          return data
        } catch (error) {
          clearTimeout(timeoutId)
          throw error
        }
      })

      return Promise.any(promises)
    },
    catch: (error) => {
      if (error instanceof AggregateError) {
        // All failed - return last error
        const lastError = error.errors[error.errors.length - 1]
        return lastError instanceof CosmosClientError
          ? lastError
          : new CosmosClientError(
            lastError instanceof Error ? lastError.message : String(lastError),
          )
      }
      return error instanceof CosmosClientError
        ? error
        : new CosmosClientError(error instanceof Error ? error.message : String(error))
    },
  })
}

// RPC response types
interface RpcBlockMeta {
  block_id: { hash: string }
  header: {
    height: string
    time: string
    proposer_address: string
  }
  num_txs: string
}

interface RpcBlockchainResponse {
  result: {
    last_height: string
    block_metas: RpcBlockMeta[]
  }
}

// RPC tx_search response
interface RpcTxResult {
  hash: string
  height: string
  index: number
  tx_result: {
    code: number
    data: string
    log: string
    gas_wanted: string
    gas_used: string
    events: Array<
      { type: string; attributes: Array<{ key: string; value: string; index?: boolean }> }
    >
  }
  tx: string // base64 encoded
}

interface RpcTxSearchResponse {
  result: {
    txs: RpcTxResult[]
    total_count: string
  }
}

// Extract message type and addresses from RPC events
function parseRpcTxEvents(events: RpcTxResult["tx_result"]["events"]): {
  msgType: string
  sender?: string
  receiver?: string
} {
  let msgType = "Unknown"
  let sender: string | undefined
  let receiver: string | undefined

  for (const event of events) {
    if (event.type === "message") {
      for (const attr of event.attributes) {
        if (attr.key === "action" && attr.value) {
          // action contains the message type like "/cosmos.bank.v1beta1.MsgSend"
          msgType = attr.value
        }
        if (attr.key === "sender" && attr.value) {
          sender = attr.value
        }
      }
    }
    if (event.type === "transfer") {
      for (const attr of event.attributes) {
        if (attr.key === "sender" && !sender) {
          sender = attr.value
        }
        if (attr.key === "recipient" && !receiver) {
          receiver = attr.value
        }
      }
    }
    if (event.type === "delegate" || event.type === "unbond" || event.type === "redelegate") {
      for (const attr of event.attributes) {
        if (attr.key === "validator" && !receiver) {
          receiver = attr.value
        }
      }
    }
  }

  return { msgType, sender, receiver }
}

export const makeCosmosClient = (config: CosmosClientConfig) => {
  const { restEndpoint, restEndpoints, rpcEndpoint, rpcEndpoints, chainName } = config
  const base = restEndpoint.replace(/\/$/, "")
  const rpc = rpcEndpoint?.replace(/\/$/, "")

  // Build list of REST bases for racing (primary + extras)
  const restBases = [base]
  if (restEndpoints) {
    restBases.push(...restEndpoints.map((e) => e.replace(/\/$/, "")))
  }

  // Build list of RPC bases for racing
  const rpcBases: string[] = []
  if (rpc) {
    rpcBases.push(rpc)
  }
  if (rpcEndpoints) {
    rpcBases.push(...rpcEndpoints.map((e) => e.replace(/\/$/, "")))
  }

  // Headers to pass (chain name for proxy)
  const proxyHeaders = chainName ? { "x-chain": chainName } : undefined

  // Helper: race all REST endpoints
  const restRace = <T>(path: string) =>
    raceEndpoints<T>(restBases.map((b) => `${b}${path}`), proxyHeaders)

  // Helper: race all RPC endpoints (also uses proxy headers in browser)
  const rpcRace = <T>(path: string) =>
    rpcBases.length > 0
      ? raceEndpoints<T>(rpcBases.map((b) => `${b}${path}`), proxyHeaders)
      : Effect.fail(new CosmosClientError("No RPC endpoints configured"))

  return {
    config,

    // Base / Tendermint - race all REST endpoints
    getLatestBlock: () => restRace<Block>("/cosmos/base/tendermint/v1beta1/blocks/latest"),
    getBlockByHeight: (height: string) =>
      restRace<Block>(`/cosmos/base/tendermint/v1beta1/blocks/${height}`),

    // Bulk fetch blocks via RPC (races RPC endpoints) with REST fallback
    getBlockRange: (minHeight: number, maxHeight: number) =>
      Effect.gen(function*() {
        // Try RPC first (faster for bulk) - race all RPC endpoints
        if (rpcBases.length > 0) {
          const rpcResult = yield* Effect.either(
            rpcRace<RpcBlockchainResponse>(
              `/blockchain?minHeight=${minHeight}&maxHeight=${maxHeight}`,
            ),
          )
          if (rpcResult._tag === "Right") {
            return rpcResult.right.result.block_metas.map((meta) => ({
              height: meta.header.height,
              time: meta.header.time,
              hash: meta.block_id.hash,
              proposer: meta.header.proposer_address,
              txCount: parseInt(meta.num_txs) || 0,
            }))
          }
          // RPC failed, fall through to REST
        }

        // Fallback: fetch via REST (slower but reliable) - race endpoints per block
        const heights = Array.from(
          { length: maxHeight - minHeight + 1 },
          (_, i) => maxHeight - i,
        )
        const blocks = yield* Effect.all(
          heights.map((h) => restRace<Block>(`/cosmos/base/tendermint/v1beta1/blocks/${h}`)),
          { concurrency: 5 },
        )
        return blocks.map((b) => ({
          height: b.block.header.height,
          time: b.block.header.time,
          hash: b.block_id.hash,
          proposer: b.block.header.proposer_address,
          txCount: b.block.data.txs?.length ?? 0,
        }))
      }),

    getNodeInfo: () => restRace<NodeInfo>("/cosmos/base/tendermint/v1beta1/node_info"),
    getValidatorSet: (height?: string) =>
      restRace<
        {
          block_height: string
          validators: Array<
            {
              address: string
              pub_key: { "@type": string; "key": string }
              voting_power: string
              proposer_priority: string
            }
          >
        }
      >(`/cosmos/base/tendermint/v1beta1/validatorsets/${height ?? "latest"}`),

    // Transactions - race all REST endpoints
    getTx: (hash: string) =>
      restRace<{ tx_response: TxResponse }>(`/cosmos/tx/v1beta1/txs/${hash}`),

    // Get txs by height - try RPC first
    getTxsByHeight: (height: string) =>
      Effect.gen(function*() {
        if (rpcBases.length > 0) {
          const rpcResult = yield* Effect.either(
            rpcRace<RpcTxSearchResponse>(
              `/tx_search?query="tx.height=${height}"&per_page=100&page=1`,
            ),
          )
          if (rpcResult._tag === "Right") {
            const { txs, total_count } = rpcResult.right.result
            const tx_responses: TxResponse[] = txs.map((tx) => {
              const parsed = parseRpcTxEvents(tx.tx_result.events)
              const message: Record<string, unknown> = { "@type": parsed.msgType }
              if (parsed.sender) {
                message.sender = parsed.sender
              }
              if (parsed.receiver) {
                message.receiver = parsed.receiver
              }

              return {
                height: tx.height,
                txhash: tx.hash,
                codespace: "",
                code: tx.tx_result.code,
                data: tx.tx_result.data,
                raw_log: tx.tx_result.log,
                logs: [],
                info: "",
                gas_wanted: tx.tx_result.gas_wanted,
                gas_used: tx.tx_result.gas_used,
                tx: {
                  "@type": "/cosmos.tx.v1beta1.Tx",
                  "body": { messages: [message] },
                  "auth_info": { fee: {} },
                  "signatures": [],
                },
                timestamp: "",
                events: tx.tx_result.events,
              }
            })
            return { txs: [], tx_responses, pagination: { total: total_count, next_key: null } }
          }
        }
        return yield* restRace<
          { txs: unknown[]; tx_responses: TxResponse[]; pagination: PaginationResponse }
        >(
          `/cosmos/tx/v1beta1/txs?query=tx.height=${height}&pagination.limit=100`,
        )
      }),

    // Search txs - try RPC first (works on most nodes), fall back to REST
    searchTxs: (query: string, page = 1, limit = 20) =>
      Effect.gen(function*() {
        // Try RPC first - usually works even when REST is rate-limited
        if (rpcBases.length > 0) {
          const rpcResult = yield* Effect.either(
            rpcRace<RpcTxSearchResponse>(
              `/tx_search?query="${
                encodeURIComponent(query)
              }"&per_page=${limit}&page=${page}&order_by="desc"`,
            ),
          )
          if (rpcResult._tag === "Right") {
            const { txs, total_count } = rpcResult.right.result
            // Convert RPC format to REST format
            const tx_responses: TxResponse[] = txs.map((tx) => {
              const parsed = parseRpcTxEvents(tx.tx_result.events)
              // Build a minimal message from events
              const message: Record<string, unknown> = { "@type": parsed.msgType }
              if (parsed.sender) {
                message.sender = parsed.sender
              }
              if (parsed.receiver) {
                message.receiver = parsed.receiver
              }

              return {
                height: tx.height,
                txhash: tx.hash,
                codespace: "",
                code: tx.tx_result.code,
                data: tx.tx_result.data,
                raw_log: tx.tx_result.log,
                logs: [],
                info: "",
                gas_wanted: tx.tx_result.gas_wanted,
                gas_used: tx.tx_result.gas_used,
                tx: {
                  "@type": "/cosmos.tx.v1beta1.Tx",
                  "body": { messages: [message] },
                  "auth_info": { fee: {} },
                  "signatures": [],
                },
                timestamp: "",
                events: tx.tx_result.events,
              }
            })
            return {
              txs: [],
              tx_responses,
              pagination: { total: total_count, next_key: null },
            }
          }
          // RPC failed, fall through to REST
        }

        // Fallback to REST
        return yield* restRace<
          { txs: unknown[]; tx_responses: TxResponse[]; pagination: PaginationResponse }
        >(
          `/cosmos/tx/v1beta1/txs?order_by=2&query=${
            encodeURIComponent(query)
          }&pagination.limit=${limit}&pagination.offset=${(page - 1) * limit}`,
        )
      }),

    // Legacy txs endpoint for address-based queries (may work better on some chains)
    searchTxsLegacy: (sender: string, page = 1, limit = 20) =>
      restRace<{ txs: unknown[]; tx_responses: TxResponse[]; pagination: PaginationResponse }>(
        `/txs?message.sender=${sender}&page=${page}&limit=${limit}`,
      ),

    // Bank - race all REST endpoints
    getBalances: (address: string) =>
      restRace<{ balances: Coin[]; pagination: PaginationResponse }>(
        `/cosmos/bank/v1beta1/balances/${address}`,
      ),
    getTotalSupply: () =>
      restRace<{ supply: Coin[]; pagination: PaginationResponse }>("/cosmos/bank/v1beta1/supply"),
    getSupplyByDenom: (denom: string) =>
      restRace<Supply>(`/cosmos/bank/v1beta1/supply/by_denom?denom=${encodeURIComponent(denom)}`),

    // Staking - race all REST endpoints
    getValidators: (status?: string) => {
      const path = status
        ? `/cosmos/staking/v1beta1/validators?status=${status}&pagination.limit=500`
        : `/cosmos/staking/v1beta1/validators?pagination.limit=500`
      return restRace<{ validators: Validator[]; pagination: PaginationResponse }>(path)
    },
    getValidator: (address: string) =>
      restRace<{ validator: Validator }>(`/cosmos/staking/v1beta1/validators/${address}`),
    getDelegations: (delegatorAddress: string) =>
      restRace<{ delegation_responses: Delegation[]; pagination: PaginationResponse }>(
        `/cosmos/staking/v1beta1/delegations/${delegatorAddress}`,
      ),
    getValidatorDelegations: (validatorAddress: string) =>
      restRace<{ delegation_responses: Delegation[]; pagination: PaginationResponse }>(
        `/cosmos/staking/v1beta1/validators/${validatorAddress}/delegations?pagination.limit=100`,
      ),
    getUnbondingDelegations: (delegatorAddress: string) =>
      restRace<{ unbonding_responses: UnbondingDelegation[]; pagination: PaginationResponse }>(
        `/cosmos/staking/v1beta1/delegators/${delegatorAddress}/unbonding_delegations`,
      ),
    getStakingParams: () => restRace<{ params: StakingParams }>("/cosmos/staking/v1beta1/params"),
    getStakingPool: () => restRace<{ pool: StakingPool }>("/cosmos/staking/v1beta1/pool"),
    getSlashingParams: () =>
      restRace<{ params: SlashingParams }>("/cosmos/slashing/v1beta1/params"),
    getDistributionParams: () =>
      restRace<{ params: DistributionParams }>("/cosmos/distribution/v1beta1/params"),
    getGovParams: () => restRace<{ params: GovParams }>("/cosmos/gov/v1/params"),
    getMintParams: () => restRace<{ params: MintParams }>("/cosmos/mint/v1beta1/params"),

    // Governance - race all REST endpoints
    getProposals: (status?: string) => {
      const path = status
        ? `/cosmos/gov/v1/proposals?proposal_status=${status}&pagination.limit=100&pagination.reverse=true`
        : `/cosmos/gov/v1/proposals?pagination.limit=100&pagination.reverse=true`
      return restRace<{ proposals: Proposal[]; pagination: PaginationResponse }>(path)
    },
    getProposal: (id: string) => restRace<{ proposal: Proposal }>(`/cosmos/gov/v1/proposals/${id}`),
    getProposalVotes: (id: string) =>
      restRace<
        {
          votes: Array<
            {
              proposal_id: string
              voter: string
              options: Array<{ option: string; weight: string }>
            }
          >
          pagination: PaginationResponse
        }
      >(`/cosmos/gov/v1/proposals/${id}/votes?pagination.limit=100`),
    getProposalTally: (id: string) =>
      restRace<
        {
          tally: {
            yes_count: string
            abstain_count: string
            no_count: string
            no_with_veto_count: string
          }
        }
      >(`/cosmos/gov/v1/proposals/${id}/tally`),

    // Auth - race all REST endpoints
    getAccount: (address: string) =>
      restRace<{ account: Account }>(`/cosmos/auth/v1beta1/accounts/${address}`),

    // Distribution - race all REST endpoints
    getDelegationRewards: (delegatorAddress: string, validatorAddress: string) =>
      restRace<{ rewards: Coin[] }>(
        `/cosmos/distribution/v1beta1/delegators/${delegatorAddress}/rewards/${validatorAddress}`,
      ),
    getDelegatorTotalRewards: (delegatorAddress: string) =>
      restRace<{ rewards: Array<{ validator_address: string; reward: Coin[] }>; total: Coin[] }>(
        `/cosmos/distribution/v1beta1/delegators/${delegatorAddress}/rewards`,
      ),

    // IBC - race all REST endpoints
    getIBCChannels: () =>
      restRace<{ channels: IBCChannel[]; pagination: PaginationResponse }>(
        "/ibc/core/channel/v1/channels?pagination.limit=100",
      ),
    getIBCConnections: () =>
      restRace<{ connections: IBCConnection[]; pagination: PaginationResponse }>(
        "/ibc/core/connection/v1/connections?pagination.limit=100",
      ),
    getIBCClientStates: () =>
      restRace<
        {
          client_states: Array<{ client_id: string; client_state: unknown }>
          pagination: PaginationResponse
        }
      >("/ibc/core/client/v1/client_states?pagination.limit=100"),
  }
}

export const CosmosClientLive = (config: CosmosClientConfig) =>
  Layer.succeed(CosmosClient, makeCosmosClient(config))
