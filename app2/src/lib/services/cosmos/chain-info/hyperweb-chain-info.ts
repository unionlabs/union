import { Array as A, Option as O, pipe, Schema as S, Struct } from "effect"

const Binaries = S.Struct({
  "linux/amd64": S.OptionFromSelf(S.String),
  "linux/arm64": S.OptionFromSelf(S.String),
  "darwin/amd64": S.OptionFromSelf(S.String),
  "darwin/arm64": S.OptionFromSelf(S.String),
  "windows/amd64": S.OptionFromSelf(S.String),
  "windows/arm64": S.OptionFromSelf(S.String),
})
type Binaries = typeof Binaries.Type

const Version = S.String
type Version = typeof Version.Type

const Repo = S.String
type Repo = typeof Repo.Type

const Tag = S.String
type Tag = typeof Tag.Type

const Ibc = S.Struct({
  type: S.Union(S.Literal("go"), S.Literal("rust"), S.Literal("other")),
  version: S.OptionFromSelf(Version),
  repo: S.OptionFromSelf(Repo),
  tag: S.OptionFromSelf(Tag),
  ics_enabled: S.OptionFromSelf(
    S.NonEmptyArray(S.Union(S.Literal("ics20-1"), S.Literal("ics27-1"), S.Literal("mauth"))),
  ),
})
type Ibc = typeof Ibc.Type

const Cosmwasm = S.Struct({
  version: S.OptionFromSelf(Version),
  repo: S.OptionFromSelf(Repo),
  tag: S.OptionFromSelf(Tag),
  enabled: S.OptionFromSelf(S.Boolean),
  path: S.OptionFromSelf(S.String),
})
type Cosmwasm = typeof Cosmwasm.Type

const Consensus = S.Struct({
  type: S.Union(S.Literal("tendermint"), S.Literal("cometbft"), S.Literal("sei-tendermint")),
  version: S.OptionFromSelf(Version),
  repo: S.OptionFromSelf(Repo),
  tag: S.OptionFromSelf(Tag),
})
type Consensus = typeof Consensus.Type

const Sdk = S.Struct({
  type: S.Union(S.Literal("cosmos"), S.Literal("penumbra"), S.Literal("other")),
  version: S.OptionFromSelf(Version),
  repo: S.OptionFromSelf(Repo),
  tag: S.OptionFromSelf(Tag),
})
type Sdk = typeof Sdk.Type

const Language = S.Struct({
  type: S.Union(
    S.Literal("go"),
    S.Literal("rust"),
    S.Literal("solidity"),
    S.Literal("other"),
  ),
  version: S.OptionFromSelf(Version),
  repo: S.OptionFromSelf(Repo),
  tag: S.OptionFromSelf(Tag),
})
type Language = typeof Language.Type

const Pointer = S.Struct({
  chain_name: S.String,
  base_denom: S.OptionFromSelf(S.String),
})
type Pointer = typeof Pointer.Type

const Explorer = S.Struct({
  kind: S.OptionFromSelf(S.String),
  url: S.OptionFromSelf(S.String),
  tx_page: S.OptionFromSelf(S.String),
  account_page: S.OptionFromSelf(S.String),
  validator_page: S.OptionFromSelf(S.String),
  proposal_page: S.OptionFromSelf(S.String),
  block_page: S.OptionFromSelf(S.String),
})
type Explorer = typeof Explorer.Type

const Endpoint = S.Struct({
  address: S.String,
  provider: S.OptionFromSelf(S.String),
  archive: S.OptionFromSelf(S.Boolean),
})
type Endpoint = typeof Endpoint.Type

const Peer = S.Struct({
  id: S.String,
  address: S.String,
  provider: S.OptionFromSelf(S.String),
})
type Peer = typeof Peer.Type

const StakingToken = S.Struct({
  denom: S.String,
})
type StakingToken = typeof StakingToken.Type

