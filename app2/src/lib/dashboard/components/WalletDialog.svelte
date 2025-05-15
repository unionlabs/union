<script lang="ts">
import Modal from "$lib/components/ui/Modal.svelte";
import { dashboard } from "$lib/dashboard/stores/user.svelte";
import { Option, Effect, Match } from "effect"; // For handling Option type from WalletStore
import type { Chain } from "../stores/wallets.svelte"; // Import Chain type
import { evmWalletsInformation, sepoliaStore } from "$lib/wallet/evm/config.svelte";
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/config.svelte";
import { uiStore } from "$lib/stores/ui.svelte";
import Button from "$lib/components/ui/Button.svelte";
import { addEvmWallet, AddEvmWalletState } from "../services/add-evm-wallet";
import { sepolia } from "viem/chains";
import { getWagmiConnectorClient } from "$lib/services/evm/clients";
import { createViemWalletClient } from "@unionlabs/sdk/evm";
import { custom } from "viem";
import InsetError from "$lib/components/model/InsetError.svelte"

type PageState = 'select' | 'evm' | 'cosmos' | 'cosmos-verify' | 'completed';
let page: PageState = $state('select');
let isOpen = $state(false);
let selectedChains = $state<(string | null)[]>([]);
let message: string = $state("");
let hasError = $state(false);
let currentState: AddEvmWalletState | null = $state(null);
let showError = $state(false);
let error = $state<Option.Option<Error>>(Option.none());

const headerTitle = $derived(Match.value<PageState>(page).pipe(
  Match.when('select', () => "Add wallet"),
  Match.when('evm', () => "Verify EVM Wallet"),
  Match.when('cosmos', () => "Select Cosmos Chains"),
  Match.when('cosmos-verify', () => "Verify Cosmos Wallet"),
  Match.when('completed', () => "Wallet Added"),
  Match.orElse(() => "Add wallet")
));

const headerSubtitle = $derived(Match.value<PageState>(page).pipe(
  Match.when('select', () => "Select a wallet type to connect."),
  Match.when('evm', () => "Sign a message to verify ownership of your EVM wallet."),
  Match.when('cosmos', () => "Choose the Cosmos chains you'd like to add."),
  Match.when('cosmos-verify', () => "Sign a message to verify ownership of your Cosmos wallet."),
  Match.when('completed', () => "Your wallet has been successfully added to your dashboard."),
  Match.orElse(() => "Please follow the steps.")
));

const evmButtonText = $derived(sepoliaStore.connectionStatus === "connected" ? "EVM" : "EVM (not connected)");
const cosmosButtonText = $derived(cosmosStore.connectionStatus === "connected" ? "Cosmos" : "Cosmos (not connected)");

const connectedEvmWallet = $derived.by(() => {
  if (sepoliaStore.connectionStatus === "connected" && sepoliaStore.connectedWallet && evmWalletsInformation) {
    return evmWalletsInformation.find((wallet: {id: string}) => wallet.id === sepoliaStore.connectedWallet);
  }
  return null;
});

const connectedCosmosWallet = $derived.by(() => {
  if (cosmosStore.connectionStatus === "connected" && cosmosStore.connectedWallet && cosmosWalletsInformation) {
    return cosmosWalletsInformation.find((wallet: {id: string}) => wallet.id === cosmosStore.connectedWallet);
  }
  return null;
});

const toggleChain = (chainId: string | null) => {
  if (!chainId) return;
  if (selectedChains.includes(chainId)) {
    selectedChains = selectedChains.filter((id: string | null) => id !== chainId);
  } else {
    selectedChains = [...selectedChains, chainId];
  }
};

const cosmosChains: Chain[] = $derived(
  Option.match(dashboard.wallets, {
    onNone: () => [] as Chain[],
    onSome: (walletStore) => 
      Option.getOrElse(walletStore.chains, () => [] as Chain[]).filter(
        (chain: Chain) => chain.type === "cosmos" && chain.chain_id
      )
  })
);

