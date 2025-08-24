import type { FeeIntent } from "$lib/stores/fee.svelte"
import { createMultisigMessage } from "$lib/transfer/multisig/services/create-multisig"
import type { TransferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { validateTransfer } from "$lib/transfer/shared/data/validation"
import {
  type ContextFlowError,
  GenericFlowError,
  OrderCreationError,
} from "$lib/transfer/shared/errors"
import { checkAllowances } from "$lib/transfer/shared/services/filling/check-allowance"
import {
  type BalanceCheckResult,
  checkBalanceForIntent,
} from "$lib/transfer/shared/services/filling/check-balance"
import {
  FillingState,
  getFillingState,
  type TransferArgs,
} from "$lib/transfer/shared/services/filling/check-filling"
import {
  createContext,
  type TransferContext,
} from "$lib/transfer/shared/services/filling/create-context"
import { Data, Effect, Either as E, Match, Option, pipe } from "effect"

export type StateResult = {
  nextState: Option.Option<CreateContextState>
  message: string
  context: Option.Option<TransferContext>
  error: Option.Option<ContextFlowError>
}

export type CreateContextState = Data.TaggedEnum<{
  Empty: {}
  Filling: {}
  Validation: { args: TransferArgs }
  CreateContext: { args: TransferArgs }
  CheckBalance: {
    context: TransferContext
  }
  CheckAllowance: {
    context: TransferContext
  }
  CheckReceiver: {
    context: TransferContext
  }
  CreateMessage: {
    context: TransferContext
  }
  CreateSteps: {
    context: TransferContext
  }
}>

export const CreateContextState = Data.taggedEnum<CreateContextState>()
const {
  Empty,
  Validation,
  CreateContext,
  CheckBalance,
  CheckAllowance,
  CheckReceiver,
  CreateMessage,
  CreateSteps,
} = CreateContextState

const fail = (msg: string, err?: ContextFlowError): StateResult => ({
  nextState: Option.none(),
  message: msg,
  context: Option.none(),
  error: err ? Option.some(err) : Option.none(),
})

const ok = (state: CreateContextState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  context: Option.none(),
  error: Option.none(),
})

const complete = (msg: string, context: TransferContext): StateResult => ({
  nextState: Option.none(),
  message: msg,
  context: Option.some(context),
  error: Option.none(),
})

export const createContextState = (
  cts: CreateContextState,
  transfer: TransferData,
  fee: Option.Option<E.Either<FeeIntent, string>>,
) => {
  return CreateContextState.$match(cts, {
    Empty: () => Effect.void,
    Filling: () => {
      const state = getFillingState(transfer, fee)

      return FillingState.$match(state, {
        Empty: () => Effect.void,
        NoWallet: () => Effect.succeed(ok(Empty(), "Enter sender address")),
        SourceChainMissing: () => Effect.succeed(ok(Empty(), "Select from chain")),
        SourceWalletMissing: () => Effect.succeed(ok(Empty(), "Connect wallet")),
        BaseTokenMissing: () => Effect.succeed(ok(Empty(), "Select asset")),
        DestinationMissing: () => Effect.succeed(ok(Empty(), "Select to chain")),
        NoRoute: () => Effect.succeed(ok(Empty(), "No route")),
        NoContract: () => Effect.succeed(ok(Empty(), "No ucs03 contract")),
        EmptyAmount: () => Effect.succeed(ok(Empty(), "Enter amount")),
        NoFee: ({ message }) => Effect.succeed(ok(Empty(), message ?? "Loading fee...")),
        InvalidAmount: () => Effect.succeed(ok(Empty(), "Invalid amount")),
        ReceiverMissing: () => Effect.succeed(ok(Empty(), "Select receiver")),
        Ready: (args) => Effect.succeed(ok(Validation({ args }), "Validating...")),
        Generic: ({ message }) => Effect.succeed(ok(Empty(), message ?? "Unknown error.")),
      })
    },

    Validation: ({ args }) => {
      // const validation = validateTransfer(args)
      // if (validation._tag !== "Success") {
      //   return Effect.succeed(fail("Validation failed"))
      // }
      return Effect.succeed(ok(CreateContext({ args }), "Creating context..."))
    },

    CreateContext: ({ args }) => {
      return pipe(
        createContext(args),
        Effect.mapBoth({
          onFailure: (cause) =>
            fail(cause.message, new GenericFlowError({ message: cause.message, cause })),
          onSuccess: (context) => ok(CheckBalance({ context }), "something"),
        }),
        Effect.catchAllDefect((defect) => Effect.logError("[CreateContext] Defect:", defect)),
        Effect.merge,
      )
    },

    CheckBalance: ({ context }) => {
      return checkBalanceForIntent(context).pipe(
        Effect.flatMap((result: BalanceCheckResult) =>
          Match.type<BalanceCheckResult>().pipe(
            Match.tag("HasEnough", () =>
              Effect.succeed(
                ok(CheckAllowance({ context }), "Checking allowance..."),
              )),
            Match.tag(
              "InsufficientFunds",
              () => Effect.succeed(ok(Empty(), "Insufficient balance")),
            ),
            Match.exhaustive,
          )(result)
        ),
      )
    },

    CheckAllowance: ({ context }) => {
      return checkAllowances(context).pipe(
        Effect.map((allowances) => {
          const updatedContext = {
            ...context,
            allowances,
          }

          return ok(
            CheckReceiver({ context: updatedContext }),
            "Checking receiver...",
          )
        }),
        Effect.catchAll((error) => Effect.succeed(fail("Allowance check failed", error))),
      )
    },

    CheckReceiver: ({ context }) =>
      Effect.sleep(1000).pipe(
        Effect.flatMap(() => Effect.succeed(ok(CreateMessage({ context }), "Creating message..."))),
      ),

    CreateMessage: ({ context }) =>
      createMultisigMessage(context).pipe(
        Effect.flatMap((message) => {
          const updatedContext: TransferContext = {
            ...context,
            message: Option.some(message),
          }

          return Effect.succeed(
            ok(CreateSteps({ context: updatedContext }), "Final steps..."),
          )
        }),
        Effect.catchAll((error) => Effect.succeed(fail("Message creation failed", error))),
      ),

    CreateSteps: ({ context }) => {
      return Effect.succeed(complete("Export message", context))
    },
  })
}
