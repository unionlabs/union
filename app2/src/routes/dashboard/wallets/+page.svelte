<script lang="ts">
import WalletConnectModal from "$lib/components/ui/Wallet/index.svelte"
import WalletDisplay from "$lib/dashboard/components/WalletDisplay.svelte"
import WalletStats from "$lib/dashboard/components/WalletStats.svelte"
</script>

<div class="flex flex-col gap-6">
  <WalletStats />
  <WalletDisplay />
</div>
