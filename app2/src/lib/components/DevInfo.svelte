<script lang="ts">
import { PUBLIC_GIT_REV, PUBLIC_LAST_MODIFIED_EPOCH } from "$env/static/public"
import { ENV } from "$lib/constants"
import { Number as N, Option as O, pipe, Schema as S, String as Str } from "effect"

const revision = O.liftPredicate(PUBLIC_GIT_REV, Str.isNonEmpty)
const lastModified = pipe(
  PUBLIC_LAST_MODIFIED_EPOCH,
  O.liftPredicate(Str.isNonEmpty),
  O.flatMap(S.decodeOption(S.NumberFromString)),
  O.map(N.multiply(1000)), // to millis
  O.flatMap(S.decodeOption(S.DateFromNumber)),
  O.map(x => x.toISOString()),
)
</script>

{#snippet sep()}
  |&nbsp;
{/snippet}

{#if ENV() !== "PRODUCTION"}
  <div
    class="absolute bottom-0 right-0 bg-zinc-900/[var(--bg-opacity)] [--bg-opacity:80%] text-accent px-[1em] py-2 font-mono rounded-tl-lg z-999"
  >
    <span>
      {`${ENV().toLowerCase()}`}
      {#if O.isSome(revision)}
        {@render sep()}{revision.value}
      {/if}
      {#if O.isSome(lastModified)}
        {@render sep()}{lastModified.value}
      {/if}
    </span>
  </div>
{/if}
