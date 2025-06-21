<script lang="ts">
import SharpInfoIcon from "$lib/components/icons/SharpInfoIcon.svelte"
import * as AppRuntime from "$lib/runtime"
import { empty, mapOption, matchRuntimeResult } from "$lib/utils/snippets.svelte"
import * as Validators from "@unionlabs/sdk/schema/validators.js"
import * as Staking from "@unionlabs/sdk/Staking.js"
import { BigDecimal, Cause, Effect, Option as O, pipe, Schema as S } from "effect"

const limit = $state<5 | 10 | 30>(5)
const status = $state<Validators.Status | undefined>(undefined)

const arg = $derived(pipe(
  {
    _tag: "GetValidators",
    status,
    pagination: {
      limit,
    },
  },
  S.decodeSync(Staking.GetValidators),
))

const validators = AppRuntime.runPromiseExit$(() =>
  pipe(
    Staking.Staking,
    Effect.andThen((staking) => staking.getValidators(arg)),
    Effect.provide(Staking.Staking.Default),
  )
)

const params = AppRuntime.runPromiseExit$(() =>
  pipe(
    Staking.Staking,
    Effect.andThen((staking) =>
      pipe(
        new Staking.GetParams({}),
        staking.getParams,
      )
    ),
    Effect.provide(Staking.Staking.Default),
  )
)
</script>

<h1 class="text-xl font-bold">Stake</h1>

{#snippet renderError<E>(cause: Cause.Cause<E>)}
  <div class="text-red-500">
    {Cause.pretty(cause)}
  </div>
{/snippet}

{#snippet renderValidators(validators: readonly Validators.ValidatorWithImage[])}
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
    {#each validators as validator}
      {#snippet renderIcon(url: URL)}
        <img
          class="size-12 rounded-full bg-zinc-600"
          src={url.toString()}
          alt={O.getOrUndefined(validator.description.identity) ?? validator.description.moniker}
        />
      {/snippet}
      <div class="flex flex-row items-center gap-2">
        {@render mapOption(validator.icon, renderIcon)}
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

<div class="mx-8 p-8 bg-zinc-900">
  <div class="text-xl font-bold">
    Params
  </div>
  <pre>{JSON.stringify(params.current, null, 2)}</pre>
</div>
<br />
<div class="mx-8 p-8 bg-zinc-900">
  <div class="text-xl font-bold">
    Search Validators
  </div>
  {@render matchRuntimeResult(
      validators.current,
      renderValidators,
      renderError,
      empty,
    )}
</div>

<details>
  {JSON.stringify(validators.current, null, 2)}
</details>
