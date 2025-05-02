<script lang="ts">
import { Match, pipe } from "effect"
import { onMount } from "svelte"
import Multisig from "./multisig/index.svelte"
import Normal from "./normal/index.svelte"
import { type SigningMode, signingMode } from "./signingMode.svelte.js"

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
  Match.exhaustive,
)

const Component = $derived(pipe(signingMode.mode, matchComponent))
</script>

<Component />
