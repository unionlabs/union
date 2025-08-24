/**
 * This module allows usage of Safe wallet.
 *
 * @since 0.0.0
 */

import SafeAppsSDK, { type Opts } from "@safe-global/safe-apps-sdk"
import { foreverSchedule } from "@unionlabs/sdk/Constants"
import { Hex } from "@unionlabs/sdk/schema/hex"
import { Data, Effect, Option as O, pipe } from "effect"

/**
 * @category errors
 * @since 0.0.0
 */
export class SafeError extends Data.TaggedError("@unionlabs/sdk-evm/Safe/SafeError")<{
  message: string
  cause?: unknown | undefined
}> {}

/**
 * @category services
 * @since 0.0.0
 */
export class Safe extends Effect.Service<Safe>()("@unionlabs/sdk-evm/Safe", {
  effect: Effect.fn(function*(opts: Opts) {
    const sdk = yield* pipe(
      Effect.try(() => new SafeAppsSDK(opts)),
      Effect.mapError((cause) =>
        new SafeError({
          message: "Could not initialize Safe SDK",
          cause,
        })
      ),
    )

    return {
      resolveTxHash: Effect.fn((hash: Hex) =>
        pipe(
          Effect.tryPromise({
            try: () => sdk.txs.getBySafeTxHash(hash),
            catch: (cause) =>
              new SafeError({
                message: "Could not get transaction by hash.",
                cause,
              }),
          }),
          Effect.tap((details) => Effect.logInfo("safe details", details)),
          Effect.flatMap((details) => O.fromNullable(details.txHash)),
          Effect.catchTag("NoSuchElementException", (cause) =>
            new SafeError({
              message: "Safe txHash is nullish.",
              cause,
            })),
          Effect.tapErrorCause((cause) =>
            Effect.logError(
              "Could not get transaction details by Safe transaction hash. Retrying...",
              cause,
            )
          ),
          Effect.annotateLogs({
            opts,
            hash,
          }),
          Effect.retry(foreverSchedule),
        )
      ),
    } as const
  }),
}) {
}
