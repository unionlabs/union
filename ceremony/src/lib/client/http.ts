import { browser } from "$app/environment"

export const port = 4919
export const host = `http://localhost:${port}`

type Fetch = (input: RequestInfo | URL, init?: RequestInit | undefined) => Promise<Response>
export type Params = Record<string, string | number | boolean | undefined>

export const get = async <T>(
  resource: string,
  params: Params,
  credentials = false,
  _fetch: Fetch = fetch
): Promise<T | undefined> => {
  try {
    const url = new URL(`${host}/${resource}`)
    Object.entries(params).forEach(
      ([key, value]) => value !== undefined && url.searchParams.set(key, `${value}`)
    )
    const res = await _fetch(url, browser && credentials ? { credentials: "include" } : {})
    if (!res.ok) throw res.status
    const data = await res.json()
    return data as T
  } catch (error) {
    return undefined
  }
}

export const post = async <T>(
  resource: string,
  params: Params,
  body: Record<string, unknown>,
  _fetch: Fetch = fetch
): Promise<T | undefined> => {
  try {
    const url = new URL(`${host}/${resource}`)
    Object.entries(params).forEach(
      ([key, value]) => value !== undefined && url.searchParams.set(key, `${value}`)
    )
    const res = await _fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify(body)
    })
    if (!res.ok) throw res.status
    const data = await res.json()
    return data as T
  } catch (error) {
    return undefined
  }
}