function selectCosmosAndSetDefaults() {
  page = 'cosmos';
  selectedChains = [];
}

const connectButtonText = $derived(() => {
  if (cosmosChains.length > 0 && selectedChains.length === cosmosChains.length) {
    return "Connect (All selected)";
  } else if (selectedChains.length > 0) {
    return `Connect (${selectedChains.length} selected)`;
  }
  return "";
});

function connectWallet() {
  isOpen = false;
  uiStore.openWalletModal();
}

function add() {
  hasError = false;
  error = Option.none();
  currentState = null;
  
  const program = Effect.gen(function*() {
    const actionEffect = Match.value(page).pipe(
      Match.when('evm', () => Effect.gen(function*() {
        currentState = AddEvmWalletState.SwitchChain({chain: sepolia});
        const connectorClient = yield* getWagmiConnectorClient;
        const walletClient = yield* createViemWalletClient({
          account: connectorClient.account,
          chain: sepolia,
          transport: custom(connectorClient),
        });
        let evmResult: any;
        while (currentState !== null) {
          evmResult = yield* addEvmWallet(currentState as AddEvmWalletState, walletClient);
          if (Option.isSome(evmResult.error)) {
            message = evmResult.message;
            hasError = true;
            error = evmResult.error;
            currentState = null;
            return;
          }
          message = evmResult.message;
          if (Option.isSome(evmResult.nextState)) {
            currentState = evmResult.nextState.value;
          } else {
            currentState = null;
            page = 'completed';
          }
        }
      })),
      Match.when('cosmos-verify', () => 
        Effect.log("Cosmos wallet addition flow selected - To be implemented.")
      ),
      Match.orElse((currentPage) => 
        Effect.sync(() => { 
          console.log(`add() called on inactive page: ${currentPage}. No action taken.`);
        })
      )
    );
    yield* actionEffect;
  });

  Effect.runPromise(program);
}

function reset() {
  page = 'select';
  selectedChains = [];
  message = "";
  hasError = false;
  currentState = null;
  showError = false;
  error = Option.none();
}

// Dynamically determine button properties
const primaryButtonText = $derived(
  Match.value<[PageState, boolean]>([page, hasError]).pipe(
    // Error states take precedence (except for 'completed' page)
    Match.when(['select', true], () => "Retry"),
    Match.when(['evm', true], () => "Retry"),
    Match.when(['cosmos', true], () => "Retry"),
    Match.when(['cosmos-verify', true], () => "Retry"),
    // Normal states
    Match.when(['completed', false], () => "Done"),
    Match.when(['completed', true], () => "Done"),   // 'completed' page always shows "Done"
    Match.when(['evm', false], () => "Verify Wallet"),
    Match.when(['cosmos-verify', false], () => "Verify Wallet"),
    Match.when(['cosmos', false], () => "Continue"),
    Match.orElse(() => "Next") // Default for 'select' [false] or any other unhandled combo
  )
);

const onPrimaryButtonClick = $derived(
  Match.value<PageState>(page).pipe(
    Match.when('completed', () => () => { isOpen = false; reset(); }),
    Match.when('evm', () => add), 
    Match.when('cosmos', () => 
      selectedChains.length > 0 
        ? () => { page = 'cosmos-verify'; } 
        : () => {} 
    ),
    Match.orElse(() => () => {})
  )
);

const showCancelButton = $derived(
  Match.value<PageState>(page).pipe(
    Match.when('completed', () => false),
    Match.when('select', () => false),
    Match.orElse(() => true) // Show for evm, cosmos, cosmos-verify
  )
);

const cancelButtonText = $derived(
  Match.value<PageState>(page).pipe(
    Match.when('cosmos-verify', () => "Back"),
    Match.orElse(() => "Cancel")
  )
);

