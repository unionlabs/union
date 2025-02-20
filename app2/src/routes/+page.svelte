<script lang="ts">
import { Console, Effect, Fiber, Schedule } from "effect"

let number = $state(0)
const program = Effect.repeat(
  Effect.sync(() => {
    console.log(number)
    number += 1
  }),
  Schedule.spaced("200 millis")
)
const fiber = Effect.runFork(program)
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>

<button onclick={() => Effect.runFork(Fiber.interrupt(fiber))}>Run effect</button>
{number}
