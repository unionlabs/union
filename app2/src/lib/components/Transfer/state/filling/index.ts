import {Data, Effect, Option} from "effect"
import type {Transfer, TransferIntents} from "$lib/components/Transfer/transfer.svelte.ts"
import {checkBalanceForIntents} from "$lib/components/Transfer/state/filling/check-balance.ts"
import {createOrdersBatch} from "$lib/components/Transfer/state/filling/create-orders.ts"
import {type ApprovalStep, checkAllowances} from "$lib/components/Transfer/state/filling/check-allowance.ts"
import {
  InsufficientFundsError,
  OrderCreationError,
  type TransferFlowError
} from "$lib/components/Transfer/state/errors.ts"
import type {Instruction} from "@unionlabs/sdk/ucs03/instruction"
import {FillingState, getFillingState, type TransferArgs} from "$lib/components/Transfer/state/filling/check-filling.ts"
import {validateTransfer} from "$lib/components/Transfer/validation.ts"
import {createIntents} from "$lib/components/Transfer/state/filling/create-intents.ts"
import {constVoid} from "effect/Function"

export type StateResult = {
  nextState: Option.Option<CreateTransferState>
  message: string
  orders: Option.Option<Array<Instruction>>
  allowances: Option.Option<Array<ApprovalStep>>
  error: Option.Option<TransferFlowError>
}

export type CreateTransferState = Data.TaggedEnum<{
  Empty: {}
  Filling: {}
  Validation: { args: TransferArgs }
  CreateIntents: { args: TransferArgs }
  CheckBalance: {
    args: TransferArgs
    intents: TransferIntents
  }
  CheckAllowance: {
    args: TransferArgs
    intents: TransferIntents
  }
  CreateOrders: {
    args: TransferArgs
    intents: TransferIntents
    allowances: Array<ApprovalStep>
  }
  CreateSteps: {
    args: TransferArgs
    allowances: Array<ApprovalStep>
    orders: Array<Instruction>
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
  CreateSteps
} = CreateTransferState

const fail = (msg: string, err?: TransferFlowError): StateResult => ({
  nextState: Option.none(),
  message: msg,
  orders: Option.none(),
  allowances: Option.none(),
  error: err ? Option.some(err) : Option.none()
})

const ok = (state: CreateTransferState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  orders: Option.none(),
  allowances: Option.none(),
  error: Option.none()
})

const complete = (
  msg: string,
  orders: Array<Instruction>,
  allowances: Array<ApprovalStep>
): StateResult => ({
  nextState: Option.none(),
  message: msg,
  orders: Option.some(orders),
  allowances: Option.some(allowances),
  error: Option.none()
})

export const createTransferState = (cts: CreateTransferState, transfer: Transfer) => {
  return CreateTransferState.$match(cts, {
    Empty: constVoid,
    Filling: () => {
      const state = getFillingState(transfer)

      return FillingState.$match(state, {
        Empty: constVoid,
        WalletMissing: () => Effect.succeed(ok(Empty(), "Connect wallet")),
        SourceChainMissing: () => Effect.succeed(ok(Empty(), "Select from chain")),
        ChainWalletMissing: () => Effect.succeed(ok(Empty(), "Connect wallet")),
        BaseTokenMissing: () => Effect.succeed(ok(Empty(), "Select asset")),
        DestinationMissing: () => Effect.succeed(ok(Empty(), "Select to chain")),
        NoRoute: () => Effect.succeed(ok(Empty(), "No route")),
        NoContract: () => Effect.succeed(ok(Empty(), "No ucs3 contract")),
        InvalidAmount: () => Effect.succeed(ok(Empty(), "Invalid amount")),
        ReceiverMissing: () => Effect.succeed(ok(Empty(), "Select receiver")),
        Ready: args => Effect.succeed(ok(Validation({ args }), "Validating..."))
      })
    },

    Validation: ({ args }) => {
      const validation = validateTransfer(args)

      if (validation._tag !== "Success") {
        console.log(validation)
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

      return Effect.succeed(ok(CheckBalance({ args, intents }), "Checking balance..."))
    },

    CheckBalance: ({ args, intents }) =>
      checkBalanceForIntents(args.sourceChain, intents).pipe(
        Effect.flatMap(hasEnough =>
          hasEnough
            ? Effect.succeed(ok(CheckAllowance({ args, intents }), "Checking allowance..."))
            : Effect.succeed(
                fail(
                  "Insufficient funds",
                  new InsufficientFundsError({ cause: "Insufficient funds" })
                )
              )
        ),
        Effect.catchAll(error => Effect.succeed(fail("Balance check failed", error)))
      ),

    CheckAllowance: ({ args, intents }) =>
      checkAllowances(args.sourceChain, intents, args.receiver, args.ucs03address).pipe(
        Effect.map(allowancesOpt => {
          const allowances = Option.getOrElse(allowancesOpt, () => [])
          return ok(CreateOrders({ args, intents, allowances }), "Creating orders...")
        }),
        Effect.catchAll(error => Effect.succeed(fail("Allowance check failed", error)))
      ),

    CreateOrders: ({ args, intents, allowances }) =>
      createOrdersBatch(
        args.sourceChain,
        args.destinationChain,
        args.channel,
        args.ucs03address,
        intents
      ).pipe(
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
          return Effect.succeed(
            ok(
              CreateSteps({ args, allowances, orders: [...batch.operand] }),
              "Building final steps..."
            )
          )
        }),
        Effect.catchAll(error => Effect.succeed(fail("Order creation failed", error)))
      ),

    CreateSteps: ({ allowances, orders }) => {
      console.log({ allowances, orders })
      return Effect.succeed(complete("Transfer ready", orders, allowances))
    }
  })
}
