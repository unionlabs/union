import { Schedule } from "effect"

export const cosmosBalanceRetrySchedule = Schedule.exponential("2 seconds", 2.0).pipe(
  Schedule.intersect(Schedule.recurs(8))
)
