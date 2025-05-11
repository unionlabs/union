<script lang="ts">
    import type { Snippet } from "svelte";
    import Sections from "$lib/components/ui/Sections.svelte";
    import NewUser from "$lib/dashboard/components/NewUser.svelte";
    import { onMount, onDestroy } from "svelte";
  import { getLastGlobalSync } from "$lib/dashboard/cache";
  import { Option } from "effect";
  
    interface Props {
      children: Snippet;
    }
  
    let { children }: Props = $props();
  
    let lastSyncLabel: string | null = $state(null);
  
    const updateLastSync = () => {
      const meta = getLastGlobalSync("sv-SE");
      console.log("meta:", meta);
      lastSyncLabel = Option.isSome(meta) ? meta.value.label : null;
      console.log("lastSyncLabel:", lastSyncLabel);
    };
  
    onMount(() => {
      updateLastSync();
      const interval = setInterval(updateLastSync, 60_000); 
  
      onDestroy(() => clearInterval(interval));
    });
  </script>
  
  <Sections>
    {#if lastSyncLabel}
      <p class="text-xs text-zinc-400 -my-4 px-2 text-end">{lastSyncLabel}</p>
    {/if}
    <NewUser />
    {@render children()}
  </Sections>
  