const FeeToken = S.Struct({
  denom: S.String,
  fixed_min_gas_price: S.OptionFromSelf(S.Number),
  low_gas_price: S.OptionFromSelf(S.Number),
  average_gas_price: S.OptionFromSelf(S.Number),
  high_gas_price: S.OptionFromSelf(S.Number),
  gas_costs: S.OptionFromSelf(S.Struct({
    cosmos_send: S.OptionFromSelf(S.Number),
    ibc_transfer: S.OptionFromSelf(S.Number),
  })),
})
type FeeToken = typeof FeeToken.Type

const ChainType = S.Union(
  S.Literal("cosmos"),
  S.Literal("eip155"),
  S.Literal("bip122"),
  S.Literal("polkadot"),
  S.Literal("solana"),
  S.Literal("algorand"),
  S.Literal("arweave"),
  S.Literal("ergo"),
  S.Literal("fil"),
  S.Literal("hedera"),
  S.Literal("monero"),
  S.Literal("reef"),
  S.Literal("stacks"),
  S.Literal("starknet"),
  S.Literal("stellar"),
  S.Literal("tezos"),
  S.Literal("vechain"),
  S.Literal("waves"),
  S.Literal("xrpl"),
  S.Literal("unknown"),
)
type ChainType = typeof ChainType.Type

/**
 * An explicit superset of `@chain-registry/types` meant to be the source transformation for a given wallet.
 */
