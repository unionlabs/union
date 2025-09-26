<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import StatusDisplay from "./StatusDisplay.svelte"
import TokenBalanceRow from "./TokenBalanceRow.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm/chain"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import { safeOpts } from "$lib/transfer/shared/services/handlers/safe"
import { uiStore } from "$lib/stores/ui.svelte"
import { predictProxy } from "$lib/stake/instantiate2"
import { 
  DESTINATION_CHANNEL_ID, 
  SOURCE_CHANNEL_ID,
  ETHEREUM_CHAIN_ID,
  UNION_CHAIN_ID,
  UCS03_EVM_ADDRESS,
  UCS03_ZKGM,
  UCS03_MINTER_ON_UNION,
} from "$lib/stake/config"
import { Batch, Call, Token, TokenOrder, Ucs03, Ucs05, Utils, ZkgmClient, ZkgmClientRequest } from "@unionlabs/sdk"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { Evm, EvmZkgmClient, Safe } from "@unionlabs/sdk-evm"
import { EU_LST, EU_ERC20, EU_SOLVER_ON_ETH_METADATA } from "@unionlabs/sdk/Constants"
import type { Chain, Token as TokenType } from "@unionlabs/sdk/schema"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import { BigDecimal, pipe, Data, Effect, Schema, Match, Layer } from "effect"
import * as O from "effect/Option"
import { custom } from "viem"
import { mainnet } from "viem/chains"

interface Props {
  evmChain: O.Option<Chain>
  eUOnEvmToken: O.Option<TokenType>
  redemptionRate: O.Option<BigDecimal.BigDecimal>
  proxyEuDust: O.Option<BigDecimal.BigDecimal>
  proxyAddress: O.Option<string>
}

let {
  evmChain,
  eUOnEvmToken,
  redemptionRate,
  proxyEuDust,
  proxyAddress,
}: Props = $props()

const JsonFromBase64 = Schema.compose(
  Schema.StringFromBase64,
  Schema.parseJson(),
)

const UCS03_EVM = Ucs05.EvmDisplay.make({ address: UCS03_EVM_ADDRESS })

// Dust withdrawal state machine
type DustWithdrawState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  PreparingTransaction: {}
  ConfirmingWithdrawal: {}
  WithdrawalSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  WaitingForIndexer: { txHash: string }
  Success: { txHash: string }
  Error: { message: string }
}>

const DustWithdrawState = Data.taggedEnum<DustWithdrawState>()

let dustWithdrawState = $state<DustWithdrawState>(DustWithdrawState.Ready())
let shouldWithdrawDust = $state<boolean>(false)

const isWithdrawing = $derived(
  !DustWithdrawState.$is("Ready")(dustWithdrawState)
    && !DustWithdrawState.$is("Success")(dustWithdrawState)
    && !DustWithdrawState.$is("Error")(dustWithdrawState),
)
const isSuccess = $derived(DustWithdrawState.$is("Success")(dustWithdrawState))
const isError = $derived(DustWithdrawState.$is("Error")(dustWithdrawState))

const hasDust = $derived(
  O.isSome(proxyEuDust) && BigDecimal.greaterThan(proxyEuDust.value, BigDecimal.make(0n, 18))
)
const isButtonDisabled = $derived(!hasDust || isWithdrawing || isSuccess)

