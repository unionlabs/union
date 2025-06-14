import type { FeeIntent } from "$lib/stores/fee.svelte"
import type { TransferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { validateTransfer } from "$lib/transfer/shared/data/validation.ts"
import { type ContextFlowError, OrderCreationError } from "$lib/transfer/shared/errors"
import { checkAllowances } from "$lib/transfer/shared/services/filling/check-allowance.ts"
import {
  type BalanceCheckResult,
  checkBalanceForIntent,
} from "$lib/transfer/shared/services/filling/check-balance.ts"
import {
  FillingState,
  getFillingState,
  type TransferArgs,
} from "$lib/transfer/shared/services/filling/check-filling.ts"
import {
  createContext,
  type TransferContext,
} from "$lib/transfer/shared/services/filling/create-context.ts"
import { createOrdersBatch } from "$lib/transfer/shared/services/filling/create-orders.ts"
import { Data, Effect, Match, Option } from "effect"
import { pipe } from "effect/Function"

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
  CreateOrders: {
    context: TransferContext
  }
  CheckReceiver: {
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
  CreateOrders,
  CheckReceiver,
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
  fee: Option.Option<FeeIntent>,
) => {
  return CreateContextState.$match(cts, {
    Empty: () => Effect.void,
    Filling: () => {
      const state = getFillingState(transfer, fee)

      return FillingState.$match(state, {
        Empty: () => Effect.void,
        NoWallet: () => Effect.succeed(ok(Empty(), "Connect wallet")),
        SourceChainMissing: () => Effect.succeed(ok(Empty(), "Select from chain")),
        SourceWalletMissing: () => Effect.succeed(ok(Empty(), "Connect wallet")),
        BaseTokenMissing: () => Effect.succeed(ok(Empty(), "Select asset")),
        DestinationMissing: () => Effect.succeed(ok(Empty(), "Select to chain")),
        NoRoute: () => Effect.succeed(ok(Empty(), "No route")),
        NoContract: () => Effect.succeed(ok(Empty(), "No ucs03 contract")),
        EmptyAmount: () => Effect.succeed(ok(Empty(), "Enter amount")),
        InvalidAmount: () => Effect.succeed(ok(Empty(), "Invalid amount")),
        ReceiverMissing: () => Effect.succeed(ok(Empty(), "Select receiver")),
        NoFee: () => Effect.succeed(ok(Empty(), "No fee")),
        Ready: (args) => Effect.succeed(ok(Validation({ args }), "Validating...")),
      })
    },

    Validation: ({ args }) => {
      // TODO: validate fee intent
      const validation = validateTransfer(args)
      if (validation._tag !== "Success") {
        return Effect.succeed(fail("Validation failed"))
      }
      return Effect.succeed(ok(CreateContext({ args }), "Creating context..."))
    },

    CreateContext: ({ args }) => {
      const contextOpt = createContext(args)

      if (Option.isNone(contextOpt)) {
        return Effect.succeed(fail("Failed to create context"))
      }

      const context = contextOpt.value
      return Effect.succeed(
        ok(CheckBalance({ context }), "Checking receiver..."),
      )
    },

    CheckBalance: ({ context }) => {
      const nextState = pipe(
        Match.type<BalanceCheckResult>(),
        Match.tagsExhaustive({
          HasEnough: () =>
            Effect.succeed(
              ok(CheckAllowance({ context }), "Checking allowance..."),
            ),
          InsufficientFunds: () => Effect.succeed(ok(Empty(), "Insufficient balance")),
        }),
      )
      return pipe(
        context,
        checkBalanceForIntent,
        Effect.flatMap(nextState),
        Effect.catchAll((error) => Effect.succeed(fail("Check balance failed", error))),
      )
    },

    CheckAllowance: ({ context }) => {
      return checkAllowances(context).pipe(
        Effect.map((allowancesOpt) => {
          const allowances = Option.getOrElse(allowancesOpt, () => [])

          const updatedContext = {
            ...context,
            allowances: allowances.length > 0 ? Option.some(allowances) : Option.none(),
          }

          return ok(
            CreateOrders({ context: updatedContext }),
            "Creating orders...",
          )
        }),
        Effect.catchAll((error) => Effect.succeed(fail("Allowance check failed", error))),
      )
    },

    CreateOrders: ({ context }) =>
      createOrdersBatch(context).pipe(
        Effect.flatMap((batchOpt) => {
          if (Option.isNone(batchOpt)) {
            return Effect.succeed(
              fail(
                "Could not create orders",
                new OrderCreationError({ details: "No batch returned" }),
              ),
            )
          }

          const batch = batchOpt.value

          const updatedContext = {
            ...context,
            instruction: Option.some(batch),
          }

          return Effect.succeed(
            ok(
              CheckReceiver({ context: updatedContext }),
              "Checking receiver...",
            ),
          )
        }),
        Effect.catchAll((error) => Effect.succeed(fail("Order creation failed", error))),
      ),

    CheckReceiver: ({ context }) =>
      Effect.sleep(1000).pipe(
        Effect.flatMap(() => Effect.succeed(ok(CreateSteps({ context }), "Final steps..."))),
      ),

    CreateSteps: ({ context }) => {
      return Effect.succeed(complete("Transfer ready", context))
    },
  })
}
