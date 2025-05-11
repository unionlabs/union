<script lang="ts">
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Option } from "effect";
  import { Effect } from "effect";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import Card from "$lib/components/ui/Card.svelte";
</script>

<Card class="flex flex-col gap-3 lg:gap-3 lg:w-64">
  <div class="flex items-center justify-between">
    <h2 class="text-lg font-bold">Accounts</h2>
  </div>
  <div class="flex flex-col justify-between h-full gap-1.5">
    <!-- GitHub Connection -->
    <button
      class="w-full bg-transparent hover:bg-zinc-900 rounded-lg p-2.5 flex items-center justify-between cursor-pointer transition-colors duration-200 ease-in-out focus:outline-none text-sm font-medium capitalize relative h-11 group"
      onclick={() => Effect.runPromise(
        Option.isSome(dashboard.connections?.github) && dashboard.connections.github.value
          ? dashboard.unlinkIdentity("github")
          : dashboard.linkIdentity("github")
      )}
    >
      <div class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {
          Option.isSome(dashboard.connections?.github) && dashboard.connections.github.value
            ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
            : 'bg-white/5 backdrop-blur-sm ring-white/20'
        }"></span>
        <span>GitHub</span>
      </div>
      <div class="flex items-center">
        {#if Option.isSome(dashboard.connections?.github) && dashboard.connections.github.value}
          <div
            class="p-1.5 rounded-lg border border-zinc-800 transition-colors cursor-pointer"
            aria-label="Disconnect GitHub"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-zinc-400 group-hover:text-red-500 transition-colors">
              <path d="M18.36 6.64A9 9 0 1 1 5.64 6.64a9 9 0 0 1 12.72 0" />
              <line x1="12" y1="2" x2="12" y2="12" />
            </svg>
          </div>
        {:else}
          <span class="text-xs text-zinc-400">Connect</span>
        {/if}
      </div>
    </button>

    <!-- Twitter Connection -->
    <button
      class="w-full bg-transparent hover:bg-zinc-900 rounded-lg p-2.5 flex items-center justify-between cursor-pointer transition-colors duration-200 ease-in-out focus:outline-none text-sm font-medium capitalize relative h-11 group"
      onclick={() => Effect.runPromise(
        Option.isSome(dashboard.connections?.twitter) && dashboard.connections.twitter.value
          ? dashboard.unlinkIdentity("twitter")
          : dashboard.linkIdentity("twitter")
      )}
    >
      <div class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {
          Option.isSome(dashboard.connections?.twitter) && dashboard.connections.twitter.value
            ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
            : 'bg-white/5 backdrop-blur-sm ring-white/20'
        }"></span>
        <span>Twitter</span>
      </div>
      <div class="flex items-center">
        {#if Option.isSome(dashboard.connections?.twitter) && dashboard.connections.twitter.value}
          <div
            class="p-1.5 rounded-lg border border-zinc-800 transition-colors cursor-pointer"
            aria-label="Disconnect Twitter"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-zinc-400 group-hover:text-red-500 transition-colors">
              <path d="M18.36 6.64A9 9 0 1 1 5.64 6.64a9 9 0 0 1 12.72 0" />
              <line x1="12" y1="2" x2="12" y2="12" />
            </svg>
          </div>
        {:else}
          <span class="text-xs text-zinc-400">Connect</span>
        {/if}
      </div>
    </button>

    <!-- Discord Connection -->
    <button
      class="w-full bg-transparent hover:bg-zinc-900 rounded-lg p-2.5 flex items-center justify-between cursor-pointer transition-colors duration-200 ease-in-out focus:outline-none text-sm font-medium capitalize relative h-11 group"
      onclick={() => Effect.runPromise(
        Option.isSome(dashboard.connections?.discord) && dashboard.connections.discord.value
          ? dashboard.unlinkIdentity("discord")
          : dashboard.linkIdentity("discord")
      )}
    >
      <div class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {
          Option.isSome(dashboard.connections?.discord) && dashboard.connections.discord.value
            ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
            : 'bg-white/5 backdrop-blur-sm ring-white/20'
        }"></span>
        <span>Discord</span>
      </div>
      <div class="flex items-center">
        {#if Option.isSome(dashboard.connections?.discord) && dashboard.connections.discord.value}
          <div
            class="p-1.5 rounded-lg border border-zinc-800 transition-colors cursor-pointer"
            aria-label="Disconnect Discord"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-zinc-400 group-hover:text-red-500 transition-colors">
              <path d="M18.36 6.64A9 9 0 1 1 5.64 6.64a9 9 0 0 1 12.72 0" />
              <line x1="12" y1="2" x2="12" y2="12" />
            </svg>
          </div>
        {:else}
          <span class="text-xs text-zinc-400">Connect</span>
        {/if}
      </div>
    </button>
  </div>
</Card> 