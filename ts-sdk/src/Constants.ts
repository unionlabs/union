/**
 * This module interfaces with the indexer via the GraphQL protocol.
 *
 * @since 2.0.0
 */
import { UniversalChainId } from "./schema/chain.js"
import { TokenRawDenom } from "./schema/token.js"

/**
 * @category models
 * @since 2.0.0
 */
interface GasDenomMetadata {
  address: TokenRawDenom
  name: string
  symbol: string
  /**
   * Symbol used to fetch USD pricing.
   */
  tickerSymbol: string
  decimals: number
}

/**
 * @category constants
 * @since 2.0.0
 */
const ETH_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
  name: "Ether",
  symbol: "ETH",
  tickerSymbol: "ETH",
  decimals: 18,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
const SEI_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
  name: "Sei",
  symbol: "SEI",
  tickerSymbol: "SEI",
  decimals: 18,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
const CORN_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
  name: "Bitcorn",
  symbol: "BTCN",
  tickerSymbol: "BTC",
  decimals: 18,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
export const BABYLON_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0x7562626e"),
  name: "Baby",
  symbol: "BABY",
  tickerSymbol: "BABY",
  decimals: 6,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
export const OSMOSIS_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0x756f736d6f"),
  name: "Osmo",
  symbol: "OSMO",
  tickerSymbol: "OSMO",
  decimals: 6,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
const UNION_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0x6d756e6f"),
  name: "Union",
  symbol: "UNO",
  tickerSymbol: "UNO",
  decimals: 6,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
const XION_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0x7578696f6e"),
  name: "Xion",
  symbol: "XION",
  tickerSymbol: "XION",
  decimals: 6,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
const BNB_METADATA: GasDenomMetadata = {
  address: TokenRawDenom.make("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
  name: "BNB",
  symbol: "BNB",
  tickerSymbol: "BNB",
  decimals: 18,
} as const

/**
 * @category constants
 * @since 2.0.0
 */
export const GAS_DENOMS: Record<UniversalChainId, GasDenomMetadata> = {
  // Ethereum chains
  [UniversalChainId.make("ethereum.11155111")]: ETH_METADATA,
  [UniversalChainId.make("ethereum.1")]: ETH_METADATA,
  [UniversalChainId.make("ethereum.17000")]: ETH_METADATA,

  // Bob
  [UniversalChainId.make("bob.60808")]: ETH_METADATA,
  [UniversalChainId.make("bob.808813")]: ETH_METADATA,

  // Sei chains
  [UniversalChainId.make("sei.pacific-1")]: SEI_METADATA,
  [UniversalChainId.make("sei.atlantic-2")]: SEI_METADATA,
  [UniversalChainId.make("sei.1328")]: SEI_METADATA,
  [UniversalChainId.make("sei.1329")]: SEI_METADATA,

  // Babylon chains
  [UniversalChainId.make("babylon.bbn-test-5")]: BABYLON_METADATA,
  [UniversalChainId.make("babylon.bbn-1")]: BABYLON_METADATA,

  // Osmosis chains
  [UniversalChainId.make("osmosis.osmo-test-5")]: OSMOSIS_METADATA,
  [UniversalChainId.make("osmosis.osmosis-1")]: OSMOSIS_METADATA,

  // Union chains
  [UniversalChainId.make("union.union-testnet-8")]: UNION_METADATA,
  [UniversalChainId.make("union.union-testnet-9")]: UNION_METADATA,
  [UniversalChainId.make("union.union-testnet-10")]: UNION_METADATA,
  [UniversalChainId.make("union.union-1")]: UNION_METADATA,

  // Xion chains
  [UniversalChainId.make("xion.xion-testnet-2")]: XION_METADATA,
  [UniversalChainId.make("xion.xion-mainnet-1")]: XION_METADATA,

  // Corn
  [UniversalChainId.make("corn.21000000")]: CORN_METADATA,
  [UniversalChainId.make("corn.21000001")]: CORN_METADATA,

  // BSC
  [UniversalChainId.make("bsc.97")]: BNB_METADATA,
}
