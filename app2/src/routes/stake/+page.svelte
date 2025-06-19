<script lang="ts">
import SharpInfoIcon from "$lib/components/icons/SharpInfoIcon.svelte"
import * as AppRuntime from "$lib/runtime"
import * as Favicon from "$lib/services/Favicon"
import { empty, mapOption, matchRuntimeResult } from "$lib/utils/snippets.svelte"
import { FetchHttpClient } from "@effect/platform"
import * as Validators from "@unionlabs/sdk/schema/validators.js"
import * as Staking from "@unionlabs/sdk/Staking.js"
import { BigDecimal, Cause, Effect, Option as O, pipe, Record as R } from "effect"

const getFavicon = (url: URL) =>
  pipe(
    Favicon.Favicon,
    Effect.andThen(({ of }) => of(url)),
    Effect.withExecutionPlan(Favicon.Plan),
    Effect.provide(FetchHttpClient.layer),
  )

const program = Effect.gen(function*() {
  const staking = yield* Staking.Staking

  const validators = yield* staking.getValidators(
    new Staking.GetValidators({
      status: "BOND_STATUS_BONDED",
      pagination: {},
    }),
  )

  // const favicons = yield* pipe(
  //   R.fromIterableBy(validators, (x) => x.operator_address),
  //   R.map((x) =>
  //     pipe(
  //       x.description.website,
  //       Effect.transposeMapOption(getFavicon),
  //     )
  //   ),
  //   Effect.allWith({
  //     concurrency: 2,
  //   }),
  // )

  return {
    validators,
    favicons: {},
  }
}).pipe(
  Effect.provide(Staking.Staking.Default),
)

const data = AppRuntime.runPromiseExit$(() => program)
</script>

<h1>Stake</h1>

{#snippet renderError<E>(cause: Cause.Cause<E>)}
  <div class="text-red-500">
    {Cause.pretty(cause)}
  </div>
{/snippet}

{#snippet renderValidators(props: {
  validators: readonly Validators.Validator[]
  favicons: Record<string, O.Option<string>>
})}
  {#snippet renderFavicon(src: string)}
    <img
      class="size-12 rounded-full bg-zinc-600"
      src={`https://icons.duckduckgo.com/ip3/${src}.ico`}
      alt="alt"
    />
  {/snippet}

  <div class="grid [grid-template-columns:2fr_1fr_1fr] gap-4 font-mono">
    <div class="text-zinc-400 font-bold">
      <span>Validator</span>
    </div>
    <div class="flex items-center text-zinc-400 font-bold gap-2">
      <span>Signed Blocks</span>
      <SharpInfoIcon />
    </div>
    <div class="flex items-center text-zinc-400 font-bold gap-2">
      <span>Commission</span>
      <SharpInfoIcon />
    </div>
    <div class="col-span-3"><hr class="text-zinc-700" /></div>
    {#each props.validators as validator}
      <div class="flex flex-row items-center gap-2">
        {@render mapOption(O.map(validator.description.website, (x) => x.hostname), renderFavicon)}
        <!-- {@render mapOption(props.favicons[validator.operator_address], renderFavicon)} -->
        <div class="font-medium">{validator.description.moniker}</div>
      </div>
      <div class="font-medium font-mono text-xs">
        {validator.tokens}/{BigDecimal.format(validator.delegator_shares)}
      </div>
      <div class="font-medium font-mono text-xs">
        &percnt;{
          pipe(
            validator.commission.commission_rates.rate,
            BigDecimal.multiply(BigDecimal.make(100n, 0)),
            BigDecimal.unsafeToNumber,
          )
        }
      </div>
    {/each}
  </div>
{/snippet}

<div class="w-[900px] mx-8 p-8 bg-zinc-900">
  <div class="text-xl font-bold">
    Search Validators
  </div>
  {@render matchRuntimeResult(
      data.current,
      renderValidators,
      renderError,
      empty,
    )}
</div>

<pre>
{JSON.stringify(data.current, null, 2)}
</pre>
