<script lang="ts">
import { toIsoString } from "$lib/utilities/date"

export let value: Date | undefined
</script>


<div class="text-muted-foreground">{#if value && value !== undefined}{toIsoString(new Date(value)).split('T')[1].slice(0, 5)}{:else}in transit{/if}</div>
