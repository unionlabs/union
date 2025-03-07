<script lang="ts">
  import Label from "$lib/components/ui/Label.svelte";
  import {getTransfer} from "../../../routes/transfer/transfer.svelte.ts";
  import {Option} from "effect";
  import {tokensStore} from "$lib/stores/tokens.svelte.ts";
  import Input from "$lib/components/ui/Input.svelte";
  import {fade, fly} from "svelte/transition";

  const {transfer} = getTransfer();
  let open = $state(false);
  let searchQuery = $state("");

  function ensureTokensForChain() {
    const chainId = transfer.sourceChain?.universal_chain_id;
    if (!chainId) return;
    const tokenData = tokensStore.getData(chainId);
    if (Option.isNone(tokenData)) {
      tokensStore.fetchTokens(chainId);
    }
  }

  $effect(() => {
    if (transfer.sourceChain) {
      ensureTokensForChain();
    }
  });

  const filteredTokens = $derived.by(() => {
    const query = searchQuery.toLowerCase();
    return transfer.baseTokens.filter((token) => (
      token.denom.toLowerCase().includes(query) ||
      (token.representations[0]?.name?.toLowerCase() || "").includes(query)
    ));
  });
</script>

{#if open}
  <div
          transition:fade="{{ duration: 500 }}"
          class="absolute top-0 left-0 w-full h-full z-30 flex items-center justify-center bg-zinc-900"
  >
    <div
            transition:fly="{{ y: 50, duration: 500, opacity: 0, delay: 100 }}"
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
      {#if transfer.sourceChain}
        {@const tokenData = tokensStore.getData(transfer.sourceChain.universal_chain_id)}
        {@const error = tokensStore.getError(transfer.sourceChain.universal_chain_id)}

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
                    transfer.url.updateField("asset", token.denom);
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
          disabled={!transfer.sourceChain}
          onclick={() => (open = !open)}
          class="w-full p-2 rounded-lg border border-zinc-600 bg-zinc-700 text-zinc-200 hover:bg-zinc-600 hover:border-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-500 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 cursor-pointer"
  >
    {(transfer.baseToken?.representations[0]?.name ?? transfer.url.asset) || "Select asset"}
  </button>
</div>
