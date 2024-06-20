import type { MaybePromise } from "../../types.ts"

export async function noThrow<T>(x: MaybePromise<T>): Promise<T | undefined> {
  try {
    return await x
  } catch {
    return undefined
  }
}

export function noThrowSync<T>(callback: T): T | undefined {
  try {
    return callback
  } catch {
    return undefined
  }
}
