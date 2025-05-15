<script lang="ts">
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Option } from "effect";
  import Card from "$lib/components/ui/Card.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import { page } from "$app/stores";
  import { uiStore } from "$lib/stores/ui.svelte";
  import WalletDialog from "./WalletDialog.svelte";

  // Get wallet statistics
  let stats = $derived(
    Option.flatMap(dashboard.wallets, (walletsStore) => 
      Option.flatMap(walletsStore.wallets, (_walletsData) => 
        Option.some(walletsStore.stats)
      )
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
          class="text-xs text-zinc-400 hover:text-white transition-colors border border-zinc-800 hover:border-zinc-700 px-2 py-0.5 rounded cursor-pointer"
        >
          View wallets
        </a>
      {:else}
        <WalletDialog />
      {/if}
    </div>

    {#if Option.isNone(stats)}
      <!-- Loading State -->
      <div class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-4">
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
      <div class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-4">
        <!-- Total Wallets -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Total Wallets</div>
          <div class="text-lg font-medium">
            {stats.value.total.toString()}
          </div>
        </div>

        <!-- Cosmos Wallets -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Cosmos Wallets</div>
          <div class="text-lg font-medium">
            {stats.value.cosmosCount.toString()}
          </div>
        </div>

        <!-- EVM Wallets -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">EVM Wallets</div>
          <div class="text-lg font-medium">
            {stats.value.evmCount.toString()}
          </div>
        </div>
      </div>
    {/if}
  </div>
</Card> 