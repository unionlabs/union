<script lang="ts">
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"

export let value: string | undefined
</script>

<span class="text-muted-foreground">{#if value}{toPrettyDateTimeFormat(value, { local: true })}{:else}-{/if}</span>
