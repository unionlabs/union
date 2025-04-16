import { Data, Effect, Match, Option } from "effect"
import type { Transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import {
  type BalanceCheckResult,
  checkBalanceForIntents
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
import { createIntents } from "$lib/components/Transfer/state/filling/create-intents.ts"
import { constVoid } from "effect/Function"
import type { TransferIntents } from "$lib/components/Transfer/state/filling/create-intents.ts"

export type StateResult = {
  nextState: Option.Option<CreateTransferState>
  message: string
  intents: Option.Option<TransferIntents>
  error: Option.Option<TransferFlowError>
}

export type CreateTransferState = Data.TaggedEnum<{
  Empty: {}
  Filling: {}
  Validation: { args: TransferArgs }
  CreateIntents: { args: TransferArgs }
  CheckBalance: {
    intents: TransferIntents
  }
  CheckAllowance: {
    intents: TransferIntents
  }
  CreateOrders: {
    intents: TransferIntents
  }
  CheckReciever: {
    intents: TransferIntents
  }
  CreateSteps: {
    intents: TransferIntents
  }
}>

export const CreateTransferState = Data.taggedEnum<CreateTransferState>()
const {
  Empty,
  Validation,
  CreateIntents,
  CheckBalance,
  CheckAllowance,
  CreateOrders,
  CheckReciever,
  CreateSteps
} = CreateTransferState

const fail = (msg: string, err?: TransferFlowError): StateResult => ({
  nextState: Option.none(),
  message: msg,
  intents: Option.none(),
  error: err ? Option.some(err) : Option.none()
})

const ok = (state: CreateTransferState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  intents: Option.none(),
  error: Option.none()
})

const complete = (msg: string, intents: TransferIntents): StateResult => ({
  nextState: Option.none(),
  message: msg,
  intents: Option.some(intents),
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
      return Effect.succeed(ok(CreateIntents({ args }), "Validation passed"))
    },

    CreateIntents: ({ args }) => {
      const intentsOpt = createIntents(args)

      if (Option.isNone(intentsOpt)) {
        return Effect.succeed(fail("Failed to create intents"))
      }

      const intents = intentsOpt.value

      return Effect.succeed(ok(CheckBalance({ intents }), "Checking receiver..."))
    },

    CheckBalance: ({ intents }) =>
      checkBalanceForIntents(intents).pipe(
        Effect.flatMap((result: BalanceCheckResult) =>
          Match.type<BalanceCheckResult>().pipe(
            Match.tag("HasEnough", () =>
              Effect.succeed(ok(CheckAllowance({ intents }), "Checking allowance..."))
            ),
            Match.tag("InsufficientFunds", () =>
              Effect.succeed(ok(Empty(), "Insufficient balance"))
            ),
            Match.exhaustive
          )(result)
        )
      ),

    CheckAllowance: ({ intents }) =>
      checkAllowances(intents).pipe(
        Effect.map(allowancesOpt => {
          const allowances = Option.getOrElse(allowancesOpt, () => [])

          const updatedIntents = intents.map(intent => {
            const matched = allowances.find(a => a.token === intent.context.baseToken)
            return {
              ...intent,
              allowances: Option.fromNullable(matched)
            }
          })

          return ok(CreateOrders({ intents: updatedIntents }), "Creating orders...")
        }),
        Effect.catchAll(error => Effect.succeed(fail("Allowance check failed", error)))
      ),

    CreateOrders: ({ intents }) =>
      createOrdersBatch(intents).pipe(
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

          const updatedIntents = intents.map(intent => ({
            ...intent,
            instructions: Option.some(batch),
          }))

          return Effect.succeed(
            ok(CheckReciever({ intents: updatedIntents }), "Checking receiver...")
          )
        }),
        Effect.catchAll(error => Effect.succeed(fail("Order creation failed", error)))
      ),

    //Move check reciever in here
    CheckReciever: ({ intents }) =>
      Effect.sleep(1000).pipe(
        Effect.flatMap(() =>
          Effect.succeed(ok(CreateSteps({ intents }), "Final steps..."))
        )
      ),

    CreateSteps: ({ intents }) => {
      return Effect.succeed(complete("Transfer ready", intents))
    }
  })
}