const onCancelClick = $derived(
  Match.value<PageState>(page).pipe(
    Match.when('cosmos-verify', () => () => { page = 'cosmos'; selectedChains = []; }),
    Match.orElse(() => () => { page = 'select'; reset(); })
  )
);

const showSecondaryCloseButton = $derived(
  Match.value<PageState>(page).pipe(
    Match.when('completed', () => true),
    Match.orElse(() => false)
  )
);

</script>

<button
  class="text-xs text-zinc-400 hover:text-white transition-colors border border-zinc-800 hover:border-zinc-700 px-2 py-0.5 rounded cursor-pointer"
  onclick={() => isOpen = true}
>
  Connect Wallet
</button>

<Modal {isOpen} onClose={() => {
  isOpen = false;
  reset();
}} class="p-0 min-h-[450px]">

  <section class="border-b border-zinc-200 dark:border-zinc-800 flex-none p-4">
    <h2 class="text-xl font-bold mb-2 text-white">{headerTitle}</h2>
    <p class="text-sm text-zinc-500 dark:text-zinc-400">{headerSubtitle}</p>
  </section>

  <div class="flex-1 min-h-0 flex flex-col">
    {#if page === 'select'}
      <div class="flex flex-col gap-4 p-4 flex-1 min-h-0">
        <button
          onclick={() => {
            if (sepoliaStore.connectionStatus === "connected") {
              page = 'evm';
            } else {
              connectWallet();
            }
          }}
          class="flex-1 p-4 bg-zinc-100 dark:bg-zinc-900 border border-zinc-300 dark:border-zinc-700 rounded-lg text-zinc-900 dark:text-zinc-50 hover:bg-zinc-200 dark:hover:bg-zinc-800 font-semibold text-lg transition-all flex items-start text-left"
        >
          {#if sepoliaStore.connectionStatus === 'connected'}
            <div class="w-full">
              <div class="text-xs text-zinc-400 dark:text-zinc-500 mb-1">Connected with</div>
              <div class="flex items-center gap-2">
                {#if connectedEvmWallet?.icon}
                  <img 
                    src={connectedEvmWallet.icon} 
                    alt="{connectedEvmWallet.name ?? 'EVM Wallet'} icon" 
                    class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1 flex-shrink-0"
                  />
                {/if}
                {#if connectedEvmWallet?.name}
                  <span class="font-semibold text-base text-zinc-900 dark:text-zinc-50 truncate">
                    {connectedEvmWallet.name}
                  </span>
                {/if}
              </div>
            </div>
          {:else}
            {#if connectedEvmWallet?.icon} <img 
                src={connectedEvmWallet.icon} 
                alt="{connectedEvmWallet.name ?? 'EVM Wallet'} icon" 
                class="w-7 h-7 rounded-md flex-shrink-0"
              />
            {/if}
            <span class="w-full text-center text-white">{evmButtonText}</span>
          {/if}
        </button>
        <button
          onclick={() => {
            if (cosmosStore.connectionStatus === "connected") {
              selectCosmosAndSetDefaults();
            } else {
              connectWallet();
            }
          }}
          class="flex-1 p-4 bg-zinc-100 dark:bg-zinc-900 border border-zinc-300 dark:border-zinc-700 rounded-lg text-zinc-900 dark:text-zinc-50 hover:bg-zinc-200 dark:hover:bg-zinc-800 font-semibold text-lg transition-all flex items-start text-left"
        >
          {#if cosmosStore.connectionStatus === 'connected'}
            <div class="w-full">
              <div class="text-xs text-zinc-400 dark:text-zinc-500 mb-1">Connected with</div>
              <div class="flex items-center gap-2">
                {#if connectedCosmosWallet?.icon}
                  <img 
                    src={connectedCosmosWallet.icon} 
                    alt="{connectedCosmosWallet.name ?? 'Cosmos Wallet'} icon" 
                    class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1 flex-shrink-0"
                  />
                {/if}
                {#if connectedCosmosWallet?.name}
                  <span class="font-semibold text-base text-zinc-900 dark:text-zinc-50 truncate">
                    {connectedCosmosWallet.name}
                  </span>
                {/if}
              </div>
            </div>
          {:else}
            {#if connectedCosmosWallet?.icon} <img 
                src={connectedCosmosWallet.icon} 
                alt="{connectedCosmosWallet.name ?? 'Cosmos Wallet'} icon" 
                class="w-7 h-7 rounded-md flex-shrink-0"
              />
            {/if}
            <span class="w-full text-center text-white">{cosmosButtonText}</span>
          {/if}
        </button>

        {#if sepoliaStore.connectionStatus === 'connected' || cosmosStore.connectionStatus === 'connected'}
          <Button  variant="secondary"
            onclick={connectWallet}
          >
            Connect Another Wallet
          </Button>
        {/if}

      </div>
    {:else if page === 'cosmos'}
      <div class="relative min-w-full flex flex-col flex-1 h-full min-h-0">
        <div class="grow flex flex-col gap-2 p-4 flex-1 min-h-0">
          <p class="text-sm text-zinc-400">
            Select the Cosmos chains you want to add to your dashboard.
          </p>

          <div class="mt-4">
            <div class="space-y-2 max-h-60 overflow-y-auto">
              {#if cosmosChains.length === 0}
                <p class="text-sm text-zinc-500">No Cosmos chains available or still loading...</p>
              {:else}
                {#each cosmosChains as chain (chain.chain_id)}
                  <button
                    onclick={() => toggleChain(chain.chain_id)}
                    class="flex items-center justify-between w-full px-4 py-3 rounded-lg bg-neutral-800 hover:bg-neutral-700 transition-colors border border-neutral-700 hover:border-neutral-600"
                  >
                    <span class="text-white text-sm font-medium">{chain.name}</span>
                    <div
                      class="w-4 h-4 rounded border bg-transparent flex items-center justify-center transition-colors"
                      class:border-accent={selectedChains.includes(chain.chain_id)} 
                      class:border-zinc-600={!selectedChains.includes(chain.chain_id)}
                    >
                      {#if selectedChains.includes(chain.chain_id)}
                        <svg class="w-2.5 h-2.5 text-accent pointer-events-none" viewBox="0 0 16 16" fill="currentColor">
                            <path d="M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z"/>
                        </svg>
                      {/if}
                    </div>
                  </button>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      </div>
    {:else if page === 'evm'}
      <div class="relative min-w-full flex flex-col flex-1 h-full min-h-0">
        <div class="grow flex flex-col gap-2 p-4 flex-1 min-h-0">
          <p class="text-sm text-zinc-400">
            To add your EVM wallet to your dashboard, you need to verify ownership by signing a message. This is a one-time verification for this wallet.
          </p>

          <div class="mt-4">
            <div class="text-sm text-zinc-400 mb-2">Wallet Details</div>
            <div class="bg-zinc-900/50 rounded-lg p-4 border border-zinc-800">
              {#if connectedEvmWallet}
                <div class="flex items-center gap-3 mb-3">
                  {#if connectedEvmWallet.icon}
                    <img 
                      src={connectedEvmWallet.icon} 
                      alt="{connectedEvmWallet.name ?? 'EVM Wallet'} icon" 
                      class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1 flex-shrink-0"
                    />
                  {/if}
                  <div>
                    <div class="text-sm font-medium text-white">{connectedEvmWallet.name}</div>
                    <div class="text-xs text-zinc-400 font-mono">
                      {sepoliaStore.address ? `${sepoliaStore.address.slice(0, 6)}...${sepoliaStore.address.slice(-4)}` : 'Not connected'}
                    </div>
                  </div>
                </div>
              {/if}
              
              <div class="text-sm text-zinc-400">
                {#if hasError}
                  <div class="text-red-400">
                    {message}
                  </div>
                {:else if message}
                  <div class="text-zinc-400">
                    {message}
                  </div>
                {:else}
                  Ready to verify
                {/if}
              </div>
            </div>
          </div>
        </div>
      </div>
    {:else if page === 'cosmos-verify'}
      <div class="relative min-w-full flex flex-col flex-1 h-full min-h-0">
        <div class="grow flex flex-col gap-2 p-4 flex-1 min-h-0">
          <p class="text-sm text-zinc-400">
            To add your Cosmos wallet to your dashboard, you need to verify ownership by signing a message. This is a one-time verification for this wallet.
          </p>

          <div class="mt-4">
            <div class="text-sm text-zinc-400 mb-2">Wallet Details</div>
            <div class="bg-zinc-900/50 rounded-lg p-4 border border-zinc-800">
              {#if connectedCosmosWallet}
                <div class="flex items-center gap-3 mb-3">
                  {#if connectedCosmosWallet.icon}
                    <img 
                      src={connectedCosmosWallet.icon} 
                      alt="{connectedCosmosWallet.name ?? 'Cosmos Wallet'} icon" 
                      class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1 flex-shrink-0"
                    />
                  {/if}
                  <div>
                    <div class="text-sm font-medium text-white">{connectedCosmosWallet.name}</div>
                    <div class="text-xs text-zinc-400 font-mono">
                      {cosmosStore.address ? `${cosmosStore.address.slice(0, 6)}...${cosmosStore.address.slice(-4)}` : 'Not connected'}
                    </div>
                  </div>
                </div>
              {/if}
              
              <div class="text-sm text-zinc-400">
                {#if hasError}
                  <div class="text-red-400">
                    {message}
                  </div>
                {:else if message}
                  <div class="text-zinc-400">
                    {message}
                  </div>
                {:else}
                  Ready to verify
                {/if}
              </div>
            </div>
          </div>
        </div>
      </div>
    {:else if page === 'completed'}
      <div class="flex flex-col flex-1 min-h-0">
        <div class="flex-1 min-h-0 flex items-center justify-center p-4">
          <div class="text-center">
            <div class="flex justify-center mb-3">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-10 w-10 text-accent"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 13l4 4L19 7"
                />
              </svg>
            </div>
            <h3 class="text-xl font-bold mb-1 text-zinc-400">
              Wallet Added Successfully!
            </h3>
            <p class="text-sm text-zinc-500">
              Your wallet has been verified and added to your dashboard
            </p>
          </div>
        </div>
      </div>
    {/if}

    <!-- Unified Bottom Bar -->
    {#if page !== 'select'} <!-- Show button bar for all pages except 'select' -->
      <div class="border-t border-zinc-800 bg-zinc-925">
        <div class="flex justify-between p-4">
          {#if showCancelButton}
            <Button variant="secondary" onclick={onCancelClick}>
              {cancelButtonText}
            </Button>
          {/if}
          {#if showSecondaryCloseButton} <!-- Specifically for "Close" on completed page -->
             <Button variant="secondary" onclick={() => { isOpen = false; reset(); }}>
              Close
            </Button>
          {/if}

          <div class="flex justify-end gap-2 flex-1">
            {#if Option.isSome(error) && page !== 'completed'}
              <Button variant="danger" onclick={() => (showError = true)}>
                Error
              </Button>
            {/if}
            <Button 
              variant={hasError && page !== 'completed' ? "danger" : "primary"} 
              onclick={onPrimaryButtonClick} 
              disabled={(page === 'cosmos' && selectedChains.length === 0)}
            >
              {primaryButtonText}
            </Button>
          </div>
        </div>
      </div>
    {/if}
  </div>

  <InsetError
    open={showError}
    error={Option.isSome(error) ? error.value : null}
    onClose={() => {
      showError = false;
      error = Option.none();
    }}
  />
</Modal>