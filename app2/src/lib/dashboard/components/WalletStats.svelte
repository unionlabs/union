<script lang="ts">
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Option } from "effect";
  import Card from "$lib/components/ui/Card.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import { page } from "$app/stores";

  // Get wallet statistics
  let stats = $derived(
    Option.flatMap(dashboard.wallets, (wallets) => 
      Option.some(wallets.stats)
    )
  );

  // Get wallets by chain
  let byChain = $derived(
    Option.flatMap(dashboard.wallets, (wallets) => 
      Option.some(wallets.byChain)
    )
  );

  // Check if we're on the wallets page
  let isOnWalletsPage = $derived($page.url.pathname === '/dashboard/wallets');
</script>

<Card class="flex flex-col flex-1">
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-zinc-200">Wallet Stats</h3>
      {#if !isOnWalletsPage}
        <a 
          href="/dashboard/wallets" 
          class="text-xs text-zinc-400 hover:text-white transition-colors border border-zinc-800 hover:border-zinc-700 px-2 py-0.5 rounded"
        >
          View wallets
        </a>
      {/if}
    </div>

    {#if Option.isNone(dashboard.wallets)}
      <!-- Loading State -->
      <div class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4">
        <div class="flex flex-col gap-1">
          <Skeleton class="h-3 w-16" />
          <Skeleton class="h-6 w-12" />
        </div>
        <div class="flex flex-col gap-1">
          <Skeleton class="h-3 w-24" />
          <Skeleton class="h-6 w-12" />
        </div>
        <div class="flex flex-col gap-1">
          <Skeleton class="h-3 w-20" />
          <Skeleton class="h-6 w-12" />
        </div>
      </div>
    {:else}
      <!-- Wallet Statistics -->
      <div class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4">
        <!-- Total Wallets -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Total Wallets</div>
          <div class="text-lg font-medium">
            {Option.isNone(stats) ? '0' : stats.value.total.toString()}
          </div>
        </div>

        <!-- Connected Chains -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Connected Chains</div>
          <div class="text-lg font-medium">
            {Option.isNone(stats) ? '0' : stats.value.chains.toString()}
          </div>
        </div>

        <!-- Grouped Wallets -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Grouped Wallets</div>
          <div class="text-lg font-medium">
            {Option.isNone(stats) ? '0' : stats.value.grouped.toString()}
          </div>
        </div>
      </div>

      <!-- Chain Breakdown -->
      {#if Option.isSome(byChain) && Object.keys(byChain.value).length > 0}
        <div class="mt-4">
          <div class="text-xs text-zinc-500 mb-2">Connected Chains</div>
          <div class="flex flex-wrap gap-2">
            {#each Object.entries(byChain.value) as [chain, wallets]}
              <div class="flex items-center gap-2 px-2 py-1 bg-zinc-800/50 rounded text-sm">
                <span class="text-zinc-300">{chain}</span>
                <span class="text-zinc-500">({wallets.length})</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</Card> 