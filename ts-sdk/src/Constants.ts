/**
 * This module interfaces with the indexer via the GraphQL protocol.
 *
 * @since 2.0.0
 */
import { Match, Schedule } from "effect"
import { UniversalChainId } from "./schema/chain.js"
import { TokenRawDenom } from "./schema/token.js"
import * as Token from "./Token.js"
import * as Ucs05 from "./Ucs05.js"

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
  address: TokenRawDenom.make("0x6175"),
  name: "Union",
  symbol: "U",
  tickerSymbol: "U",
  decimals: 18,
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
  [UniversalChainId.make("bsc.56")]: BNB_METADATA,
  [UniversalChainId.make("bsc.97")]: BNB_METADATA,

  // Base
  [UniversalChainId.make("base.8453")]: ETH_METADATA,
  [UniversalChainId.make("base.84532")]: ETH_METADATA,
}

/**
 * @category schedules
 * @since 2.0.0
 */
export const foreverSchedule = Schedule.addDelay(Schedule.forever, () => "500 millis")

/**
 * @category schedules
 * @since 2.0.0
 */
export const rpcSchedule = Schedule.compose(
  Schedule.fixed("2 seconds"),
  Schedule.recurUpTo("10 seconds"),
)

/**
 * @category utils
 * @since 2.0.0
 */
export const tokenMetaOverride = Match.type<string>().pipe(
  Match.when("uxion", () =>
    ({
      symbol: "XION",
      name: "xion",
      decimals: 0,
    }) as const),
  Match.orElse((address) =>
    ({
      symbol: address,
      name: address,
      decimals: 6,
    }) as const
  ),
)

/**
 * @category constants
 * @since 2.0.0
 */
export const ON_ZKGM_CALL_PROXY = Ucs05.CosmosDisplay.make({
  address: "union1mtxk8tjz85ry2a8a6k58uwrztmwslaxzsurh5l0dlxh7wrnvmxkshqkuwd",
})

/**
 * @category constants
 * @since 2.0.0
 */
export const U_BANK = Token.CosmosBank.make({ address: "au" })

/**
 * @category constants
 * @since 2.0.0
 */
export const U_ERC20 = Token.Erc20.make({ address: "0xba5eD44733953d79717F6269357C77718C8Ba5ed" })

/**
 * @category constants
 * @since 2.0.0
 */
export const EU_ERC20 = Token.Erc20.make({ address: "0xe5Cf13C84c0fEa3236C101Bd7d743d30366E5CF1" })

/**
 * @category constants
 * @since 2.0.0
 */
export const EU_LST = Ucs05.CosmosDisplay.make({
  address: "union1eueueueu9var4yhdruyzkjcsh74xzeug6ckyy60hs0vcqnzql2hq0lxc2f",
})

/**
 * @category constants
 * @since 2.0.0
 */
export const EU_STAKING_HUB = Ucs05.CosmosDisplay.make({
  address: "union1d2r4ecsuap4pujrlf3nz09vz8eha8y0z25knq0lfxz4yzn83v6kq0jxsmk",
})

/**
 * @category constants
 * @since 2.0.0
 */
export const U_ON_ETH_SOLVER_METADATA =
  "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" as const

/**
 * @category constants
 * @since 2.0.0
 */
export const U_TO_UNION_SOLVER_METADATA =
  "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040756e696f6e3175757575757575757539756e3271706b73616d37726c747470786338646337366d63706868736d70333970786a6e7376727463717679763537720000000000000000000000000000000000000000000000000000000000000000" as const

/**
 * @category constants
 * @since 2.0.0
 */
export const EU_FROM_UNION_SOLVER_METADATA =
  "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014e5cf13c84c0fea3236c101bd7d743d30366e5cf10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" as const
