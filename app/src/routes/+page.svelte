<script lang="ts">
  import Faucet from '$/lib/components/Faucet.svelte'
  import Connect from '$lib/components/Connect.svelte'
</script>

<main class="mx-auto mt-12 flex min-h-full min-w-full items-center justify-center flex-col space-y-6">
  <Connect />
  <Faucet />
</main>
