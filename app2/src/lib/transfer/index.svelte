<script lang="ts">
import Normal from "./normal/index.svelte"
import Multisig from "./multisig/index.svelte"
import { onMount } from "svelte"
import { signingMode, type SigningMode } from "./signingMode.svelte.js"
import { Match, pipe } from "effect"

type Props = {
  mode: "normal" | "multisig"
}

let props: Props = $props()

onMount(() => {
  signingMode.setMode(props.mode === "normal" ? "single" : "multi")
})

const matchComponent = Match.type<SigningMode>().pipe(
  Match.when("single", () => Normal),
  Match.when("multi", () => Multisig),
  Match.exhaustive
)

const Component = $derived(pipe(signingMode.mode, matchComponent))
</script>

<Component />

