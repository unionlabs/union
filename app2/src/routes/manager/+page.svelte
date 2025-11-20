<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import * as AppRuntime from "$lib/runtime"
import { Snippets } from "@unionlabs/effect-svelte"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { CosmosDisplay } from "@unionlabs/sdk/Ucs05"
import { Effect as E, Exit, Option, pipe, Record, Schema as S, String } from "effect"
import type { FormEventHandler, KeyboardEventHandler } from "svelte/elements"

let clicked = $state<number>(0)
const onClick = () => {
  console.log(clicked)
  clicked += 1
}

let rawTarget = $state<string>("")
const targetOnInput: FormEventHandler<HTMLInputElement> = (event) => {
  rawTarget = event.currentTarget.value
}

let rawSelector = $state<string>("")
const selectorOnInput: KeyboardEventHandler<HTMLInputElement> = (event) => {
  if (event.key === "Enter") {
    rawSelector = event.currentTarget.value
  }
}

const fetchRoleLabel = (
  roleId: BigInt,
) =>
  pipe(
    Cosmos.queryContract(
      CosmosDisplay.make({
        address: "union1g8eayx25kmzmywzwq4uw44ftfpqxfz6qplnyutwqdzn92reavtmqltyh3e",
      }),
      {
        get_role_labels: { role_ids: [roleId.toString()] },
      },
    ),
    // string(bigint) -> string
    E.flatMap(S.decodeUnknown(S.Record({
      key: S.String,
      value: S.NullOr(S.String),
    }))),
    E.map(Record.get(roleId.toString())),
    E.provide(Cosmos.Client.Live("http://localhost:26657")),
  )

const targetEffect = $derived.by(() => {
  return pipe(
    S.decodeUnknown(CosmosDisplay)({ _tag: "CosmosDisplay", address: rawTarget }),
    E.flatMap(
      target =>
        Cosmos.queryContract(
          CosmosDisplay.make({
            address: "union1g8eayx25kmzmywzwq4uw44ftfpqxfz6qplnyutwqdzn92reavtmqltyh3e",
          }),
          {
            get_target_function_role: { target: target.address, selector: rawSelector },
          },
        ),
    ),
    // string(bigint) -> string
    E.flatMap(S.decodeUnknown(S.BigInt)),
    E.flatMap(E.fn(function*(roleId) {
      return {
        roleLabel: yield* (yield* fetchRoleLabel(roleId)),
        roleId,
      }
    })),
    E.provide(Cosmos.Client.Live("http://localhost:26657")),
  )
})

const result = AppRuntime.runPromiseExit$(() => {
  if (clicked > 0) {
    return targetEffect
  }
  return { current: Option.none() } as unknown as typeof targetEffect
})
</script>

{#snippet onSuccess(x: { roleId: BigInt; roleLabel: String | null })}
  <section>
    <span>Role ID: {x.roleId}</span>
    {#if x.roleLabel !== null}
      <span class="text-gray-500">{x.roleLabel}</span>
    {/if}
  </section>
{/snippet}

{#snippet onNone()}
  <section>
    <span>Role ID: </span>
  </section>
{/snippet}

{#snippet onFailure(cause: any)}
  <section>
    <span>Role ID: </span>
    ERROR: {cause}
  </section>
{/snippet}

<Sections class="max-w-4xl mx-auto mt-40">
  <Card>
    <div class="flex flex-col gap-4 w-full">
      <Input
        id="target"
        label="target"
        type="text"
        required
        autocorrect="off"
        spellcheck="false"
        autocomplete="off"
        inputmode="text"
        autocapitalize="none"
        value={rawTarget}
        class="flex-grow text-center focus:text-white"
        onblur={targetOnInput}
      />

      <Input
        id="selector"
        label="selector"
        type="text"
        required
        autocorrect="off"
        spellcheck="false"
        autocomplete="off"
        inputmode="text"
        autocapitalize="none"
        value={rawSelector}
        class="flex-grow text-center focus:text-white"
        onkeydown={selectorOnInput}
      />

      <Button onclick={onClick}>CLICKME</Button>
    </div>

    {@render Snippets.matchRuntimeResult(result.current, {
        onSuccess,
        onFailure,
        onNone,
      })}
  </Card>
</Sections>
