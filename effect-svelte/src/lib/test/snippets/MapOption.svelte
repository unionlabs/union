<script lang="ts">
import { mapOption } from "$lib/Snippets.svelte"
import type { Option } from "effect/Option"
type Props = {
  value: Option<unknown>
}
const { value }: Props = $props()
</script>

{#snippet onSome(a: unknown)}
  <div data-testid="Some">{a}</div>
{/snippet}

{@render mapOption(value, onSome)}
