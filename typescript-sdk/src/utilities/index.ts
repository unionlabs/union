export const sleep = async (ms: number): Promise<void> =>
  new Promise(resolve => setTimeout(resolve, ms))

export function timestamp() {
  const d = new Date()
  const [date] = d.toISOString().split("T")
  const [time] = d.toTimeString().split(" ")
  return `${date}--${time?.replace(/:/g, "-")}`
}

export function raise(error: unknown): never {
  throw typeof error === "string" ? new Error(error) : error
}
