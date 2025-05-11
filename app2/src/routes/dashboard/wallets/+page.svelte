<script lang="ts">
  import { dashboard } from '$lib/dashboard/stores/user.svelte';
  import { Option, Effect, pipe } from 'effect';
  import { extractErrorDetails } from '@unionlabs/sdk/utils';
  import WalletDialog from '$lib/dashboard/components/WalletDialog.svelte';
  import Truncate from '$lib/components/ui/Truncate.svelte';

  let isConnecting = $state(false);

  const evmWallets = $derived(
    Option.getOrNull(dashboard.wallets)?.filter(w => w.chain_id.startsWith('evm')) ?? []
  );

  const cosmosWallets = $derived(
    Option.getOrNull(dashboard.wallets)?.filter(w => w.chain_id.startsWith('cosmos')) ?? []
  );

  const groupedCosmosWallets = $derived(
    Object.entries(cosmosWallets.reduce((groups: Record<string, typeof cosmosWallets>, wallet) => {
      const key = wallet.grouping ?? 'ungrouped';
      if (!groups[key]) groups[key] = [];
      groups[key].push(wallet);
      return groups;
    }, {}))
  );

  async function removeWallet(event: MouseEvent, address: string) {
    event.preventDefault();

    const wallets = Option.getOrNull(dashboard.wallets);
    if (!wallets || wallets.length === 1) {
      console.error('Cannot delete last wallet');
      return;
    }

    await Effect.runPromise(
      pipe(
        dashboard.wallets?.removeWallet(address),
        Effect.catchAll((error) => {
          console.error('Failed to remove wallet:', extractErrorDetails(error));
          return Effect.void;
        })
      )
    );
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex justify-between items-center">
    <h1 class="text-2xl font-medium text-zinc-200">Wallets</h1>
    <button
      class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded-lg transition-colors"
      onclick={() => document.querySelector('dialog#connect-dialog')?.showPopover()}
    >
      Connect Wallet
    </button>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    {#if groupedCosmosWallets.length}
      <div class="bg-zinc-900 border border-zinc-800 rounded-lg">
        <div class="px-4 pt-4">
          <h3 class="text-xl font-mono uppercase text-[#9DE2EA]">
            cosmos
          </h3>
        </div>
        <div class="divide-y divide-zinc-800">
          {#each groupedCosmosWallets as [groupId, wallets]}
            <div class="p-4">
              <ul class="space-y-2">
                {#each wallets as wallet (wallet.address)}
                  <li class="text-md flex justify-between items-center p-2 bg-zinc-800/50 hover:bg-zinc-800 rounded-lg transition-colors group">
                    <button
                      class="flex items-center font-mono text-zinc-300 hover:text-[#9DE2EA] transition-colors"
                      onclick={() => copyToClipboard(wallet.address)}
                    >
                      <Truncate value={wallet.address} maxLength={20} showCopy={false} />
                      <span class="ml-2 opacity-0 group-hover:opacity-100 transition-opacity text-xs text-zinc-500">
                        copy
                      </span>
                    </button>
                    <button
                      onclick={(event) => removeWallet(event, wallet.address)}
                      class="text-zinc-500 hover:text-[#9DE2EA] transition-colors px-2 ml-2"
                    >
                      ×
                    </button>
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if evmWallets.length}
      <div class="bg-zinc-900 border border-zinc-800 rounded-lg">
        <div class="p-4">
          <h3 class="text-xl font-mono uppercase text-[#9DE2EA] mb-2">
            evm
          </h3>
          <ul class="space-y-2">
            {#each evmWallets as wallet (wallet.address)}
              <li class="text-md flex justify-between items-center p-2 bg-zinc-800/50 hover:bg-zinc-800 rounded-lg transition-colors group">
                <button
                  class="flex items-center font-mono text-zinc-300 hover:text-[#9DE2EA] transition-colors"
                  onclick={() => copyToClipboard(wallet.address)}
                >
                  <Truncate value={wallet.address} maxLength={20} showCopy={false} />
                  <span class="ml-2 opacity-0 group-hover:opacity-100 transition-opacity text-xs text-zinc-500">
                    copy
                  </span>
                </button>
                <button
                  onclick={(event) => removeWallet(event, wallet.address)}
                  class="text-zinc-500 hover:text-[#9DE2EA] transition-colors px-2 ml-2"
                >
                  ×
                </button>
              </li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  </div>
</div>

<WalletDialog />