export class HyperwebChainInfo extends S.Class<HyperwebChainInfo>("HyperwebChainInfo")({
  $schema: S.OptionFromSelf(S.String),
  chain_name: S.String,
  chain_type: ChainType,
  chain_id: S.String,
  pre_fork_chain_name: S.OptionFromSelf(S.String),
  pretty_name: S.OptionFromSelf(S.String),
  website: S.OptionFromSelf(S.String),
  update_link: S.OptionFromSelf(S.String),
  status: S.OptionFromSelf(S.Union(S.Literal("live"), S.Literal("upcoming"), S.Literal("killed"))),
  network_type: S.OptionFromSelf(
    S.Union(S.Literal("mainnet"), S.Literal("testnet"), S.Literal("devnet")),
  ),
  bech32_prefix: S.OptionFromSelf(S.String),
  bech32_config: S.OptionFromSelf(S.Struct({
    bech32PrefixAccAddr: S.OptionFromSelf(S.String),
    bech32PrefixAccPub: S.OptionFromSelf(S.String),
    bech32PrefixValAddr: S.OptionFromSelf(S.String),
    bech32PrefixValPub: S.OptionFromSelf(S.String),
    bech32PrefixConsAddr: S.OptionFromSelf(S.String),
    bech32PrefixConsPub: S.OptionFromSelf(S.String),
  })),
  daemon_name: S.OptionFromSelf(S.String),
  node_home: S.OptionFromSelf(S.String),
  key_algos: S.OptionFromSelf(S.NonEmptyArray(S.Union(
    S.Literal("secp256k1"),
    S.Literal("ethsecp256k1"),
    S.Literal("ed25519"),
    S.Literal("sr25519"),
    S.Literal("bn254"),
  ))),
  slip44: S.OptionFromSelf(S.String),
  alternative_slip44s: S.OptionFromSelf(S.NonEmptyArray(S.Number)),
  fees: S.OptionFromSelf(S.Struct({
    fee_tokens: S.NonEmptyArray(FeeToken),
  })),
  staking: S.OptionFromSelf(S.Struct({
    staking_tokens: S.Array(StakingToken),
    lock_duration: S.OptionFromSelf(S.Struct({
      blocks: S.OptionFromSelf(S.Number),
      time: S.OptionFromSelf(S.String),
    })),
  })),
  codebase: S.OptionFromSelf(S.Struct({
    git_repo: S.OptionFromSelf(S.String),
    recommended_version: S.OptionFromSelf(S.String),
    compatible_versions: S.OptionFromSelf(S.NonEmptyArray(S.String)),
    language: S.OptionFromSelf(Language),
    binaries: S.OptionFromSelf(Binaries),
    cosmos_sdk_version: S.OptionFromSelf(S.String),
    sdk: S.OptionFromSelf(Sdk),
    consensus: S.OptionFromSelf(Consensus),
    cosmwasm_version: S.OptionFromSelf(S.String),
    cosmwasm_enabled: S.OptionFromSelf(S.Boolean),
    cosmwasm_path: S.OptionFromSelf(S.String),
    cosmwasm: S.OptionFromSelf(Cosmwasm),
    ibc: S.OptionFromSelf(Ibc),
    genesis: S.OptionFromSelf(S.Struct({
      name: S.OptionFromSelf(S.String),
      genesis_url: S.String,
      ics_ccv_url: S.OptionFromSelf(S.String),
    })),
    versions: S.OptionFromSelf(S.NonEmptyArray(S.Struct({
      name: S.String,
      tag: S.OptionFromSelf(S.String),
      height: S.OptionFromSelf(S.Number),
      proposal: S.OptionFromSelf(S.Number),
      previous_version_name: S.OptionFromSelf(S.String),
      next_version_name: S.OptionFromSelf(S.String),
      recommended_version: S.OptionFromSelf(S.String),
      compatible_versions: S.OptionFromSelf(S.NonEmptyArray(S.String)),
      language: S.OptionFromSelf(Language),
      cosmos_sdk_version: S.OptionFromSelf(S.String),
      sdk: S.OptionFromSelf(Sdk),
      consensus: S.OptionFromSelf(Consensus),
      cosmwasm_version: S.OptionFromSelf(S.String),
      cosmwasm_enabled: S.OptionFromSelf(S.Boolean),
      cosmwasm_path: S.OptionFromSelf(S.String),
      cosmwasm: S.OptionFromSelf(Cosmwasm),
      ibc: S.OptionFromSelf(Ibc),
      binaries: S.OptionFromSelf(Binaries),
    }))),
  })),
  images: S.OptionFromSelf(S.NonEmptyArray(S.Struct({
    image_sync: S.OptionFromSelf(Pointer),
    png: S.OptionFromSelf(S.String),
    svg: S.OptionFromSelf(S.String),
    theme: S.OptionFromSelf(S.Struct({
      primary_color_hex: S.OptionFromSelf(S.String),
      background_color_hex: S.OptionFromSelf(S.String),
      circle: S.OptionFromSelf(S.Boolean),
      dark_mode: S.OptionFromSelf(S.Boolean),
      monochrome: S.OptionFromSelf(S.Boolean),
    })),
  }))),
  logo_URIs: S.OptionFromSelf(S.Struct({
    png: S.OptionFromSelf(S.String),
    svg: S.OptionFromSelf(S.String),
  })),
  description: S.OptionFromSelf(S.String),
  peers: S.OptionFromSelf(S.Struct({
    seeds: S.OptionFromSelf(S.NonEmptyArray(Peer)),
    persistent_peers: S.OptionFromSelf(S.NonEmptyArray(Peer)),
  })),
  apis: S.OptionFromSelf(S.Struct({
    "rpc": S.OptionFromSelf(S.NonEmptyArray(Endpoint)),
    "rest": S.OptionFromSelf(S.NonEmptyArray(Endpoint)),
    "grpc": S.OptionFromSelf(S.NonEmptyArray(Endpoint)),
    "wss": S.OptionFromSelf(S.NonEmptyArray(Endpoint)),
    "grpc-web": S.OptionFromSelf(S.NonEmptyArray(Endpoint)),
    "evm-http-jsonrpc": S.OptionFromSelf(S.NonEmptyArray(Endpoint)),
  })),
  explorers: S.OptionFromSelf(S.NonEmptyArray(Explorer)),
  keywords: S.OptionFromSelf(S.NonEmptyArray(S.String)),
  extra_codecs: S.OptionFromSelf(
    S.NonEmptyArray(S.Union(S.Literal("ethermint"), S.Literal("injective"))),
  ),
}) {
  get primaryRest(): O.Option<string> {
    return pipe(
      this.apis,
      O.flatMap(Struct.get("rest")),
      O.map(A.headNonEmpty),
      O.map(Struct.get("address")),
    )
  }

  get primaryRpc(): O.Option<string> {
    return pipe(
      this.apis,
      O.flatMap(Struct.get("rpc")),
      O.map(A.headNonEmpty),
      O.map(Struct.get("address")),
    )
  }
}
