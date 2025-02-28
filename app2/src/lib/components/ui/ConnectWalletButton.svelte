<script lang="ts">
import Button from "./Button.svelte"
import SharpWalletIcon from "$lib/components/icons/SharpWalletIcon.svelte"
import { uiStore } from "$lib/stores/ui.svelte"

export let variant: "primary" | "secondary" = "secondary"
</script>

<Button {variant} onclick={() => uiStore.openWalletModal()}>
  <SharpWalletIcon class="size-5"/>
  Connect Wallet
</Button>
