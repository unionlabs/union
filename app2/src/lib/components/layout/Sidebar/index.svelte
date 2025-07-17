<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import ConnectWalletButton from "$lib/components/ui/ConnectWalletButton.svelte"
import ProfileCard from "$lib/dashboard/components/SideCard.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Navigation from "../Navigation.svelte"
</script>

<div class="h-full overflow-y-auto">
  <div class="min-h-full flex flex-col">
    <a
      class="px-6 flex items-center gap-2 border-b-1 h-16 border-zinc-900"
      href="/"
    >
      <img
        class="h-10"
        src="/images/union-logo.svg"
        alt="Union"
      />
      {#key uiStore.edition}
        {#if uiStore.edition === "app"}
          <div class="flex items-center gap-1 bg-accent/10 border border-accent/20 px-1.5 py-1 rounded text-accent hover:bg-accent/20 transition-all duration-200">
            <span class="text-sm font-mono font-bold leading-none">{uiStore.theme.label}</span>
          </div>
        {:else}
          <div class="bg-accent px-2 py rounded text-sm font-mono font-bold">
            {uiStore.theme.label}
          </div>
        {/if}
      {/key}
    </a>

    <ProfileCard />

    <Navigation
      class="flex-1"
      variant="animated"
    />

    <div class="p-4 border-t border-zinc-800/50 bg-zinc-950/30">
      <ConnectWalletButton />
    </div>
  </div>
</div>
