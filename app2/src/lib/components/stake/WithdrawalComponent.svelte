<script lang="ts">
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Button from "$lib/components/ui/Button.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import {
  DESTINATION_CHANNEL_ID,
  ETHEREUM_CHAIN_ID,
  SOURCE_CHANNEL_ID,
  UCS03_EVM_ADDRESS,
  UCS03_ZKGM,
  UNION_CHAIN_ID,
} from "$lib/stake/config"
import { predictProxy } from "$lib/stake/instantiate2"
import { BatchResponseSchema, type UnstakeRequest, UnstakeRequestSchema } from "$lib/stake/schemas"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import {
  Batch,
  Call,
  TokenOrder,
  Ucs03,
  Ucs05,
  Utils,
  ZkgmClient,
  ZkgmClientRequest,
} from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { Evm, EvmZkgmClient, Safe } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { EU_STAKING_HUB } from "@unionlabs/sdk/Constants"
import { U_BANK, U_ERC20, U_SOLVER_ON_ETH_METADATA } from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import { BigDecimal, Data, Effect, Layer, Match, pipe, Schema } from "effect"
import * as A from "effect/Array"
import * as O from "effect/Option"
import { custom } from "viem"
import { sepolia } from "viem/chains"
import StatusDisplay from "./StatusDisplay.svelte"

const UCS03_EVM = Ucs05.EvmDisplay.make({ address: UCS03_EVM_ADDRESS })

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<TokenType>
  onWithdrawSuccess?: () => void
}

let {
  evmChain,
  uOnEvmToken,
  onWithdrawSuccess,
}: Props = $props()

type WithdrawalState = Data.TaggedEnum<{
  Ready: {}
  Loading: {}
  Success: { message: string }
  Error: { message: string }
}>

const WithdrawalState = Data.taggedEnum<WithdrawalState>()

let withdrawalState = $state<WithdrawalState>(WithdrawalState.Ready())

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

const isReady = $derived(WithdrawalState.$is("Ready")(withdrawalState))
const isLoading = $derived(WithdrawalState.$is("Loading")(withdrawalState))
const isSuccess = $derived(WithdrawalState.$is("Success")(withdrawalState))
const isError = $derived(WithdrawalState.$is("Error")(withdrawalState))

// Query user's withdrawal data
const withdrawalData = runPromiseExit$(() => {
  void WalletStore.evmAddress

  return pipe(
    WalletStore.evmAddress,
    O.match({
      onNone: () => Effect.succeed({ userBatches: [] }),
      onSome: (address) =>
        Effect.gen(function*() {
          const proxy = yield* predictProxy({
            path: 0n,
            channel: DESTINATION_CHANNEL_ID,
            sender: address,
          })

          // Query unstake requests
          const unstakeRequests = yield* pipe(
            Cosmos.queryContract(EU_STAKING_HUB, {
              unstake_requests_by_staker: { staker: proxy.address },
            }),
            Effect.flatMap(Schema.decodeUnknown(Schema.Array(UnstakeRequestSchema))),
          )

          if (unstakeRequests.length === 0) {
            return { userBatches: [] }
          }

          // Query batch statuses
          const batchIds = unstakeRequests.map(req => req.batch_id)
          const batchesResponse = yield* pipe(
            Cosmos.queryContract(EU_STAKING_HUB, {
              batches_by_ids: { batch_ids: batchIds },
            }),
            Effect.flatMap(Schema.decodeUnknown(BatchResponseSchema)),
          )

          const processRequest = (request: UnstakeRequest) =>
            Effect.gen(function*() {
              const batch = batchesResponse.batches.find(b => b.batch_id === request.batch_id)
              if (!batch || batch.batch.status !== "received") {
                return O.none<{ batchId: string; withdrawableAmount: BigDecimal.BigDecimal }>()
              }

              return yield* pipe(
                O.all([
                  BigDecimal.fromString(request.amount),
                  BigDecimal.fromString(batch.batch.total_lst_to_burn),
                  BigDecimal.fromString(batch.batch.received_native_unstaked),
                ]),
                O.match({
                  onNone: () =>
                    Effect.succeed(
                      O.none<{ batchId: string; withdrawableAmount: BigDecimal.BigDecimal }>(),
                    ),
                  onSome: ([userAmount, totalLstToBurn, receivedNativeUnstaked]) =>
                    Effect.gen(function*() {
                      const userShare = BigDecimal.isZero(totalLstToBurn)
                        ? BigDecimal.fromBigInt(0n)
                        : yield* BigDecimal.divide(
                          BigDecimal.multiply(userAmount, receivedNativeUnstaked),
                          totalLstToBurn,
                        ).pipe(
                          Effect.mapError(() =>
                            new Error("Division by zero in user share calculation")
                          ),
                        )

                      return O.some({
                        batchId: request.batch_id,
                        withdrawableAmount: userShare,
                      })
                    }),
                }),
              )
            })

          const userBatchOptions = yield* Effect.all(
            unstakeRequests.map(processRequest),
            { concurrency: "unbounded" },
          )

          const userBatches = pipe(
            userBatchOptions,
            A.filterMap((opt) => opt),
          )

          return { userBatches }
        }),
    }),
    Effect.provide(Cosmos.Client.Live("https://rpc.union-testnet-10.union.chain.kitchen")),
  )
})

