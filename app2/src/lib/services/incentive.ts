/**
 * Incentive Service
 *
 * Calculates liquid staking incentives including:
 * - Base staking rewards (inflation × total_supply ÷ bonded_tokens)
 * - Community tax deduction
 * - Validator commission deduction (weighted average from delegated validators)
 */

import { HttpClient, HttpClientResponse } from "@effect/platform"
import { EU_STAKING_HUB } from "@unionlabs/sdk/Constants"
import { Array, BigDecimal, Data, Effect, pipe, Schema } from "effect"

const REST_BASE_URL = "https://rest.union.build"

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

const ValidatorsResponse = Schema.Struct({
  validators: Schema.Array(Schema.Struct({
    operator_address: Schema.String,
    tokens: Schema.BigDecimal,
    commission: Schema.Struct({
      commission_rates: Schema.Struct({
        rate: Schema.BigDecimal,
      }),
    }),
    status: Schema.String,
    jailed: Schema.Boolean,
  })),
})

const DelegatorDelegationsResponse = Schema.Struct({
  delegation_responses: Schema.Array(Schema.Struct({
    delegation: Schema.Struct({
      delegator_address: Schema.String,
      validator_address: Schema.String,
      shares: Schema.BigDecimal,
    }),
    balance: Schema.Struct({
      denom: Schema.String,
      amount: Schema.BigDecimal,
    }),
  })),
})

const LstConfigResponse = Schema.Struct({
  data: Schema.Struct({
    staker_address: Schema.String,
    native_token_denom: Schema.String,
    minimum_liquid_stake_amount: Schema.String,
    protocol_fee_config: Schema.Struct({
      fee_rate: Schema.String,
      fee_recipient: Schema.String,
    }),
    monitors: Schema.Array(Schema.String),
    lst_address: Schema.String,
    batch_period_seconds: Schema.Number,
    unbonding_period_seconds: Schema.Number,
    stopped: Schema.Boolean,
  }),
})

export const IncentiveResult = Schema.Struct({
  rates: Schema.Struct({
    yearly: Schema.BigDecimalFromSelf,
  }),
  incentiveNominal: Schema.BigDecimalFromSelf,
  incentiveAfterTax: Schema.BigDecimalFromSelf,
  incentiveAfterCommission: Schema.BigDecimalFromSelf,
  communityTaxAmount: Schema.BigDecimalFromSelf,
  validatorCommissionAmount: Schema.BigDecimalFromSelf,
  weightedAverageCommission: Schema.BigDecimalFromSelf,
  inflation: Schema.BigDecimalFromSelf,
  totalSupply: Schema.BigDecimalFromSelf,
  bondedTokens: Schema.BigDecimalFromSelf,
  communityTax: Schema.BigDecimalFromSelf,
  bondedRatio: Schema.BigDecimalFromSelf,
})

export type IncentiveResult = Schema.Schema.Type<typeof IncentiveResult>

const getInflation = pipe(
  HttpClient.HttpClient,
  Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
  Effect.andThen((client) =>
    pipe(
      client.get(`${REST_BASE_URL}/cosmos/mint/v1beta1/inflation`),
      Effect.flatMap(HttpClientResponse.schemaBodyJson(InflationResponse)),
      Effect.mapError((cause) =>
        new IncentiveError({
          message: "Failed to fetch inflation rate",
          cause,
        })
      ),
    )
  ),
)

const getStakingPool = pipe(
  HttpClient.HttpClient,
  Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
  Effect.andThen((client) =>
    pipe(
      client.get(`${REST_BASE_URL}/cosmos/staking/v1beta1/pool`),
      Effect.flatMap(HttpClientResponse.schemaBodyJson(StakingPoolResponse)),
      Effect.mapError((cause) =>
        new IncentiveError({
          message: "Failed to fetch staking pool",
          cause,
        })
      ),
    )
  ),
)

const getDistributionParams = pipe(
  HttpClient.HttpClient,
  Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
  Effect.andThen((client) =>
    pipe(
      client.get(`${REST_BASE_URL}/cosmos/distribution/v1beta1/params`),
      Effect.flatMap(HttpClientResponse.schemaBodyJson(DistributionParamsResponse)),
      Effect.mapError((cause) =>
        new IncentiveError({
          message: "Failed to fetch distribution params",
          cause,
        })
      ),
    )
  ),
)

const getCirculatingSupply = pipe(
  HttpClient.HttpClient,
  Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
  Effect.andThen((client) =>
    pipe(
      client.get(`${REST_BASE_URL}/cosmos/bank/v1beta1/supply/by_denom?denom=au`),
      Effect.flatMap(HttpClientResponse.schemaBodyJson(CirculatingSupplyResponse)),
      Effect.mapError((cause) =>
        new IncentiveError({
          message: "Failed to fetch circulating supply",
          cause,
        })
      ),
    )
  ),
)

