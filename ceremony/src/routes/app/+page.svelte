<script lang="ts">

  import {user} from "$lib/stores/user.svelte.ts";
  import Text from "$lib/components/typography/Text.svelte";
</script>

{#if user.session}
  <div>
    <Text>{user.session.user.email}</Text>
  </div>
{/if}