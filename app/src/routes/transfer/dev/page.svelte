<script lang="ts">
  import { wagmiConfig } from '$lib/wallet/evm'
  import { createCosmosSdkClient } from '@union/client'
  import Button from '$lib/components/ui/button/button.svelte'

  const onTransferClick = (event: MouseEvent) => {
    event.preventDefault()
  }
</script>

<Button on:click={event => {}}></Button>

<style lang="postcss"></style>
