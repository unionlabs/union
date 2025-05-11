import { Schedule, Duration } from "effect";

export const retryForever = Schedule.exponential(Duration.millis(500));