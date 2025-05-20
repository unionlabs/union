<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Effect } from "effect"

type AuthProvider = {
  id: "twitter" | "github" | "discord"
  name: string
  icon: string
  iconColor: string
  disabled?: boolean
}

const providers: Array<AuthProvider> = [
  {
    id: "twitter",
    name: "X",
    icon:
      `<path fill="currentColor" d="m17.687 3.063l-4.996 5.711l-4.32-5.711H2.112l7.477 9.776l-7.086 8.099h3.034l5.469-6.25l4.78 6.25h6.102l-7.794-10.304l6.625-7.571zm-1.064 16.06L5.654 4.782h1.803l10.846 14.34z"/>`,
    iconColor: "text-white",
  },
  {
    id: "github",
    name: "GitHub",
    icon:
      `<path fill="currentColor" fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd"/>`,
    iconColor: "text-white",
    disabled: false,
  },
  {
    id: "discord",
    name: "Discord",
    icon:
      `<path fill="currentColor" d="M20.317 4.37a19.791 19.791 0 00-4.885-1.515.074.074 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 00-5.487 0 12.64 12.64 0 00-.617-1.25.077.077 0 00-.079-.037A19.736 19.736 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 00.031.057 19.9 19.9 0 005.993 3.03.078.078 0 00.084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 00-.041-.106 13.107 13.107 0 01-1.872-.892.077.077 0 01-.008-.128 10.2 10.2 0 00.372-.292.074.074 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 01.078.01c.12.098.246.198.373.292a.077.077 0 01-.006.127 12.299 12.299 0 01-1.873.892.077.077 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 00.084.028 19.839 19.839 0 006.002-3.03.077.077 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 00-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/>`,
    iconColor: "text-white",
  },
] as const

let loading = $state(false)

function handleLogin(provider: AuthProvider) {
  if (loading) {
    return
  }
  loading = true

  Effect.runPromise(
    Effect.gen(function*() {
      yield* dashboard.login(provider.id)
    }).pipe(
      Effect.tap(() => Effect.sync(() => loading = false)),
      Effect.catchAll(() => Effect.sync(() => loading = false)),
    ),
  )
}
</script>

<div class="h-full w-full flex flex-col items-center justify-center">
  <Card class="w-full max-w-lg p-8 shadow-lg backdrop-blur-sm bg-background/70">
    <div class="space-y-2.5">
      {#each providers as provider (provider.id)}
        <Button
          variant="secondary"
          class="w-full flex items-center justify-center gap-3 h-11 relative hover:translate-y-[1px] transition-all {provider.disabled ? 'opacity-30 cursor-not-allowed' : ''}"
          disabled={provider.disabled || loading}
          onclick={() => handleLogin(provider)}
          title={provider.disabled ? `${provider.name} is temporarily disabled` : ""}
        >
          <svg
            class="w-5 h-5 {provider.iconColor} {loading ? 'opacity-70' : ''}"
            viewBox="0 0 24 24"
          >
            {@html provider.icon}
          </svg>
          <span class={loading ? "opacity-70" : ""}>Continue with {provider.name}</span>
          {#if loading}
            <div
              class="absolute right-4 w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"
            >
            </div>
          {/if}
        </Button>
      {/each}
    </div>
  </Card>

  <div class="text-center mt-4 text-xs text-muted-foreground/80">
    By signing in, you agree to our{" "}
    <a
      href="/privacy-policy"
      class="font-medium text-primary hover:text-primary/80 underline underline-offset-4 decoration-primary/30 hover:decoration-primary/60 transition-all"
    >
      Privacy Policy
    </a>
  </div>
</div>
