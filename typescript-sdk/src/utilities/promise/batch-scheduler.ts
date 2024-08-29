/**
 * @source https://github.com/wevm/viem/blob/main/src/utils/promise/createBatchScheduler.ts
 */
import type { ErrorType } from "../../types.ts"

type Resolved<TReturnType extends ReadonlyArray<unknown> = any> = [
  result: TReturnType[number],
  results: TReturnType
]

type PendingPromise<TReturnType extends ReadonlyArray<unknown> = any> = {
  resolve?: ((data: Resolved<TReturnType>) => void) | undefined
  reject?: ((reason?: unknown) => void) | undefined
}

type SchedulerItem = { args: unknown; pendingPromise: PendingPromise }

type BatchResultsCompareFn<TResult = unknown> = (a: TResult, b: TResult) => number

type CreateBatchSchedulerArguments<
  TParameters = unknown,
  TReturnType extends ReadonlyArray<unknown> = ReadonlyArray<unknown>
> = {
  fn: (args: Array<TParameters>) => Promise<TReturnType>
  id: number | string
  shouldSplitBatch?: ((args: Array<TParameters>) => boolean) | undefined
  wait?: number | undefined
  sort?: BatchResultsCompareFn<TReturnType[number]> | undefined
}

type CreateBatchSchedulerReturnType<
  TParameters = unknown,
  TReturnType extends ReadonlyArray<unknown> = ReadonlyArray<unknown>
> = {
  flush: () => void
  schedule: TParameters extends undefined
    ? (args?: TParameters | undefined) => Promise<Resolved<TReturnType>>
    : (args: TParameters) => Promise<Resolved<TReturnType>>
}

export type CreateBatchSchedulerErrorType = ErrorType

const schedulerCache = /*#__PURE__*/ new Map<number | string, Array<SchedulerItem>>()

/** @internal */
export function createBatchScheduler<TParameters, TReturnType extends ReadonlyArray<unknown>>({
  fn,
  id,
  shouldSplitBatch,
  wait = 0,
  sort
}: CreateBatchSchedulerArguments<TParameters, TReturnType>): CreateBatchSchedulerReturnType<
  TParameters,
  TReturnType
> {
  const exec = async () => {
    const scheduler = getScheduler()
    flush()

    const args = scheduler.map(({ args }) => args)

    if (args.length === 0) return

    fn(args as Array<TParameters>)
      .then(data => {
        if (sort && Array.isArray(data)) data.sort(sort)
        for (let index = 0; index < scheduler.length; index++) {
          // @ts-expect-error
          const { pendingPromise } = scheduler[index]
          pendingPromise.resolve?.([data[index], data])
        }
      })
      .catch(error => {
        for (let index = 0; index < scheduler.length; index++) {
          // @ts-expect-error
          const { pendingPromise } = scheduler[index]
          pendingPromise.reject?.(error)
        }
      })
  }

  const flush = () => schedulerCache.delete(id)

  const getBatchedArgs = () => getScheduler().map(({ args }) => args) as Array<TParameters>

  const getScheduler = () => schedulerCache.get(id) || []

  const setScheduler = (item: SchedulerItem) => schedulerCache.set(id, [...getScheduler(), item])

  return {
    flush,
    async schedule(args: TParameters) {
      const pendingPromise: PendingPromise<TReturnType> = {}
      const promise = new Promise<Resolved<TReturnType>>((resolve, reject) => {
        pendingPromise.resolve = resolve
        pendingPromise.reject = reject
      })

      const split = shouldSplitBatch?.([...getBatchedArgs(), args])

      if (split) exec()

      const hasActiveScheduler = getScheduler().length > 0
      if (hasActiveScheduler) {
        setScheduler({ args, pendingPromise })
        return promise
      }

      setScheduler({ args, pendingPromise })
      setTimeout(exec, wait)
      return promise
    }
  } as unknown as CreateBatchSchedulerReturnType<TParameters, TReturnType>
}
