export function getTimeoutInNanoseconds24HoursFromNow(): bigint {
  const millisecondsNow = Date.now() // current time in ms
  const millisecondsIn24Hours = 24 * 60 * 60 * 1000 // 24 hours in ms
  const totalMilliseconds = millisecondsNow + millisecondsIn24Hours
  return BigInt(totalMilliseconds) * BigInt(1_000_000) // convert ms to ns
}