// Execute dust withdrawal
const executeDustWithdrawal = (
  sender: Ucs05.EvmDisplay,
  dustAmount: BigDecimal.BigDecimal,
  proxyAddr: string,
) =>
  Effect.gen(function*() {
    const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)
    const unionChain = yield* ChainRegistry.byUniversalId(UNION_CHAIN_ID)
    
    // For dust withdrawal, we need to transfer eU from proxy to user's wallet on Union
    // then send it to Ethereum
    const proxy = yield* predictProxy({
      path: 0n,
      channel: DESTINATION_CHANNEL_ID,
      sender,
    })

    // Convert dust amount to raw value
    const dustAmountRaw = Utils.toRawAmount(dustAmount)

    // Give allowance to UCS03_MINTER to spend the eU dust
    const increaseAllowanceCall = yield* pipe(
      {
        increase_allowance: {
          spender: UCS03_MINTER_ON_UNION.address,
          amount: dustAmountRaw.toString(),
        },
      } as const,
      Schema.encode(JsonFromBase64),
      Effect.map((msg) => ({
        wasm: {
          execute: {
            contract_addr: EU_LST.address,
            msg,
            funds: [],
          },
        },
      })),
    )

    const salt = yield* Utils.generateSalt("cosmos")
    const timeout_timestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()

    // Send the eU back to Ethereum
    const sendCall = yield* pipe(
      TokenOrder.make({
        source: unionChain,
        destination: ethereumChain,
        sender: proxy,
        receiver: sender,
        baseToken: Token.Cw20.make({ address: EU_LST.address }),
        baseAmount: dustAmountRaw,
        quoteToken: EU_ERC20,
        quoteAmount: dustAmountRaw,
        kind: "solve",
        metadata: EU_SOLVER_ON_ETH_METADATA,
        version: 2,
      }),
      Effect.flatMap(TokenOrder.encodeV2),
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
            funds: [],
          },
        },
      })),
    )

    const calls = yield* pipe(
      [increaseAllowanceCall, sendCall],
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

    const batchInstruction = Batch.make([calls])

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

// Run dust withdrawal when triggered
runPromiseExit$(() =>
  shouldWithdrawDust
    ? Effect.gen(function*() {
      if (!hasDust || O.isNone(proxyEuDust) || O.isNone(proxyAddress)) {
        dustWithdrawState = DustWithdrawState.Error({
          message: "No dust available to withdraw",
        })
        shouldWithdrawDust = false
        return yield* Effect.fail(new Error("No dust available"))
      }

      const sender = O.getOrNull(WalletStore.evmAddress)
      if (!sender) {
        dustWithdrawState = DustWithdrawState.Error({
          message: "Wallet not connected",
        })
        shouldWithdrawDust = false
        return yield* Effect.fail(new Error("Wallet not connected"))
      }

      dustWithdrawState = DustWithdrawState.SwitchingChain()

      const VIEM_CHAIN = mainnet
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

      dustWithdrawState = DustWithdrawState.PreparingTransaction()

      dustWithdrawState = DustWithdrawState.ConfirmingWithdrawal()

      const { txHash } = yield* executeDustWithdrawal(
        sender, 
        proxyEuDust.value,
        proxyAddress.value
      ).pipe(
        Effect.provide(EvmZkgmClient.layerWithoutWallet),
        Effect.provide(walletClient),
        Effect.provide(publicClient),
        Effect.provide(ChainRegistry.Default),
        Effect.provide(maybeSafe),
      )

      dustWithdrawState = DustWithdrawState.WithdrawalSubmitted({ txHash })
      yield* Effect.sleep("500 millis")

      dustWithdrawState = DustWithdrawState.WaitingForConfirmation({ txHash })

      yield* pipe(
        Evm.waitForTransactionReceipt(txHash),
        Effect.provide(publicClient),
      )

      dustWithdrawState = DustWithdrawState.Success({ txHash })
      shouldWithdrawDust = false
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          const errorDetails = extractErrorDetails(error) as any
          const fullError = errorDetails?.cause?.cause?.shortMessage
            || errorDetails?.cause?.message
            || errorDetails?.message
            || JSON.stringify(errorDetails)
          const shortMessage = String(fullError).split(".")[0]

          dustWithdrawState = DustWithdrawState.Error({ message: shortMessage })
          shouldWithdrawDust = false
          return yield* Effect.void
        })
      ),
    )
    : Effect.void
)

$effect(() => {
  if (!hasDust && DustWithdrawState.$is("Success")(dustWithdrawState)) {
    dustWithdrawState = DustWithdrawState.Ready()
  }
})

