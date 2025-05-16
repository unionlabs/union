import { Duration, Schedule } from "effect"

export const retryForever = Schedule.exponential(Duration.millis(500))
