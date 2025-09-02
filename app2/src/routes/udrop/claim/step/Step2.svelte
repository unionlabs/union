<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import { evmWalletsInformation, sepoliaStore } from "$lib/wallet/evm/config.svelte"
import { Option } from "effect"
import StepLayout from "../StepLayout.svelte"
interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

const connectedEvmWallet = $derived(
  sepoliaStore.connectionStatus === "connected" && sepoliaStore.connectedWallet
    && evmWalletsInformation
    ? Option.fromNullable(
      evmWalletsInformation.find((wallet: { id: string }) =>
        wallet.id === sepoliaStore.connectedWallet
      ),
    )
    : Option.none(),
)

const isEvmConnected = $derived(sepoliaStore.connectionStatus === "connected")

const evmAddress = $derived(
  Option.map(wallets.evmAddress, (evmDisplay) => evmDisplay.address),
)

const claimBeneficiary = $derived(
  Option.flatMap(
    dashboard.airdrop,
    (store) => Option.map(store.claim, (claim) => claim.beneficiary),
  ),
)

const hasWalletMismatch = $derived(
  Option.match(evmAddress, {
    onNone: () => false,
    onSome: (connectedAddr) =>
      Option.match(claimBeneficiary, {
        onNone: () => false,
        onSome: (beneficiaryAddr) => connectedAddr.toLowerCase() !== beneficiaryAddr.toLowerCase(),
      }),
  }),
)

const hasWalletConnected = $derived(isEvmConnected)

function connectWallet() {
  uiStore.openWalletModal()
}

function handleContinue() {
  if (hasWalletConnected) {
    onNext()
  }
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            Connect Ethereum Wallet
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Connect your Ethereum wallet to claim your U tokens on Ethereum.
          </p>
        </div>
      </div>

      <div class="space-y-3">
        <!-- Wallet Mismatch Warning -->
        {#if hasWalletMismatch}
          <div class="bg-orange-500/10 border border-orange-500/20 rounded-lg p-4">
            <div class="flex items-start gap-3">
              <div class="w-8 h-8 bg-orange-500/20 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                <svg
                  class="w-4 h-4 text-orange-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-orange-400 mb-2">Wallet Mismatch Detected</div>
                <div class="text-xs text-zinc-400 space-y-2">
                  <div>
                    <span class="text-zinc-500">Connected wallet:</span>
                    <span class="font-mono text-zinc-300">
                      {
                        Option.match(evmAddress, {
                          onNone: () => "None",
                          onSome: (addr) =>
                            `${addr.slice(0, 6)}...${addr.slice(-4)}`,
                        })
                      }
                    </span>
                  </div>
                  <div>
                    <span class="text-zinc-500">Claim beneficiary:</span>
                    <span class="font-mono text-zinc-300">
                      {
                        Option.match(claimBeneficiary, {
                          onNone: () => "None",
                          onSome: (addr) =>
                            `${addr.slice(0, 6)}...${addr.slice(-4)}`,
                        })
                      }
                    </span>
                  </div>
                  <div class="text-xs text-orange-400/80 mt-2">
                    Please connect the wallet that matches your claim's beneficiary address to
                    proceed.
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/if}

        <!-- Wallet Connection Status - using check flow design -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800 cursor-pointer hover:bg-zinc-900/50 transition-colors"
          onclick={connectWallet}
        >
          {#if isEvmConnected}
            <!-- Connected wallet display -->
            <div class="flex gap-3">
              <!-- Wallet Icon -->
              {#if Option.isSome(connectedEvmWallet)}
                <img
                  src={connectedEvmWallet.value.icon}
                  alt="{connectedEvmWallet.value.name ?? 'EVM Wallet'} icon"
                  class="size-8 rounded-lg bg-white p-1 flex-shrink-0"
                />
              {:else}
                <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                  <svg
                    class="w-4 h-4 text-zinc-400"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M12 1.75L5.75 12.25L12 16L18.25 12.25L12 1.75ZM5.75 13.5L12 22.25L18.25 13.5L12 17.25L5.75 13.5Z" />
                  </svg>
                </div>
              {/if}

              <!-- Wallet name and address -->
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium text-white">
                  {#if Option.isSome(connectedEvmWallet)}
                    {connectedEvmWallet.value.name}
                  {:else}
                    Ethereum Wallet
                  {/if}
                </div>
                <div class="text-xs text-zinc-400 font-mono">
                  {
                    Option.match(evmAddress, {
                      onNone: () => "No address",
                      onSome: (addr) => `${addr.slice(0, 6)}...${addr.slice(-4)}`,
                    })
                  }
                </div>
              </div>

              <!-- Status indicator -->
              <div class="flex items-center">
                {#if hasWalletMismatch}
                  <div class="w-4 h-4 bg-orange-500/20 border border-orange-500/40 rounded-full flex items-center justify-center">
                    <svg
                      class="w-2.5 h-2.5 text-orange-400"
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
                  </div>
                {:else}
                  <div class="w-4 h-4 bg-accent/20 border border-accent/40 rounded-full flex items-center justify-center">
                    <svg
                      class="w-2.5 h-2.5 text-accent"
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
                  </div>
                {/if}
              </div>
            </div>
          {:else}
            <!-- No wallet connected -->
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                <svg
                  class="w-4 h-4 text-zinc-400"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M12 1.75L5.75 12.25L12 16L18.25 12.25L12 1.75ZM5.75 13.5L12 22.25L18.25 13.5L12 17.25L5.75 13.5Z" />
                </svg>
              </div>
              <div>
                <div class="text-sm font-medium text-zinc-400">No wallet connected</div>
                <div class="text-xs text-zinc-500">Connect your Ethereum wallet to continue</div>
              </div>
            </div>
          {/if}
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-3">
          <Button
            variant="secondary"
            class="flex flex-1 items-center justify-center gap-3"
            onclick={connectWallet}
          >
            {hasWalletConnected ? "Change Wallet" : "Connect Ethereum Wallet"}
          </Button>
          <Button
            variant="primary"
            class="flex flex-1 items-center justify-center gap-3"
            disabled={!hasWalletConnected || hasWalletMismatch}
            onclick={handleContinue}
          >
            Continue
          </Button>
        </div>

        <!-- Back button -->
        {#if onBack}
          <Button
            variant="secondary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onBack}
          >
            ← Back
          </Button>
        {/if}
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">Connect Ethereum Wallet</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Connect your Ethereum wallet to claim your U tokens on Ethereum.
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Ethereum Logo as full background -->
        <div
          class="w-full h-full flex items-center justify-center"
          style="background-color: #0D2024;"
        >
          <svg
            class="w-32 h-32 {isEvmConnected ? 'text-accent' : 'text-rose-400'} opacity-80"
            fill="currentColor"
            viewBox="0 0 24 24"
          >
            <path d="M12 1.75L5.75 12.25L12 16L18.25 12.25L12 1.75ZM5.75 13.5L12 22.25L18.25 13.5L12 17.25L5.75 13.5Z" />
          </svg>
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
