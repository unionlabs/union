<script lang="ts">
import { DateTime } from "effect"
import * as A from "effect/Array"
import { constant, constFalse, constTrue, flow, identity, pipe } from "effect/Function"
import * as M from "effect/Match"
import * as N from "effect/Number"
import * as O from "effect/Option"
import * as P from "effect/Predicate"
import * as R from "effect/Record"
import * as Str from "effect/String"
import Self from "./JsonPreview.svelte"

export type Props = {
  value: unknown
  initiallyExpanded?: number | undefined
  maxStringLength?: number | undefined
  maxChildren?: number | undefined
  depth?: number | undefined
}

const {
  value,
  initiallyExpanded = 1,
  maxStringLength = 160,
  maxChildren = 300,
  depth = 0,
}: Props = $props()

const isNested: (u: unknown) => boolean = pipe(
  M.type<unknown>(),
  M.when(DateTime.isDateTime, constFalse),
  M.whenAnd(
    O.isOption,
    O.isSome,
    (x) => isNested(x.value),
  ),
  M.whenAnd(
    O.isOption,
    O.isNone,
    constFalse,
  ),
  M.when(M.string, constFalse),
  M.when(A.isArray, constTrue),
  M.when(P.isObject, constTrue),
  M.orElse(constFalse),
)

const formatPrimitive: (u: unknown) => string = pipe(
  M.type<unknown>(),
  M.when(
    O.isOption,
    O.match({
      onNone: constant("None"),
      onSome: (x) => `Some(${formatPrimitive(x)})`,
    }),
  ),
  M.when(DateTime.isDateTime, DateTime.formatIso),
  M.when(M.bigint, flow(String, Str.concat("n"))),
  M.when(M.string, JSON.stringify),
  M.when(M.null, constant("null")),
  M.when(M.undefined, constant("undefined")),
  M.when(M.boolean, String),
  M.orElse((x) => {
    try {
      return JSON.stringify(x)
    } catch {
      return String(x)
    }
  }),
)

const entriesOf: (u: unknown) => readonly (readonly [string, unknown])[] = pipe(
  M.type<unknown>(),
  M.when(A.isArray, A.map((x, i) => [String(i), x] as const)),
  M.when(P.isObject, flow(Object.entries, identity<readonly [string, unknown][]>)),
  M.orElse(A.empty<readonly [string, unknown]>),
)

const entries = $derived(pipe(value, entriesOf))
const truncated = $derived(pipe(entries, A.take(maxChildren)))
</script>

{#if isNested(value)}
  <details
    class="block font-mono text-xs leading-4 py-2"
    style={`--indent:${depth}`}
    data-depth={depth}
    open={depth < initiallyExpanded}
  >
    <summary class="flex items-center gap-2 cursor-pointer select-none outline-none [padding-inline-start:calc(var(--indent)*var(--spacing))] focus-visible:ring-1 focus-visible:ring-current rounded">
      {#if A.isArray(value)}
        <span class="inline-block">{"[…]"}</span>
        <span class="text-xs text-zinc-500">{(value as unknown[]).length} items</span>
      {:else if P.isRecord(value)}
        <span class="inline-block">{"{…}"}</span>
        <span class="text-xs text-zinc-500">{R.size(value)} keys</span>
      {:else}
        <span class="text-red-500 italic">render failure</span>
      {/if}
    </summary>

    <div class="mt-1 space-y-1 border-l border-zinc-600 [margin-inline-start:calc(var(--indent)*var(--spacing)]">
      {#each truncated as [k, v] (k)}
        {#if isNested(v)}
          <div class="px-2 py-1">
            <span class="text-xs block font-medium break-words text-accent">{k}:</span>
            <div class="mt-1">
              <Self
                value={v}
                initiallyExpanded={initiallyExpanded}
                maxStringLength={maxStringLength}
                maxChildren={maxChildren}
                depth={N.increment(depth)}
              />
            </div>
          </div>
        {:else}
          <div class="px-2 py-1 flex items-baseline gap-2 min-w-0">
            <span class="text-xs font-medium shrink-0 text-accent">{k}:</span>
            <Self
              value={v}
              initiallyExpanded={initiallyExpanded}
              maxStringLength={maxStringLength}
              maxChildren={maxChildren}
              depth={N.increment(depth)}
            />
          </div>
        {/if}
      {/each}

      {#if A.length(entries) > maxChildren}
        <div class="px-2 py-1 text-xs italic opacity-70 select-none">
          … {A.length(entries) - maxChildren} more
        </div>
      {/if}
    </div>
  </details>
{:else}
  <code class="font-mono text-xs whitespace-pre-wrap break-words break-all">
    {formatPrimitive(value)}
  </code>
{/if}
