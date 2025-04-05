<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import SharpWalletIcon from "$lib/components/icons/SharpWalletIcon.svelte"
</script>

<Card divided class="w-lg self-center">
  <div class="p-4 flex gap-1 ">
    <h2>UNO Faucet</h2>
  </div>
  <div class="flex flex-col gap-4 p-4">
    <div>
      <p>Union's official faucet for the UNO testnet token.</p>
      <p>This faucet is protected by CloudFlare Turnstile.</p>
      <p>You can use this faucet once a day.</p>
    </div>
    <div class="flex gap-4">
      <Button class="flex-1">Claim</Button>
      <Button><SharpWalletIcon class="size-5"/></Button>
    </div>
  </div>
</Card>
