<script
  lang="ts"
  module
>
import type { Cause, Either, Exit, Option } from "effect"
import type { Snippet } from "svelte"
export {
  /**
   * @category core
   */
  empty,
  /**
   * @category option
   */
  getOptionOrNull,
  /**
   * @category either
   */
  getRightOrNull,
  /**
   * @category either
   */
  mapEither,
  /**
   * @category option
   */
  mapOption,
  /**
   * @category either
   */
  matchEither,
  /**
   * @category option
   */
  matchOption,
  /**
   * @category runtime
   */
  matchRuntimeResult,
}
</script>

{#snippet empty()}{/snippet}

{#snippet mapOption<T>(self: Option.Option<T>, onSome: Snippet<[T]>)}
  {#if self._tag === "Some"}
    {@render onSome(self.value)}
  {/if}
{/snippet}

{#snippet matchOption<T>(
  self: Option.Option<T>,
  options: {
    onSome: Snippet<[T]>
    onNone: Snippet<[]>
  },
)}
  {#if self._tag === "Some"}
    {@render options.onSome(self.value)}
  {:else}
    {@render (options.onNone)()}
  {/if}
{/snippet}

{#snippet getOptionOrNull<T>(self: Option.Option<T>)}
  {#if self._tag === "Some"}
    {self.value}
  {/if}
{/snippet}

{#snippet mapEither<R>(self: Either.Either<R>, onSome: Snippet<[R]>)}
  {#if self._tag === "Right"}
    {@render onSome(self.right)}
  {/if}
{/snippet}

{#snippet matchEither<R, L = never>(
  self: Either.Either<R, L>,
  options: {
    onRight: Snippet<[R]>
    onLeft: Snippet<[L]>
  },
)}
  {#if self._tag === "Right"}
    {@render options.onRight(self.right)}
  {:else}
    {@render options.onLeft(self.left)}
  {/if}
{/snippet}

{#snippet getRightOrNull<R>(self: Either.Either<R>)}
  {#if self._tag === "Right"}
    {self.right}
  {/if}
{/snippet}

{#snippet matchRuntimeResult<A, E>(
  self: Option.Option<Exit.Exit<A, E>>,
  options: {
    onSuccess: Snippet<[A]>
    onFailure: Snippet<[Cause.Cause<E>]>
    onNone: Snippet<[]>
  },
)}
  {#if self._tag === "Some"}
    {#if self.value._tag === "Success"}
      {@render options.onSuccess(self.value.value)}
    {:else}
      {@render options.onFailure(self.value.cause)}
    {/if}
  {:else}
    {@render options.onNone()}
  {/if}
{/snippet}
