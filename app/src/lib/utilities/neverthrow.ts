import { ResultAsync } from "neverthrow"

export const fetchJson = (url: string, options?: RequestInit) =>
  ResultAsync.fromPromise(
    fetch(url, options).then(response => {
      if (!response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`)
      }
      return response.json()
    }),
    e =>
      new Error(`Failed to fetch data from ${url} with error: ${(e as Error).message}`, {
        cause: e
      })
  )