// Extract user batches - simple pattern like other components
const userBatches = $derived(
  O.isSome(withdrawalData.current) && withdrawalData.current.value._tag === "Success"
    ? withdrawalData.current.value.value.userBatches
    : [],
)

// Total withdrawable amount (always shows all available)
const totalWithdrawableAmount = $derived(
  userBatches.reduce(
    (sum, batch) => BigDecimal.sum(sum, batch.withdrawableAmount),
    BigDecimal.fromBigInt(0n),
  ),
)

// Always use all batches for withdrawal
const withdrawableBatches = $derived(userBatches)
const withdrawableAmount = $derived(totalWithdrawableAmount)

// Get user batch information for message display
const userBatchCount = $derived(userBatches.length)
const totalWithdrawableFormatted = $derived(
  pipe(
    totalWithdrawableAmount,
    BigDecimal.round({ mode: "from-zero", scale: 4 }),
    Utils.formatBigDecimal,
  ),
)

// Execute withdrawal transaction following Bond/Unbond pattern
const executeWithdraw = (
  sender: Ucs05.EvmDisplay,
  batches: Array<{ batchId: string; withdrawableAmount: BigDecimal.BigDecimal }>,
) =>
  Effect.gen(function*() {
    const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
    const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)
    const proxy = yield* predictProxy({
      path: 0n,
      channel: DESTINATION_CHANNEL_ID,
      sender,
    })

    // Create withdraw calls for each batch
    const withdrawCalls = yield* Effect.all(
      batches.map(batch =>
        pipe(
          {
            withdraw: {
              withdraw_to_address: proxy.address,
              batch_id: batch.batchId,
            },
          } as const,
          Schema.encode(JsonFromBase64),
          Effect.map((msg) => ({
            wasm: {
              execute: {
                contract_addr: EU_STAKING_HUB.address,
                msg,
                funds: [],
              },
            },
          } as const)),
        )
      ),
    )

    const totalAmountBigDecimal = batches.reduce(
      (sum, batch) => BigDecimal.sum(sum, batch.withdrawableAmount),
      BigDecimal.fromBigInt(0n),
    )

    const totalAmount = pipe(
      totalAmountBigDecimal,
      BigDecimal.normalize,
      (normalized) =>
        normalized.scale >= 0
          ? normalized.value / (10n ** BigInt(normalized.scale))
          : normalized.value * (10n ** BigInt(-normalized.scale)),
    )

    const tokenOrder = yield* TokenOrder.make({
      source: unionChain,
      destination: ethereumChain,
      sender: proxy,
      receiver: sender,
      baseToken: U_BANK,
      baseAmount: totalAmount,
      quoteToken: U_ERC20,
      quoteAmount: totalAmount,
      kind: "solve",
      metadata: U_SOLVER_ON_ETH_METADATA,
      version: 2,
    })

    const salt = yield* Utils.generateSalt("cosmos")
    const timeout_timestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()

    const sendCall = yield* pipe(
      tokenOrder,
      TokenOrder.encodeV2,
      Effect.flatMap(Schema.encode(Ucs03.Ucs03WithInstructionFromHex)),
      Effect.map((instruction) => ({
        send: {
          channel_id: DESTINATION_CHANNEL_ID,
          timeout_height: 0n,
          timeout_timestamp,
          salt,
          instruction,
        },
      } as const)),
      Effect.flatMap(Schema.encode(JsonFromBase64)),
      Effect.map((msg) => ({
        wasm: {
          execute: {
            contract_addr: UCS03_ZKGM.address,
            msg,
            funds: [
              { denom: "au", amount: totalAmount },
            ],
          },
        },
      })),
    )

    const calls = yield* pipe(
      [...withdrawCalls, sendCall],
      Schema.decode(HexFromJson),
      Effect.map((contractCalldata) =>
        Call.make({
          sender,
          eureka: false,
          contractAddress: proxy,
          contractCalldata,
        })
      ),
    )

    const batchInstruction = Batch.make([
      calls,
    ])

    const request = ZkgmClientRequest.make({
      source: ethereumChain,
      destination: unionChain,
      channelId: SOURCE_CHANNEL_ID,
      ucs03Address: UCS03_EVM.address,
      instruction: batchInstruction,
    })

    const client = yield* ZkgmClient.ZkgmClient
    return yield* client.execute(request)
  })

