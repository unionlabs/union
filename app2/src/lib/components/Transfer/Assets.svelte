<script lang="ts">
  import Label from "$lib/components/ui/Label.svelte"
  import {Option} from "effect"
  import {tokensStore} from "$lib/stores/tokens.svelte.ts"
  import Input from "$lib/components/ui/Input.svelte"
  import {fade, fly} from "svelte/transition"
  import {transfer} from "$lib/components/Transfer/transfer.svelte.ts"

  let open = $state(false)
  let searchQuery = $state("")

  function ensureTokensForChain() {
    if (Option.isNone(transfer.sourceChain)) return;

    const chainId = transfer.sourceChain.value.universal_chain_id;
    if (!chainId) return;

    const tokenData = tokensStore.getData(chainId);
    if (Option.isNone(tokenData)) {
      tokensStore.fetchTokens(chainId);
    }
  }

  $effect(() => {
    if (Option.isSome(transfer.sourceChain)) {
      ensureTokensForChain()
    }
  })

  const filteredTokens = $derived.by(() => {
    const query = searchQuery.toLowerCase();
    return Option.getOrElse(transfer.baseTokens, () => []).filter(
      (token) =>
        token.denom.toLowerCase().includes(query) ||
        (token.representations[0]?.name?.toLowerCase() || "").includes(query)
    );
  });
</script>

{#if open}
  <div
          transition:fade={{ duration: 500 }}
          class="absolute top-0 left-0 w-full h-full z-30 flex items-center justify-center bg-zinc-900"
  >
    <div
            transition:fly={{ y: 50, duration: 500, opacity: 0, delay: 100 }}
            class="w-full h-full rounded-lg p-4 shadow-lg flex flex-col"
    >
      <!-- Search Bar -->
      <Input
              label="Search"
              type="text"
              placeholder="Search by name or paste address"
              value={searchQuery}
              oninput={(e) => (searchQuery = (e.currentTarget as HTMLInputElement).value)}
      />

      <!-- Scrollable Token List -->
      {#if Option.isSome(transfer.sourceChain)}
        {@const tokenData = tokensStore.getData(transfer.sourceChain.value.universal_chain_id)}
        {@const error = tokensStore.getError(transfer.sourceChain.value.universal_chain_id)}

        {#if Option.isSome(error)}
          <p class="text-center text-red-500">Error: {error.value.message}</p>
        {:else if Option.isNone(tokenData)}
          <p class="text-center text-zinc-400">Loading tokens...</p>
        {:else}
          <ul class="space-y-2 overflow-y-auto flex-1">
            {#each filteredTokens as token}
              <li>
                <button
                        class="w-full p-2 text-left text-zinc-200 hover:bg-[#2a3535] hover:text-white focus:outline-none focus:bg-[#2a3535] transition-all duration-200"
                        onclick={() => {
                    transfer.raw.updateField("asset", token.denom);
                    open = false;
                    searchQuery = "";
                  }}
                >
                  <span class="font-medium">
                    {token.representations[0]?.name ?? token.denom}
                  </span>
                  {#if token.representations[0]?.name}
                    <span class="text-sm text-zinc-400 ml-2">({token.denom})</span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      {:else}
        <p class="text-center text-zinc-400">No chain selected</p>
      {/if}
    </div>
  </div>
{/if}

<div class="space-y-1">
  <Label>Asset</Label>
  <button
          disabled={Option.isNone(transfer.sourceChain)}
          onclick={() => (open = !open)}
          class="w-full p-2 rounded-lg border border-zinc-600 bg-zinc-700 text-zinc-200 hover:bg-zinc-600 hover:border-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-500 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 cursor-pointer"
  >
    {#if transfer.raw.asset && Option.isNone(transfer.baseToken)}
      <span class="flex items-center justify-center">
        <svg
                class="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
        >
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
        Loading...
      </span>
    {:else}
      {Option.match(transfer.baseToken, {
        onNone: () => transfer.raw.asset || "Select asset",
        onSome: (token) => token.representations[0]?.name ?? token.denom
      })}
    {/if}
  </button>
</div>