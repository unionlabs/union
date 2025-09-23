<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Label from "$lib/components/ui/Label.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { EU_STAKING_HUB } from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import { 
  Batch,
  Call,
  Token,
  TokenOrder,
  Ucs03,
  Ucs05, 
  Utils,
  ZkgmClient,
  ZkgmClientRequest,
} from "@unionlabs/sdk"
import { Evm, EvmZkgmClient, Safe } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { U_ERC20, U_SOLVER_ON_ETH_METADATA, U_BANK } from "@unionlabs/sdk/Constants"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { instantiate2 } from "$lib/stake/instantiate2"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import { custom } from "viem"
import { sepolia } from "viem/chains"
import {
  BigDecimal,
  Data,
  Effect,
  Layer,
  Match,
  pipe,
  Schema,
} from "effect"
import * as A from "effect/Array"
import * as O from "effect/Option"

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<TokenType>
  onWithdrawSuccess?: () => void
}

let { evmChain, uOnEvmToken, onWithdrawSuccess }: Props = $props()

type WithdrawalState = Data.TaggedEnum<{
  Ready: {}
  Loading: {}
  Success: { message: string }
  Error: { message: string }
}>

const WithdrawalState = Data.taggedEnum<WithdrawalState>()

let withdrawalState = $state<WithdrawalState>(WithdrawalState.Ready())
let selectedBatchIds = $state<Array<string>>([])

