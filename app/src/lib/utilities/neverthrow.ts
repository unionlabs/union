import { ResultAsync } from "neverthrow"

export const fetchJson = (url: string) =>
  ResultAsync.fromPromise(
    fetch(url).then(response => {
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
