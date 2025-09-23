import { Schema } from "effect"

// Schema for the staking hub status query response
export const StakingHubStatusSchema = Schema.Struct({
  total_assets: Schema.BigInt,
  total_shares: Schema.BigInt,
  total_reward_amount: Schema.BigInt,
  redemption_rate: Schema.BigDecimal,
  purchase_rate: Schema.BigDecimal,
})

export type StakingHubStatus = Schema.Schema.Type<typeof StakingHubStatusSchema>

// Schemas for unstake requests and batch data
export const UnstakeRequestSchema = Schema.Struct({
  batch_id: Schema.String,
  staker: Schema.String,
  amount: Schema.String,
})

export type UnstakeRequest = Schema.Schema.Type<typeof UnstakeRequestSchema>

export const BatchStatusPendingSchema = Schema.Struct({
  status: Schema.Literal("pending"),
  total_lst_to_burn: Schema.String,
  unstake_requests_count: Schema.String,
})

export const BatchStatusSubmittedSchema = Schema.Struct({
  status: Schema.Literal("submitted"),
  total_lst_to_burn: Schema.String,
  unstake_requests_count: Schema.String,
  receive_time: Schema.String,
  expected_native_unstaked: Schema.String,
})

export const BatchStatusReceivedSchema = Schema.Struct({
  status: Schema.Literal("received"),
  total_lst_to_burn: Schema.String,
  unstake_requests_count: Schema.String,
  received_native_unstaked: Schema.String,
})

export const BatchSchema = Schema.Union(
  BatchStatusPendingSchema,
  BatchStatusSubmittedSchema,
  BatchStatusReceivedSchema,
)

export type BatchStatus = Schema.Schema.Type<typeof BatchSchema>

export const BatchResponseSchema = Schema.Struct({
  batches: Schema.Array(Schema.Struct({
    batch_id: Schema.String,
    batch: BatchSchema,
  })),
})

export type BatchResponse = Schema.Schema.Type<typeof BatchResponseSchema>