const handleDustWithdraw = () => {
  if (isWithdrawing || !hasDust) {
    return
  }

  Match.value(WalletStore.evmAddress).pipe(
    Match.when(O.isNone, () => uiStore.openWalletModal()),
    Match.orElse(() => {
      shouldWithdrawDust = true
    }),
  )
}
</script>

<div class="flex flex-col gap-3 flex-1">
  {#if hasDust}
    <!-- Dust Balance -->
    <TokenBalanceRow
      chain={evmChain}
      token={eUOnEvmToken}
      balance={O.map(proxyEuDust, dust => Utils.toRawAmount(dust))}
      symbol="eU"
      title="Proxy Dust (eU)"
      subtitle={pipe(
        O.all([proxyEuDust, redemptionRate]),
        O.map(([dust, rate]) => {
          const valueInU = BigDecimal.multiply(dust, rate)
          return `≈ ${Utils.formatBigDecimal(BigDecimal.round({ mode: "from-zero", scale: 2 })(valueInU))} U`
        }),
        O.getOrElse(() => 
          O.isSome(proxyEuDust) && O.isSome(redemptionRate) ? "loading" : ""
        )
      )}
    />

    <!-- Status Display -->
    <StatusDisplay
      state={dustWithdrawState}
      type="dust"
      class="mb-4"
    />

    <div class="flex-1"></div>
  {:else}
    <!-- No dust message using TokenBalanceRow for consistency -->
    <TokenBalanceRow
      chain={evmChain}
      token={eUOnEvmToken}
      balance={O.some(0n)}
      symbol="eU"
      title="Proxy Dust"
      subtitle="No dust available"
    />

    <div class="flex-1 flex items-center justify-center">
      <p class="text-xs text-zinc-600">
        Dust appears here when you stake with slippage protection
      </p>
    </div>
  {/if}

  <!-- Recovery Button -->
  <Button
    variant={isError || !hasDust ? "secondary" : "primary"}
    disabled={isButtonDisabled}
    onclick={handleDustWithdraw}
    class="w-full"
  >
    {#if isWithdrawing}
      <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin mr-2"></div>
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
      Match.value(dustWithdrawState).pipe(
        Match.when(DustWithdrawState.$is("Ready"), () =>
          !hasDust
            ? "No Dust Available"
            : O.isNone(WalletStore.evmAddress)
              ? "Connect Wallet"
              : "Recover Dust to Wallet"),
        Match.when(DustWithdrawState.$is("SwitchingChain"), () => "Switching..."),
        Match.when(DustWithdrawState.$is("CheckingAllowance"), () => "Checking..."),
        Match.when(DustWithdrawState.$is("ApprovingAllowance"), () => "Confirm in Wallet"),
        Match.when(DustWithdrawState.$is("AllowanceSubmitted"), () => "Submitted"),
        Match.when(DustWithdrawState.$is("WaitingForAllowanceConfirmation"), () => "Confirming..."),
        Match.when(DustWithdrawState.$is("AllowanceApproved"), () => "Approved ✓"),
        Match.when(DustWithdrawState.$is("PreparingTransaction"), () => "Preparing..."),
        Match.when(DustWithdrawState.$is("ConfirmingWithdrawal"), () => "Confirm in Wallet"),
        Match.when(DustWithdrawState.$is("WithdrawalSubmitted"), () => "Submitted"),
        Match.when(DustWithdrawState.$is("WaitingForConfirmation"), () => "Confirming..."),
        Match.when(DustWithdrawState.$is("WaitingForIndexer"), () => "Indexing..."),
        Match.when(DustWithdrawState.$is("Success"), () => "Recovery Pending"),
        Match.when(DustWithdrawState.$is("Error"), () =>
          hasDust ? "Try Again" : "No Dust Available"),
        Match.exhaustive,
      )
    }
  </Button>
</div>
