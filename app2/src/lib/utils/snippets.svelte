<script
  lang="ts"
  module
>
import type { Option } from "effect"
import type { Snippet } from "svelte"
export { empty, getOptionOrNull, mapOption, matchOption }
</script>

{#snippet empty()}{/snippet}

{#snippet mapOption<T>(self: Option.Option<T>, onSome: Snippet<[T]>)}
  {#if self._tag === "Some"}
    {@render onSome(self.value)}
  {/if}
{/snippet}

{#snippet matchOption<T>(
  self: Option.Option<T>,
  onSome: Snippet<[T]>,
  onNone?: Snippet<[]> | undefined,
)}
  {#if self._tag === "Some"}
    {@render onSome(self.value)}
  {:else}
    {@render (onNone ?? empty)()}
  {/if}
{/snippet}

{#snippet getOptionOrNull<T>(self: Option.Option<T>)}
  {#if self._tag === "Some"}
    {self.value}
  {/if}
{/snippet}
