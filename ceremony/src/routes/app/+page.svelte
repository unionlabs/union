<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
</script>

{#if user.session}
  <div class="max-w-7xl px-6 lg:px-8 py-24 overflow-y-scroll">
    <H1 class="mb-4">Session</H1>
    <Text class="break-all">{JSON.stringify(user)}</Text>
  </div>
{/if}