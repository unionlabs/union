<script>
import Wallet from "$lib/components/ui/Wallet/index.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import ConnectWalletButton from "$lib/components/ui/ConnectWalletButton.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Button from "$lib/components/ui/Button.svelte"
import SharpSettingsIcon from "$lib/components/icons/SharpSettingsIcon.svelte"
</script>

<Sections>
  <h1 class="font-extrabold text-3xl uppercase">Union</h1>
  <section>
    <ul>
      <li><a href="/transfer" class="underline">Transfer</a></li>
      <li><a href="/transfers" class="underline">Your Transfers</a></li>
    </ul>
  </section>
  <section>
    <h2 class="font-bold text-xl">Explorer</h2>
    <ul>
      <li><a href="/" class="underline">Home</a></li>
      <li><a href="/explorer/transfers" class="underline">Transfers</a></li>
    </ul>
  </section>

  <div class="flex flex-col gap-2">
    <ConnectWalletButton/>
    <Button variant="secondary" onclick={() => uiStore.openSettingsModal()}>
      <SharpSettingsIcon class="size-5"/>
      Settings
    </Button>
  </div>

</Sections>
