import { Data, Effect, Option } from "effect"
import type { Transfer, TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import { checkBalanceForIntents } from "$lib/components/Transfer/state/filling/check-balance.ts"
import { createOrdersBatch } from "$lib/components/Transfer/state/filling/create-orders.ts"
import {
  checkAllowances,
  type ApprovalStep
} from "$lib/components/Transfer/state/filling/check-allowance.ts"
import {
  InsufficientFundsError,
  MissingTransferFieldsError,
  OrderCreationError,
  type TransferFlowError
} from "$lib/components/Transfer/state/errors.ts"
import type { Instruction } from "@unionlabs/sdk/ucs03/instruction"

export type StateResult = {
  nextState: Option.Option<CreateTransferState>
  message: string
  orders: Option.Option<Array<Instruction>>
  allowances: Option.Option<Array<ApprovalStep>>
  error: Option.Option<TransferFlowError>
}

export type CreateTransferState = Data.TaggedEnum<{
  Filling: {}
  CreateIntents: {}
  CheckBalance: { intents: TransferIntents }
  CheckAllowance: {}
  CreateOrders: { allowances: Array<ApprovalStep> }
  CreateSteps: { allowances: Array<ApprovalStep>; orders: Array<Instruction> }
}>

export const CreateTransferState = Data.taggedEnum<CreateTransferState>()
const { CreateIntents, CheckBalance, CheckAllowance, CreateOrders, CreateSteps } =
  CreateTransferState

const fail = (msg: string, err?: TransferFlowError): StateResult => ({
  nextState: Option.none<CreateTransferState>(),
  message: msg,
  orders: Option.none<Array<Instruction>>(),
  allowances: Option.none<Array<ApprovalStep>>(),
  error: err ? Option.some<TransferFlowError>(err) : Option.none<TransferFlowError>()
})

const ok = (state: CreateTransferState, msg: string): StateResult => ({
  nextState: Option.some<CreateTransferState>(state),
  message: msg,
  orders: Option.none<Array<Instruction>>(),
  allowances: Option.none<Array<ApprovalStep>>(),
  error: Option.none<TransferFlowError>()
})

const complete = (
  msg: string,
  orders: Array<Instruction>,
  allowances: Array<ApprovalStep>
): StateResult => ({
  nextState: Option.none<CreateTransferState>(),
  message: msg,
  orders: Option.some<Array<Instruction>>(orders),
  allowances: Option.some<Array<ApprovalStep>>(allowances),
  error: Option.none<TransferFlowError>()
})

export const createTransferState = (cts: CreateTransferState, transfer: Transfer) => {
  if (
    Option.isNone(transfer.sourceChain) ||
    Option.isNone(transfer.destinationChain) ||
    Option.isNone(transfer.baseToken) ||
    Option.isNone(transfer.derivedSender) ||
    Option.isNone(transfer.parsedAmount) ||
    Option.isNone(transfer.ucs03address) ||
    Option.isNone(transfer.channel) ||
    Option.isNone(transfer.intents)
  ) {
    return Effect.succeed(
      fail("Missing arguments", new MissingTransferFieldsError({ fields: [""] }))
    )
  }

  // Safe to access `.value` after validation
  const channel = transfer.channel.value
  const ucs03address = transfer.ucs03address.value
  const source = transfer.sourceChain.value
  const destination = transfer.destinationChain.value
  const sender = transfer.derivedSender.value
  const amount = transfer.parsedAmount.value
  const intents = transfer.intents.value

  if (amount === "0" || amount === "" || BigInt(amount) === BigInt(0)) {
    return Effect.succeed(
      fail("Enter an amount", new InsufficientFundsError({ cause: "Amount is zero" }))
    )
  }

  return CreateTransferState.$match(cts, {
    Filling: () => Effect.succeed(ok(CreateIntents(), "Creating intents...")),

    CreateIntents: () => Effect.succeed(ok(CheckBalance({ intents }), "Checking balance...")),

    CheckBalance: ({ intents }) =>
      checkBalanceForIntents(source, intents).pipe(
        Effect.flatMap(hasEnough =>
          hasEnough
            ? Effect.succeed(ok(CheckAllowance(), "Checking allowance..."))
            : Effect.succeed(
                fail(
                  "Insufficient funds",
                  new InsufficientFundsError({
                    cause: "Insufficient funds"
                  })
                )
              )
        ),
        Effect.catchAll(error => Effect.succeed(fail("Balance check failed", error)))
      ),

    CheckAllowance: () =>
      checkAllowances(source, intents, sender, ucs03address).pipe(
        Effect.map(allowancesOpt => {
          const allowances = Option.getOrElse(allowancesOpt, () => [])
          return ok(CreateOrders({ allowances }), "Creating orders...")
        }),
        Effect.catchAll(error => Effect.succeed(fail("Allowance check failed", error)))
      ),

    CreateOrders: ({ allowances }) =>
      createOrdersBatch(source, destination, channel, ucs03address, intents).pipe(
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
            ok(CreateSteps({ allowances, orders: [...batch.operand] }), "Building final steps...")
          )
        }),
        Effect.catchAll(error => Effect.succeed(fail("Order creation failed", error)))
      ),

    CreateSteps: ({ allowances, orders }) =>
      Effect.succeed(complete("Transfer complete", orders, allowances))
  })
}
