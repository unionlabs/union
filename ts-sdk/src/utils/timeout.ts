/**
 * Determine timeout timestamp for a fungible asset order.
 *
 * @remarks Actually three days, not 1 day
 *
,* @category utils
 * @since 2.0.0
 */
export function getTimeoutInNanoseconds24HoursFromNow(): bigint {
  const millisecondsNow = Date.now() // current time in ms
  const millisecondsIn24Hours = 24 * 60 * 60 * 1000 * 3 // 24 hours in ms * 3
  const totalMilliseconds = millisecondsNow + millisecondsIn24Hours
  return BigInt(totalMilliseconds) * BigInt(1_000_000) // convert ms to ns
}