let shouldWithdraw = $state<boolean>(false)

runPromiseExit$(() =>
  shouldWithdraw
    ? Effect.gen(function*() {
      const validatedData = O.all({
        sender: WalletStore.evmAddress,
        chain: evmChain,
      })

      if (O.isNone(validatedData) || BigDecimal.isZero(withdrawableAmount)) {
        withdrawalState = WithdrawalState.Error({
          message: "Missing required data: wallet address, withdrawable batches, or chain",
        })
        shouldWithdraw = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const { sender, chain } = validatedData.value
      const batches = withdrawableBatches

      withdrawalState = WithdrawalState.Loading()

      const VIEM_CHAIN = sepolia
      const connectorClient = yield* getWagmiConnectorClient
      const isSafeWallet = getLastConnectedWalletId() === "safe"

      if (!isSafeWallet) {
        yield* switchChain(VIEM_CHAIN)
      }

      const maybeSafe = Match.value(isSafeWallet).pipe(
        Match.when(true, () => Safe.Safe.Default(safeOpts)),
        Match.when(false, () => Layer.empty),
        Match.exhaustive,
      )

      const publicClient = Evm.PublicClient.Live({
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })

      const walletClient = Evm.WalletClient.Live({
        account: connectorClient.account,
        chain: VIEM_CHAIN,
        transport: custom(connectorClient),
      })

      const { txHash } = yield* executeWithdraw(sender, batches).pipe(
        Effect.provide(Layer.mergeAll(
          maybeSafe,
          EvmZkgmClient.layerWithoutWallet,
          walletClient,
          publicClient,
          ChainRegistry.Default,
        )),
      )

      withdrawalState = WithdrawalState.Success({ message: "Withdrawal submitted successfully" })
      shouldWithdraw = false
      onWithdrawSuccess?.()

      setTimeout(() => {
        if (WithdrawalState.$is("Success")(withdrawalState)) {
          withdrawalState = WithdrawalState.Ready()
        }
      }, 5000)
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          const errorDetails = extractErrorDetails(error) as any
          const fullError = errorDetails?.cause?.cause?.shortMessage
            || errorDetails?.cause?.message
            || errorDetails?.message
            || JSON.stringify(errorDetails)
          const shortMessage = String(fullError).split(".")[0]

          withdrawalState = WithdrawalState.Error({ message: shortMessage })
          shouldWithdraw = false
          return yield* Effect.void
        })
      ),
    )
    : Effect.void
)

const handleWithdraw = () => {
  if (isLoading || BigDecimal.isZero(withdrawableAmount)) {
    return
  }

  Match.value(WalletStore.evmAddress).pipe(
    Match.when(O.isNone, () => uiStore.openWalletModal()),
    Match.orElse(() => {
      shouldWithdraw = true
    }),
  )
}
</script>

