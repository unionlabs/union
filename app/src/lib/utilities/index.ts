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

export async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

export const generateRandomInteger = (min: number, max: number) =>
  Math.floor(Math.random() * (max - min + 1)) + min
