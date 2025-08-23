<script
  lang="ts"
  module
>
import type { Cause, Exit, Option } from "effect"
import type { Snippet } from "svelte"
export { empty, getOptionOrNull, mapOption, matchOption, matchRuntimeResult }
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

{#snippet matchRuntimeResult<A, E>(
  self: Option.Option<Exit.Exit<A, E>>,
  onSuccess: Snippet<[A]>,
  onFailure: Snippet<[Cause.Cause<E>]>,
  onNone: Snippet<[]>,
)}
  {#if self._tag === "Some"}
    {#if self.value._tag === "Success"}
      {@render onSuccess(self.value.value)}
    {:else}
      {@render onFailure(self.value.cause)}
    {/if}
  {:else}
    {@render onNone()}
  {/if}
{/snippet}
