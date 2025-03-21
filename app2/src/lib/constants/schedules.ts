import { Schedule } from "effect"
import type { FetchEvmBalanceError } from "$lib/services/evm/balances"

export const cosmosBalanceRetrySchedule = Schedule.exponential("2 seconds", 2.0).pipe(
  Schedule.intersect(Schedule.recurs(8))
)

export const aptosBalanceRetrySchedule = Schedule.exponential("2 seconds", 2.0).pipe(
  Schedule.intersect(Schedule.recurs(8))
)

export const evmBalanceRetrySchedule = Schedule.exponential("100 seconds", 2.0).pipe(
  Schedule.intersect(Schedule.recurs(8)),
  Schedule.whileInput(
    (error: FetchEvmBalanceError) =>
      (error._tag === "ReadContractError" || error._tag === "FetchNativeBalanceError") &&
      error.cause?.message?.includes("HTTP request failed")
  )
)
