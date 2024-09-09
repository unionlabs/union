/**
 * sleep for the specified number of milliseconds
 * @example
 * ```ts
 * await sleep(1000)
 * ```
 */
export const sleep = async (ms: number): Promise<void> =>
  new Promise(resolve => setTimeout(resolve, ms))

/**
 * get the current timestamp in the format `YYYY-MM-DD--HH-MM-SS`
 * @example
 * ```ts
 * timestamp()
 * ```
 */
export function timestamp(): string {
  const d = new Date()
  const [date] = d.toISOString().split("T")
  const [time] = d.toTimeString().split(" ")
  return `${date}--${time?.replace(/:/g, "-")}`
}

/**
 * raise a runtime error
 * @example
 * ```ts
 * raise("something went wrong")
 * raise(new Error("something went wrong"))
 * ```
 */
export function raise(error: unknown): never {
  throw typeof error === "string" ? new Error(error) : error
}