const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.11155111")
const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")
const SOURCE_CHANNEL_ID = ChannelId.make(3)
const DESTINATION_CHANNEL_ID = ChannelId.make(3)
const UCS03_EVM = Ucs05.EvmDisplay.make({
  address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
})
const UCS03_ZKGM = Ucs05.CosmosDisplay.make({
  address: "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
})

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
      onSome: (address) => Effect.gen(function*() {
        const receiver = yield* instantiate2({
          path: 0n,
          channel: DESTINATION_CHANNEL_ID,
          sender: address,
        })
        
        // Query unstake requests
        const unstakeRequests = yield* pipe(
          Cosmos.queryContract(EU_STAKING_HUB, {
            unstake_requests_by_staker: { staker: receiver.address },
          }),
          Effect.flatMap(Schema.decodeUnknown(Schema.Array(Schema.Struct({
            batch_id: Schema.String,
            staker: Schema.String,
            amount: Schema.String,
          })))),
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
          Effect.flatMap(Schema.decodeUnknown(Schema.Struct({
            batches: Schema.Array(Schema.Struct({
              batch_id: Schema.String,
              batch: Schema.Union(
                Schema.Struct({
                  status: Schema.Literal("pending"),
                  total_lst_to_burn: Schema.String,
                  unstake_requests_count: Schema.String,
                }),
                Schema.Struct({
                  status: Schema.Literal("submitted"),
                  total_lst_to_burn: Schema.String,
                  unstake_requests_count: Schema.String,
                  receive_time: Schema.String,
                  expected_native_unstaked: Schema.String,
                }),
                Schema.Struct({
                  status: Schema.Literal("received"),
                  total_lst_to_burn: Schema.String,
                  unstake_requests_count: Schema.String,
                  received_native_unstaked: Schema.String,
                }),
              ),
            })),
          }))),
        )
        
        const processRequest = (request: any) => Effect.gen(function*() {
          const batch = batchesResponse.batches.find(b => b.batch_id === request.batch_id)
          if (!batch || batch.batch.status !== "received") {
            return O.none<{ batchId: string; withdrawableAmount: BigDecimal.BigDecimal }>()
          }

          return yield* pipe(
            O.all([
              BigDecimal.fromString(request.amount),
              BigDecimal.fromString(batch.batch.total_lst_to_burn),
              BigDecimal.fromString(batch.batch.received_native_unstaked)
            ]),
            O.match({
              onNone: () => Effect.succeed(O.none<{ batchId: string; withdrawableAmount: BigDecimal.BigDecimal }>()),
              onSome: ([userAmount, totalLstToBurn, receivedNativeUnstaked]) => Effect.gen(function*() {
                const userShare = BigDecimal.isZero(totalLstToBurn)
                  ? BigDecimal.fromBigInt(0n)
                  : yield* BigDecimal.divide(
                      BigDecimal.multiply(userAmount, receivedNativeUnstaked),
                      totalLstToBurn
                    ).pipe(
                      Effect.mapError(() => new Error("Division by zero in user share calculation"))
                    )
                  
                return O.some({
                  batchId: request.batch_id,
                  withdrawableAmount: userShare,
                })
              })
            })
          )
        })

        const userBatchOptions = yield* Effect.all(
          unstakeRequests.map(processRequest),
          { concurrency: "unbounded" }
        )
        
        const userBatches = pipe(
          userBatchOptions,
          A.filterMap((opt) => opt)
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
    : []
)

// Auto-select all batches when they load
$effect(() => {
  if (userBatches.length > 0 && selectedBatchIds.length === 0) {
    selectedBatchIds = userBatches.map(b => b.batchId)
  }
})

// Total withdrawable amount (always shows all available)
const totalWithdrawableAmount = $derived(
  userBatches.reduce(
    (sum, batch) => BigDecimal.sum(sum, batch.withdrawableAmount),
    BigDecimal.fromBigInt(0n)
  )
)

// Selected batches and selected amount
const selectedBatches = $derived(userBatches.filter(batch => selectedBatchIds.includes(batch.batchId)))
const selectedAmount = $derived(
  selectedBatches.reduce(
    (sum, batch) => BigDecimal.sum(sum, batch.withdrawableAmount),
    BigDecimal.fromBigInt(0n)
  )
)

// Toggle functions
const toggleBatchSelection = (batchId: string) => {
  selectedBatchIds = selectedBatchIds.includes(batchId)
    ? selectedBatchIds.filter(id => id !== batchId)
    : [...selectedBatchIds, batchId]
}

const toggleAllBatches = () => {
  selectedBatchIds = selectedBatchIds.length === userBatches.length
    ? []
    : userBatches.map(b => b.batchId)
}

// Execute withdrawal transaction following Bond/Unbond pattern
const executeWithdraw = (sender: Ucs05.EvmDisplay, batches: Array<{ batchId: string; withdrawableAmount: BigDecimal.BigDecimal }>) =>
  Effect.gen(function*() {
    const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
    const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)
    const receiver = yield* instantiate2({
      path: 0n,
      channel: DESTINATION_CHANNEL_ID,
      sender,
    })

    // Create withdraw calls for each selected batch
    const withdrawCalls = yield* Effect.all(
      batches.map(batch =>
        pipe(
          {
            withdraw: {
              withdraw_to_address: receiver.address,
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
      )
    )

    const totalAmountBigDecimal = batches.reduce(
      (sum, batch) => BigDecimal.sum(sum, batch.withdrawableAmount),
      BigDecimal.fromBigInt(0n)
    )
    
    const totalAmount = pipe(
      totalAmountBigDecimal,
      BigDecimal.normalize,
      (normalized) => normalized.scale >= 0
        ? normalized.value / (10n ** BigInt(normalized.scale))
        : normalized.value * (10n ** BigInt(-normalized.scale))
    )
    
    const tokenOrder = yield* TokenOrder.make({
      source: unionChain,
      destination: ethereumChain,
      sender: Ucs05.CosmosDisplay.make({
        address: "union1ylfrhs2y5zdj2394m6fxgpzrjav7le3z07jffq",
      }),
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

    const allCalls = [...withdrawCalls, sendCall]

    const calls = yield* pipe(
      allCalls,
      Schema.decode(HexFromJson),
      Effect.map((contractCalldata) =>
        Call.make({
          sender,
          eureka: false,
          contractAddress: receiver,
          contractCalldata,
        })
      ),
    )

    const batchInstruction = Batch.make([
      calls
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
      const senderOpt = WalletStore.evmAddress
      if (O.isNone(senderOpt) || BigDecimal.isZero(selectedAmount) || O.isNone(evmChain)) {
        withdrawalState = WithdrawalState.Error({
          message: "Missing required data: wallet address, selected batches, or chain",
        })
        shouldWithdraw = false
        return yield* Effect.fail(new Error("Missing required data"))
      }

      const sender = senderOpt.value
      const batches = selectedBatches
      const chain = evmChain.value

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
        Effect.provide(maybeSafe),
        Effect.provide(EvmZkgmClient.layerWithoutWallet),
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(ChainRegistry.Default),
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
          const errorObj = error as any
          const fullError = errorObj?.cause?.cause?.shortMessage
            || errorObj?.cause?.message
            || errorObj?.message
            || JSON.stringify(error)
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
  if (isLoading || BigDecimal.isZero(selectedAmount)) return
  
  Match.value(WalletStore.evmAddress).pipe(
    Match.when(O.isNone, () => uiStore.openWalletModal()),
    Match.orElse(() => {
      shouldWithdraw = true
    })
  )
}
</script>

<div class="flex grow flex-col gap-4">
  <div>
    <Label caseSensitive>WITHDRAWABLE BALANCE</Label>
    {#if O.isNone(WalletStore.evmAddress)}
      <div>—</div>
    {:else if userBatches.length > 0 && O.isSome(evmChain) && O.isSome(uOnEvmToken)}
      <TokenComponent
        chain={evmChain.value}
        denom={uOnEvmToken.value.denom}
        amount={TokenRawAmount.make(pipe(
          totalWithdrawableAmount,
          BigDecimal.normalize,
          (normalized) => normalized.scale >= 0
            ? normalized.value / (10n ** BigInt(normalized.scale))
            : normalized.value * (10n ** BigInt(-normalized.scale))
        ))}
        showWrapping={false}
        showSymbol={true}
        showIcon={true}
      />
    {:else}
      <div class="text-zinc-400">No tokens available for withdrawal</div>
    {/if}
  </div>

  {#if O.isSome(WalletStore.evmAddress) && userBatches.length > 0}
    <div class="space-y-4">
      <div class="flex justify-between items-center">
        <h3 class="text-sm font-medium text-zinc-300">Ready to Withdraw</h3>
        
        {#if userBatches.length > 1}
          <button
            class="text-xs text-accent hover:text-accent/80 transition-colors"
            onclick={toggleAllBatches}
          >
            {selectedBatchIds.length === userBatches.length ? 'Deselect All' : 'Select All'}
          </button>
        {/if}
      </div>
      
      <div class="space-y-2">
        {#each userBatches as batch}
          <button
            class="bg-accent/10 border border-accent/20 rounded-lg p-3 w-full transition-all hover:bg-accent/15"
            onclick={() => toggleBatchSelection(batch.batchId)}
          >
            <div class="flex justify-between items-center">
              <div class="flex items-center gap-3">
                <div class="flex items-center justify-center w-4 h-4 border-2 border-accent rounded transition-colors {selectedBatchIds.includes(batch.batchId) ? 'bg-accent' : 'bg-transparent'}">
                  {#if selectedBatchIds.includes(batch.batchId)}
                    <svg class="w-2.5 h-2.5 text-zinc-900" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  {/if}
                </div>
                <span class="text-sm text-accent font-medium">Batch #{batch.batchId}</span>
              </div>
              
              <div class="text-sm font-medium text-white">
                {#if O.isSome(evmChain) && O.isSome(uOnEvmToken)}
                  <TokenComponent
                    chain={evmChain.value}
                    denom={uOnEvmToken.value.denom}
                    amount={TokenRawAmount.make(pipe(
                      batch.withdrawableAmount,
                      BigDecimal.normalize,
                      (normalized) => normalized.scale >= 0
                        ? normalized.value / (10n ** BigInt(normalized.scale))
                        : normalized.value * (10n ** BigInt(-normalized.scale))
                    ))}
                    showWrapping={false}
                    showSymbol={true}
                    showIcon={false}
                    maxDecimals={6}
                  />
                {:else}
                  {pipe(
                    batch.withdrawableAmount,
                    BigDecimal.unsafeDivide(BigDecimal.make(10n ** 18n, 0)),
                    Utils.formatBigDecimal,
                  )} U
                {/if}
              </div>
            </div>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
    <div class="flex items-center gap-3">
      <div class="size-8 rounded-lg {isError ? 'bg-red-500/20 border-red-500/40' : isSuccess ? 'bg-accent/20 border-accent/40' : isReady ? 'bg-zinc-700/20 border-zinc-600/40' : 'bg-accent/20 border-accent/40'} flex items-center justify-center flex-shrink-0">
        {#if isReady}
          <svg
            class="w-4 h-4 text-zinc-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1"
            />
          </svg>
        {:else if isLoading}
          <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
          </div>
        {:else if isSuccess}
          <svg
            class="w-4 h-4 text-accent"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            />
          </svg>
        {:else if isError}
          <svg
            class="w-4 h-4 text-red-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01"
            />
          </svg>
        {/if}
      </div>
      <div class="flex-1">
        <div class="text-sm font-medium text-white">
          {
            Match.value(withdrawalState).pipe(
              Match.when(WithdrawalState.$is("Ready"), () => {
                if (O.isNone(WalletStore.evmAddress)) return "Connect your wallet to view withdrawals"
                if (userBatches.length === 0) return "No tokens available for withdrawal"
                if (selectedBatches.length === 0) return "Select which batches to withdraw from"
                
                const totalFormatted = pipe(
                  selectedAmount,
                  BigDecimal.unsafeDivide(BigDecimal.make(10n ** 18n, 0)),
                  Utils.formatBigDecimal,
                )
                
                return `Withdraw ${totalFormatted} U`
              }),
              Match.when(WithdrawalState.$is("Loading"), () => "Processing withdrawal..."),
              Match.when(WithdrawalState.$is("Success"), ({ message }) => message),
              Match.when(WithdrawalState.$is("Error"), () => "Withdrawal failed"),
              Match.exhaustive,
            )
          }
        </div>
        <div class="text-xs {isReady ? 'text-zinc-400' : isError ? 'text-red-400' : isSuccess ? 'text-accent' : 'text-accent'} mt-1">
          {
            Match.value(withdrawalState).pipe(
              Match.when(WithdrawalState.$is("Ready"), () => {
                if (O.isNone(WalletStore.evmAddress)) {
                  return "Connect wallet to see your withdrawable batches"
                }
                if (userBatches.length === 0) {
                  return "Unstaked tokens will appear here after the 27-day unbonding period"
                }
                if (selectedBatches.length === 0) {
                  return "Select which batches to withdraw from"
                }
                
                return selectedBatches.length > 1
                  ? `From ${selectedBatches.length} selected batches`
                  : "From 1 selected batch"
              }),
              Match.when(WithdrawalState.$is("Loading"), () => "Please wait while we process your withdrawal..."),
              Match.when(WithdrawalState.$is("Success"), () => "Withdrawal completed successfully"),
              Match.when(WithdrawalState.$is("Error"), ({ message }) => message),
              Match.exhaustive,
            )
          }
        </div>
      </div>
    </div>
  </div>

  <div>
    <Button
      class="w-full relative z-30"
      variant={isError ? "secondary" : "primary"}
      disabled={isLoading || BigDecimal.isZero(selectedAmount)}
      onclick={() => handleWithdraw()}
    >
      {#if isLoading}
        <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2">
        </div>
      {:else if isSuccess}
        <svg
          class="w-4 h-4 text-current mr-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 13l4 4L19 7"
          />
        </svg>
      {/if}
      {
        Match.value(withdrawalState).pipe(
          Match.when(WithdrawalState.$is("Ready"), () => {
            if (O.isNone(WalletStore.evmAddress)) return "Connect Wallet"
            if (selectedBatches.length === 0) return "Select batches to withdraw"
            
            if (selectedBatches.length === userBatches.length && userBatches.length > 1) {
              return "Withdraw All"
            }
            
            return selectedBatches.length > 1 
              ? `Withdraw ${selectedBatches.length} Batches`
              : "Withdraw"
          }),
          Match.when(WithdrawalState.$is("Loading"), () => "Withdrawing..."),
          Match.when(WithdrawalState.$is("Success"), () => "Withdraw Again"),
          Match.when(WithdrawalState.$is("Error"), () => "Try Again"),
          Match.exhaustive,
        )
      }
    </Button>
  </div>
</div>