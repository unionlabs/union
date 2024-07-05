import "temporal-polyfill/global"

/**
 * get the current timestamp
 * - for epoch, use `Temporal.Now.instant().epochMilliseconds`,
 * - for string, use `Temporal.Now.instant().toString()`,
 * - other formats are available: `Temporal.Now.instant().toJSON()`, `Temporal.Now.instant().toZonedDateTime()`, etc.
 * @docs https://tc39.es/proposal-temporal/docs/now.html
 */
export const getCurrentTimestamp = (): string => {
  const now = Temporal.Now.instant()
  return now
    .toZonedDateTimeISO("UTC")
    .toString({
      offset: "never",
      timeZoneName: "never",
      smallestUnit: "second"
    })
    .replace("Z", "+00:00")
}

export const getCurrentISODateTime = () => Temporal.Now.plainDateTimeISO().toString()

export const isoTimestampToEpoch = (timestamp: string) =>
  Temporal.Instant.from(timestamp).epochMilliseconds

export const epochToIsoTimestamp = (epoch: number) =>
  Temporal.Instant.fromEpochMilliseconds(epoch).toString()

export const formatTimestamp = (timestamp: string) =>
  Temporal.Instant.from(timestamp).toZonedDateTimeISO("UTC").toString({
    offset: "never",
    timeZoneName: "never",
    smallestUnit: "second"
  })

// yyyy-mm-dd hh:mm:ss
export function toPrettyDateTimeFormat(timestamp: string) {
  const date = new Date(timestamp)
  return date.toISOString().replaceAll("T", " ").split(".").at(0)
}

// https://stackoverflow.com/a/17415677
export function toIsoString(date: Date) {
  const pad = (num: number) => (num < 10 ? "0" : "") + num

  // @ts-ignore
  return (
    // biome-ignore lint/style/useTemplate: would be illegible
    date.getFullYear() +
    "-" +
    pad(date.getMonth() + 1) +
    "-" +
    pad(date.getDate()) +
    "T" +
    pad(date.getHours()) +
    ":" +
    pad(date.getMinutes()) +
    ":" +
    pad(date.getSeconds())
  )
}
