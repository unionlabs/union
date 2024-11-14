<script lang="ts">
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"

export let value: Date | undefined
</script>

{#if value}{toPrettyDateTimeFormat(value, { local: true })}{:else}-{/if}
