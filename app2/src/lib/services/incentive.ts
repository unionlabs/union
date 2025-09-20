/**
 * Incentive Service (WORK IN PROGRESS)
 *
 * Calculates staking incentives using:
 * - Total supply from cosmos bank module
 * - Inflation rate from cosmos mint module
 * - Bonded token supply from staking pool
 * - Community tax from distribution params
 *
 * Formula: Incentive = ((1 + [(inflation × total_supply ÷ bonded) × (1 − tax)] ÷ 365) ^ 365) − 1
 */

import { HttpClient, HttpClientResponse } from "@effect/platform"
import { BigDecimal, Data, Effect, pipe, Schema } from "effect"

const REST_BASE_URL = import.meta.env.DEV ? "/api/union" : "https://rest.union.build"

export class IncentiveError extends Data.TaggedError("IncentiveError")<{
  message: string
  cause?: unknown
}> {}

const InflationResponse = Schema.Struct({
  inflation: Schema.BigDecimal,
})

const StakingPoolResponse = Schema.Struct({
  pool: Schema.Struct({
    not_bonded_tokens: Schema.BigDecimal,
    bonded_tokens: Schema.BigDecimal,
  }),
})

const DistributionParamsResponse = Schema.Struct({
  params: Schema.Struct({
    community_tax: Schema.BigDecimal,
    base_proposer_reward: Schema.String,
    bonus_proposer_reward: Schema.String,
    withdraw_addr_enabled: Schema.Boolean,
  }),
})

const CirculatingSupplyResponse = Schema.Struct({
  amount: Schema.Struct({
    denom: Schema.String,
    amount: Schema.BigDecimal,
  }),
})

// Schema for the incentive calculation result
export const IncentiveResult = Schema.Struct({
  rates: Schema.Struct({
    yearly: Schema.BigDecimalFromSelf,
  }),
  incentiveNominal: Schema.BigDecimalFromSelf,
  incentiveAfterTax: Schema.BigDecimalFromSelf,
  inflation: Schema.BigDecimalFromSelf,
  totalSupply: Schema.BigDecimalFromSelf,
  bondedTokens: Schema.BigDecimalFromSelf,
  communityTax: Schema.BigDecimalFromSelf,
  bondedRatio: Schema.BigDecimalFromSelf,
})

export type IncentiveResult = Schema.Schema.Type<typeof IncentiveResult>

const getInflation = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/mint/v1beta1/inflation`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(InflationResponse)),
  Effect.mapError((cause) =>
    new IncentiveError({
      message: "Failed to fetch inflation rate",
      cause,
    })
  ),
)

const getStakingPool = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/staking/v1beta1/pool`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(StakingPoolResponse)),
  Effect.mapError((cause) =>
    new IncentiveError({
      message: "Failed to fetch staking pool",
      cause,
    })
  ),
)

const getDistributionParams = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/distribution/v1beta1/params`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(DistributionParamsResponse)),
  Effect.mapError((cause) =>
    new IncentiveError({
      message: "Failed to fetch distribution params",
      cause,
    })
  ),
)

const getCirculatingSupply = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/bank/v1beta1/supply/by_denom?denom=au`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(CirculatingSupplyResponse)),
  Effect.mapError((cause) =>
    new IncentiveError({
      message: "Failed to fetch circulating supply",
      cause,
    })
  ),
)

export const calculateIncentive: Effect.Effect<
  IncentiveResult,
  IncentiveError,
  HttpClient.HttpClient
> = Effect.gen(function*() {
  const [inflationData, stakingPoolData, distributionData, circulatingSupplyData] = yield* Effect
    .all([
      getInflation,
      getStakingPool,
      getDistributionParams,
      getCirculatingSupply,
    ], { concurrency: "unbounded" })

  const inflation = inflationData.inflation
  const bondedTokensRaw = stakingPoolData.pool.bonded_tokens
  const communityTax = distributionData.params.community_tax
  const circulatingSupplyRaw = circulatingSupplyData.amount.amount

  const dividend = BigDecimal.fromBigInt(1_000_000_000_000_000_000n)
  const bondedTokens = yield* BigDecimal.divide(bondedTokensRaw, dividend).pipe(
    Effect.mapError(() =>
      new IncentiveError({
        message: "Could not calculate bonded tokens amount",
      })
    ),
  )
  const totalSupply = yield* BigDecimal.divide(circulatingSupplyRaw, dividend).pipe(
    Effect.mapError(() =>
      new IncentiveError({
        message: "Could not calculate total supply amount",
      })
    ),
  )

  if (BigDecimal.isZero(totalSupply)) {
    return yield* Effect.fail(
      new IncentiveError({
        message: "Invalid total supply",
      }),
    )
  }

  if (BigDecimal.isZero(bondedTokens)) {
    return yield* Effect.fail(
      new IncentiveError({
        message: "No bonded tokens found",
      }),
    )
  }

  // Step 1: Calculate nominal incentive rate
  const incentiveNominal = yield* pipe(
    BigDecimal.multiply(inflation, totalSupply),
    BigDecimal.divide(bondedTokens),
    Effect.mapError(() =>
      new IncentiveError({
        message: "Could not calculate nominal incentive",
      })
    ),
  )

  // Step 2: Apply community tax
  const incentiveAfterTax = BigDecimal.multiply(
    incentiveNominal,
    BigDecimal.subtract(BigDecimal.fromBigInt(1n), communityTax),
  )

  const bondedRatio = yield* BigDecimal.divide(bondedTokens, totalSupply).pipe(
    Effect.mapError(() =>
      new IncentiveError({
        message: "Could not bonded ratio",
      })
    ),
  )

  return {
    rates: {
      yearly: incentiveAfterTax,
    },
    incentiveNominal,
    incentiveAfterTax,
    inflation,
    totalSupply,
    bondedTokens,
    communityTax,
    bondedRatio,
  }
})

// Helper to format incentive as percentage
export const formatIncentive = (incentive: number): string => {
  return `${(incentive * 100).toFixed(2)}%`
}