const getValidators = pipe(
  HttpClient.HttpClient,
  Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
  Effect.andThen((client) =>
    pipe(
      client.get(`${REST_BASE_URL}/cosmos/staking/v1beta1/validators?status=BOND_STATUS_BONDED`),
      Effect.flatMap(HttpClientResponse.schemaBodyJson(ValidatorsResponse)),
      Effect.mapError((cause) =>
        new IncentiveError({
          message: "Failed to fetch validators",
          cause,
        })
      ),
    )
  ),
)

const getLstConfig = pipe(
  HttpClient.HttpClient,
  Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
  Effect.andThen((client) => {
    const queryMsg = btoa(JSON.stringify({ config: {} }))
    return pipe(
      client.get(
        `${REST_BASE_URL}/cosmwasm/wasm/v1/contract/${EU_STAKING_HUB.address}/smart/${queryMsg}`,
      ),
      Effect.flatMap(HttpClientResponse.schemaBodyJson(LstConfigResponse)),
      Effect.mapError((cause) =>
        new IncentiveError({
          message: "Failed to fetch LST contract config",
          cause,
        })
      ),
    )
  }),
)

const getDelegatorDelegations = (delegatorAddress: string) =>
  pipe(
    HttpClient.HttpClient,
    Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
    Effect.andThen((client) =>
      pipe(
        client.get(`${REST_BASE_URL}/cosmos/staking/v1beta1/delegations/${delegatorAddress}`),
        Effect.flatMap(HttpClientResponse.schemaBodyJson(DelegatorDelegationsResponse)),
        Effect.mapError((cause) =>
          new IncentiveError({
            message: "Failed to fetch delegator delegations",
            cause,
          })
        ),
      )
    ),
  )

export const calculateIncentive: Effect.Effect<
  IncentiveResult,
  IncentiveError,
  HttpClient.HttpClient
> = Effect.gen(function*() {
  // First get the LST config to find the staker address
  const lstConfig = yield* getLstConfig
  const stakerAddress = lstConfig.data.staker_address

  const [
    inflationData,
    stakingPoolData,
    distributionData,
    circulatingSupplyData,
    validatorsData,
    delegationsData,
  ] = yield* Effect
    .all([
      getInflation,
      getStakingPool,
      getDistributionParams,
      getCirculatingSupply,
      getValidators,
      getDelegatorDelegations(stakerAddress),
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

  const incentiveNominal = yield* pipe(
    BigDecimal.multiply(inflation, totalSupply),
    BigDecimal.divide(bondedTokens),
    Effect.mapError(() =>
      new IncentiveError({
        message: "Could not calculate nominal incentive",
      })
    ),
  )

  const communityTaxAmount = BigDecimal.multiply(incentiveNominal, communityTax)
  const incentiveAfterTax = BigDecimal.subtract(incentiveNominal, communityTaxAmount)

  // Calculate weighted average validator commission
  const validatorMap = new Map(validatorsData.validators.map(v => [v.operator_address, v]))

  const validDelegations = pipe(
    delegationsData.delegation_responses,
    Array.filter(delegation => {
      const validator = validatorMap.get(delegation.delegation.validator_address)
      return Boolean(validator && !validator.jailed && validator.status === "BOND_STATUS_BONDED")
    }),
    Array.map(delegation => ({
      amount: delegation.balance.amount,
      commission:
        validatorMap.get(delegation.delegation.validator_address)!.commission.commission_rates.rate,
    })),
  )

  const { totalAmount, weightedSum } = pipe(
    validDelegations,
    Array.reduce(
      { totalAmount: BigDecimal.fromBigInt(0n), weightedSum: BigDecimal.fromBigInt(0n) },
      (acc, { amount, commission }) => ({
        totalAmount: BigDecimal.sum(acc.totalAmount, amount),
        weightedSum: BigDecimal.sum(acc.weightedSum, BigDecimal.multiply(amount, commission)),
      }),
    ),
  )

  const weightedAverageCommission = BigDecimal.isZero(totalAmount)
    ? BigDecimal.fromBigInt(0n)
    : yield* BigDecimal.divide(weightedSum, totalAmount).pipe(
      Effect.mapError(() =>
        new IncentiveError({
          message: "Could not calculate weighted average commission",
        })
      ),
    )

  const validatorCommissionAmount = BigDecimal.multiply(
    incentiveAfterTax,
    weightedAverageCommission,
  )
  const incentiveAfterCommission = BigDecimal.subtract(incentiveAfterTax, validatorCommissionAmount)

  const bondedRatio = yield* BigDecimal.divide(bondedTokens, totalSupply).pipe(
    Effect.mapError(() =>
      new IncentiveError({
        message: "Could not bonded ratio",
      })
    ),
  )

  return {
    rates: {
      yearly: incentiveAfterCommission,
    },
    incentiveNominal,
    incentiveAfterTax,
    incentiveAfterCommission,
    communityTaxAmount,
    validatorCommissionAmount,
    weightedAverageCommission,
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
