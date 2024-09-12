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
    const data: T = await res.json()
    return data ?? undefined
  } catch (error) {
    return undefined
  }
}

export const post = async <T>(resource: string, params: Params, body: object, _fetch = fetch) => {
  try {
    const url = new URL(`${host}/${resource}`)
    Object.entries(params).forEach(
      ([key, value]) => value !== undefined && url.searchParams.set(key, `${value}`)
    )
    const res = await _fetch(url, {
      method: "POST",
      body: JSON.stringify(body)
    })
    if (!res.ok) throw res.status
    const data: T = await res.json()
    return data ?? undefined
  } catch (error) {
    return undefined
  }
}
