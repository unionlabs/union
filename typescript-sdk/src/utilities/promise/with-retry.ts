/**
 * source: https://github.com/wevm/viem/blob/main/src/utils/promise/withRetry.ts
 */

import { sleep } from "../index.ts"
import type { ErrorType } from "../../types.ts"

export type WithRetryParameters = {
  // The delay (in ms) between retries.
  delay?: ((config: { count: number; error: Error }) => number) | number | undefined
  // The max number of times to retry.
  retryCount?: number | undefined
  // Whether or not to retry when an error is thrown.
  shouldRetry?:
    | (({
        count,
        error
      }: {
        count: number
        error: Error
      }) => Promise<boolean> | boolean)
    | undefined
}

export type WithRetryErrorType = ErrorType

export function withRetry<TData>(
  fn: () => Promise<TData>,
  { delay: delay_ = 100, retryCount = 2, shouldRetry = () => true }: WithRetryParameters = {}
) {
  return new Promise<TData>((resolve, reject) => {
    const attemptRetry = async ({ count = 0 } = {}) => {
      const retry = async ({ error }: { error: Error }) => {
        const delay = typeof delay_ === "function" ? delay_({ count, error }) : delay_
        if (delay) await sleep(delay)
        attemptRetry({ count: count + 1 })
      }

      try {
        const data = await fn()
        resolve(data)
      } catch (err) {
        if (count < retryCount && (await shouldRetry({ count, error: err as Error })))
          return retry({ error: err as Error })
        reject(err)
      }
    }
    attemptRetry()
  })
}