<div class="flex grow flex-col gap-4">
  <!-- Available Withdrawals Section -->
  <div class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3 space-y-3">
    <div class="flex justify-between items-center">
      <span class="text-xs text-zinc-500">Available to Withdraw</span>
      <span class="text-sm font-medium text-zinc-200">
        {#if userBatches.length > 0}
          {userBatches.length} {userBatches.length === 1 ? "batch" : "batches"}
        {:else}
          No batches
        {/if}
      </span>
    </div>

    {#if userBatches.length > 0}
      <div class="pt-2 border-t border-zinc-800">
        <div class="flex justify-between items-center">
          <span class="text-xs text-zinc-500">Total Amount</span>
          <div class="text-right">
            {#if O.isSome(evmChain) && O.isSome(uOnEvmToken)}
              <TokenComponent
                chain={evmChain.value}
                denom={uOnEvmToken.value.denom}
                amount={TokenRawAmount.make(pipe(
                  totalWithdrawableAmount,
                  BigDecimal.normalize,
                  (normalized) =>
                    normalized.scale >= 0
                      ? normalized.value / (10n ** BigInt(normalized.scale))
                      : normalized.value * (10n ** BigInt(-normalized.scale)),
                ))}
                showWrapping={false}
                showSymbol={true}
                showIcon={true}
                maxDecimals={4}
              />
              <div class="text-xs text-zinc-500 mt-0.5">Ready to claim</div>
            {:else}
              <span class="text-sm font-medium text-zinc-200">—</span>
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <div class="text-center py-4">
        <svg
          class="w-10 h-10 mx-auto text-zinc-700 mb-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <div class="text-sm font-medium text-zinc-400">
          {#if O.isNone(WalletStore.evmAddress)}
            Connect wallet to view
          {:else}
            No withdrawals available
          {/if}
        </div>
        <div class="text-xs text-zinc-500 mt-1">
          Unbonded tokens appear here after 27 days
        </div>
      </div>
    {/if}
  </div>

  <!-- Transaction Info (only show if there are withdrawals) -->
  {#if userBatches.length > 0}
    <div class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3 space-y-3">
      <div class="flex justify-between items-center">
        <span class="text-xs text-zinc-500">You'll Receive</span>
        <div class="text-right">
          {#if O.isSome(evmChain) && O.isSome(uOnEvmToken)}
            <TokenComponent
              chain={evmChain.value}
              denom={uOnEvmToken.value.denom}
              amount={TokenRawAmount.make(pipe(
                totalWithdrawableAmount,
                BigDecimal.normalize,
                (normalized) =>
                  normalized.scale >= 0
                    ? normalized.value / (10n ** BigInt(normalized.scale))
                    : normalized.value * (10n ** BigInt(-normalized.scale)),
              ))}
              showWrapping={false}
              showSymbol={true}
              showIcon={true}
              maxDecimals={4}
            />
            <div class="text-xs text-zinc-500 mt-0.5">To your wallet</div>
          {:else}
            <span class="text-sm font-medium text-zinc-200">—</span>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Status Display -->
  <StatusDisplay
    state={withdrawalState}
    type="withdrawal"
  />

  <!-- Action Button -->
  <Button
    variant={isError ? "secondary" : "primary"}
    disabled={isLoading || BigDecimal.isZero(withdrawableAmount)}
    onclick={() => handleWithdraw()}
  >
    {#if isLoading}
      <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2">
      </div>
    {/if}
    {
      Match.value(withdrawalState).pipe(
        Match.when(WithdrawalState.$is("Ready"), () =>
          pipe(
            WalletStore.evmAddress,
            O.match({
              onNone: () => "Connect Wallet",
              onSome: () =>
                userBatches.length === 0
                  ? "No Withdrawals Available"
                  : "Withdraw",
            }),
          )),
        Match.when(WithdrawalState.$is("Loading"), () => "Processing..."),
        Match.when(WithdrawalState.$is("Success"), () => "Success!"),
        Match.when(WithdrawalState.$is("Error"), () => "Try Again"),
        Match.exhaustive,
      )
    }
  </Button>
</div>
