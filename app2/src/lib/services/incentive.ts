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

import { Data, Effect, Schema, pipe } from "effect"
import { HttpClient, HttpClientResponse } from "@effect/platform"

const REST_BASE_URL = import.meta.env.DEV ? "/api/union" : "https://rest.union.build"

export class IncentiveError extends Data.TaggedError("IncentiveError")<{
  message: string
  cause?: unknown
}> {}

const InflationResponse = Schema.Struct({
  inflation: Schema.String,
})

const StakingPoolResponse = Schema.Struct({
  pool: Schema.Struct({
    not_bonded_tokens: Schema.String,
    bonded_tokens: Schema.String,
  }),
})

const DistributionParamsResponse = Schema.Struct({
  params: Schema.Struct({
    community_tax: Schema.String,
    base_proposer_reward: Schema.String,
    bonus_proposer_reward: Schema.String,
    withdraw_addr_enabled: Schema.Boolean,
  }),
})

const CirculatingSupplyResponse = Schema.Struct({
  amount: Schema.Struct({
    denom: Schema.String,
    amount: Schema.String,
  }),
})

// Schema for the incentive calculation result
export const IncentiveResult = Schema.Struct({
  rates: Schema.Struct({
    yearly: Schema.Number,
  }),
  incentiveNominal: Schema.Number,
  incentiveAfterTax: Schema.Number,
  inflation: Schema.Number,
  totalSupply: Schema.Number,
  bondedTokens: Schema.Number,
  communityTax: Schema.Number,
  bondedRatio: Schema.Number,
})

export type IncentiveResult = Schema.Schema.Type<typeof IncentiveResult>

const getInflation = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/mint/v1beta1/inflation`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(InflationResponse)),
  Effect.mapError((cause) => new IncentiveError({
    message: "Failed to fetch inflation rate",
    cause,
  })),
)

const getStakingPool = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/staking/v1beta1/pool`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(StakingPoolResponse)),
  Effect.mapError((cause) => new IncentiveError({
    message: "Failed to fetch staking pool",
    cause,
  })),
)

const getDistributionParams = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/distribution/v1beta1/params`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(DistributionParamsResponse)),
  Effect.mapError((cause) => new IncentiveError({
    message: "Failed to fetch distribution params",
    cause,
  })),
)

const getCirculatingSupply = pipe(
  HttpClient.get(`${REST_BASE_URL}/cosmos/bank/v1beta1/supply/by_denom?denom=au`),
  Effect.flatMap(HttpClientResponse.schemaBodyJson(CirculatingSupplyResponse)),
  Effect.mapError((cause) => new IncentiveError({
    message: "Failed to fetch circulating supply",
    cause,
  })),
)

export const calculateIncentive: Effect.Effect<IncentiveResult, IncentiveError, HttpClient.HttpClient> = Effect.gen(function*() {
  const [inflationData, stakingPoolData, distributionData, circulatingSupplyData] = yield* Effect.all([
    getInflation,
    getStakingPool,
    getDistributionParams,
    getCirculatingSupply,
  ], { concurrency: "unbounded" })

  const inflation = parseFloat(inflationData.inflation)
  const bondedTokensRaw = parseFloat(stakingPoolData.pool.bonded_tokens)
  const communityTax = parseFloat(distributionData.params.community_tax)
  const circulatingSupplyRaw = parseFloat(circulatingSupplyData.amount.amount)

  const bondedTokens = bondedTokensRaw / 1_000_000_000_000_000_000
  const totalSupply = circulatingSupplyRaw / 1_000_000_000_000_000_000

  if (isNaN(inflation) || isNaN(bondedTokens) || isNaN(totalSupply) || isNaN(communityTax)) {
    return yield* Effect.fail(new IncentiveError({
      message: "Invalid numeric values in API responses",
    }))
  }

  if (totalSupply === 0) {
    return yield* Effect.fail(new IncentiveError({
      message: "Invalid total supply",
    }))
  }

  if (bondedTokens === 0) {
    return yield* Effect.fail(new IncentiveError({
      message: "No bonded tokens found",
    }))
  }

  // Step 1: Calculate nominal incentive rate
  const incentiveNominal = (inflation * totalSupply) / bondedTokens

  // Step 2: Apply community tax
  const incentiveAfterTax = incentiveNominal * (1 - communityTax)

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
    bondedRatio: bondedTokens / totalSupply,
  }
})

// Helper to format incentive as percentage
export const formatIncentive = (incentive: number): string => {
  return `${(incentive * 100).toFixed(2)}%`
}
