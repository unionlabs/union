<script lang="ts">
import { Effect } from "effect"

const task = Effect.sync(() => {
  console.log("hi")
})
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>

<button on:click={() => Effect.runSync(task)}>Run effect</button>
