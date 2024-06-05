export const noThrow = async <T>(x: Promise<T>): Promise<T | undefined> => x.catch(() => undefined)

export const noThrowSync = <T>(callback: T): T | undefined => {
  try {
    return callback
  } catch {
    return undefined
  }
}

export const sleep = async (ms: number): Promise<void> =>
  new Promise(resolve => setTimeout(resolve, ms))

export function throttle(func: Function, limit: number) {
  let inThrottle: boolean
  return (...args: Array<any>) => {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => (inThrottle = false), limit)
    }
  }
}

export function raise(error: unknown): never {
  throw typeof error === "string" ? new Error(error) : error
}

export async function fetcher<T>(url: string, options?: RequestInit) {
  const response = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      ...options?.headers
    }
  })
  if (!response.ok) {
    raise(
      `\n ${response.status} - Failed to fetch from ${url}:\n ${
        response.statusText
      }\n ${await response.text()}\n`
    )
  }
  const data = (await response.json()) as T
  return data
}
