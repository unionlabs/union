import { Data, Effect, Match, Option } from "effect"
import type { Transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import {
  type BalanceCheckResult,
  checkBalanceForIntent
} from "$lib/components/Transfer/state/filling/check-balance.ts"
import { createOrdersBatch } from "$lib/components/Transfer/state/filling/create-orders.ts"
import { checkAllowances } from "$lib/components/Transfer/state/filling/check-allowance.ts"
import {
  OrderCreationError,
  type TransferFlowError
} from "$lib/components/Transfer/state/errors.ts"
import {
  FillingState,
  getFillingState,
  type TransferArgs
} from "$lib/components/Transfer/state/filling/check-filling.ts"
import { validateTransfer } from "$lib/components/Transfer/validation.ts"
import {createIntents, type TransferIntent} from "$lib/components/Transfer/state/filling/create-intents.ts"
import { constVoid } from "effect/Function"

export type StateResult = {
  nextState: Option.Option<CreateTransferState>
  message: string
  intent: Option.Option<TransferIntent>
  error: Option.Option<TransferFlowError>
}

export type CreateTransferState = Data.TaggedEnum<{
  Empty: {}
  Filling: {}
  Validation: { args: TransferArgs }
  CreateIntent: { args: TransferArgs }
  CheckBalance: {
    intent: TransferIntent
  }
  CheckAllowance: {
    intent: TransferIntent
  }
  CreateOrders: {
    intent: TransferIntent
  }
  CheckReciever: {
    intent: TransferIntent
  }
  CreateSteps: {
    intent: TransferIntent
  }
}>

export const CreateTransferState = Data.taggedEnum<CreateTransferState>()
const {
  Empty,
  Validation,
  CreateIntent,
  CheckBalance,
  CheckAllowance,
  CreateOrders,
  CheckReciever,
  CreateSteps
} = CreateTransferState

const fail = (msg: string, err?: TransferFlowError): StateResult => ({
  nextState: Option.none(),
  message: msg,
  intent: Option.none(),
  error: err ? Option.some(err) : Option.none()
})

const ok = (state: CreateTransferState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  intent: Option.none(),
  error: Option.none()
})

const complete = (msg: string, intents: TransferIntent): StateResult => ({
  nextState: Option.none(),
  message: msg,
  intent: Option.some(intents),
  error: Option.none()
})

export const createTransferState = (cts: CreateTransferState, transfer: Transfer) => {
  return CreateTransferState.$match(cts, {
    Empty: constVoid,
    Filling: () => {
      const state = getFillingState(transfer)

      return FillingState.$match(state, {
        Empty: constVoid,
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
        Ready: args => Effect.succeed(ok(Validation({ args }), "Validating..."))
      })
    },

    Validation: ({ args }) => {
      const validation = validateTransfer(args)
      if (validation._tag !== "Success") {
        return Effect.succeed(fail("Validation failed"))
      }
      return Effect.succeed(ok(CreateIntent({ args }), "Validation passed"))
    },

    CreateIntent: ({ args }) => {
      const intentsOpt = createIntents(args)

      if (Option.isNone(intentsOpt)) {
        return Effect.succeed(fail("Failed to create intents"))
      }

      const intent = intentsOpt.value

      console.log({intent})

      return Effect.succeed(ok(CheckBalance({ intent }), "Checking receiver1..."))
    },

    CheckBalance: ({ intent }) => {
      console.log("[STATE] Entered CheckBalance", intent)
      return checkBalanceForIntent(intent).pipe(
        Effect.flatMap((result: BalanceCheckResult) =>
          Match.type<BalanceCheckResult>().pipe(
            Match.tag("HasEnough", () =>
              Effect.succeed(ok(CheckAllowance({ intent }), "Checking allowance..."))
            ),
            Match.tag("InsufficientFunds", () =>
              Effect.succeed(ok(Empty(), "Insufficient balance"))
            ),
            Match.exhaustive
          )(result)
        )
      )
    },

    CheckAllowance: ({ intent }) => {
      console.log("[STATE] Entered CheckAllowance", intent)
      return checkAllowances(intent).pipe(
        Effect.map(allowancesOpt => {
          const allowances = Option.getOrElse(allowancesOpt, () => [])

          const baseTokens = intent.contexts.map(c => c.baseToken)
          const relevantAllowances = allowances.filter(a =>
            baseTokens.includes(a.token)
          )

          const updatedIntent = {
            ...intent,
            allowances: relevantAllowances.length > 0
              ? Option.some(relevantAllowances)
              : Option.none()
          }

          return ok(CreateOrders({ intent: updatedIntent }), "Creating orders...")
        }),
        Effect.catchAll(error =>
          Effect.succeed(fail("Allowance check failed", error))
        )
      )
    },

    CreateOrders: ({ intent }) =>
      createOrdersBatch(intent).pipe(
        Effect.flatMap(batchOpt => {
          if (Option.isNone(batchOpt)) {
            return Effect.succeed(
              fail(
                "Could not create orders",
                new OrderCreationError({ details: "No batch returned" })
              )
            )
          }

          const batch = batchOpt.value

          const updatedIntent = {
            ...intent,
            instruction: Option.some(batch)
          }

          return Effect.succeed(
            ok(CheckReciever({ intent: updatedIntent }), "Checking receiver...")
          )
        }),

        Effect.catchAll(error =>
          Effect.succeed(fail("Order creation failed", error))
        )
      ),

    //Move check reciever in here
    CheckReciever: ({ intent }) =>
      Effect.sleep(1000).pipe(
        Effect.flatMap(() => Effect.succeed(ok(CreateSteps({ intent }), "Final steps...")))
      ),

    CreateSteps: ({ intent }) => {
      return Effect.succeed(complete("Transfer ready", intent))
    }
  })
}
