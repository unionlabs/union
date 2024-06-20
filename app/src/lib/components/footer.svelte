<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import HomeIcon from "virtual:icons/lucide/home"
import TelescopeIcon from "virtual:icons/lucide/telescope"
import { Button } from "$lib/components/ui/button/index.ts"
import WalletMinimalIcon from "virtual:icons/lucide/wallet"
import ArrowDownUpIcon from "virtual:icons/lucide/arrow-up-down"
import FaucetDripIcon from "virtual:icons/fa6-solid/faucet-drip"

const onWalletClick = () => document.querySelector("button[data-dialog-trigger]")?.click()

const navigationIconStyle = "size-7 min-w-6 dark:hover:text-white text-zinc-accent"

$: isCurrentPage = (route: string) => $page.route.id?.split("/")[1] === route

let buttons = [
  { displayName: "home", href: "", icon: HomeIcon },
  { displayName: "transfer", href: "transfer", icon: ArrowDownUpIcon },
  { displayName: "explorer", href: "explorer", icon: TelescopeIcon },
  { displayName: "faucet", href: "faucet", icon: FaucetDripIcon }
]
</script>

<footer
  class={cn(
    'overflow-hidden fixed left-0 bottom-0 right-0 w-screen h-16 uppercase font-supermolot font-bold text-xl',
    'border-t bg-card',
    'grid md:hidden grid-cols-5 place-content-center items-center',
    // styling children
  )}
>
  {#each buttons as button}
    <Button
      href={`/${button.href}`}
      size="icon"
      variant="ghost"
      name={button.displayName}
      aria-label="Home page link"
      class={cn('flex flex-col text-xs gap-px h-16 w-full', isCurrentPage(button.href) ? 'bg-foreground text-primary-foreground hover:bg-foreground hover:text-primary-foreground' : 'bg-transparent')}
    >
      <svelte:component this={button.icon} class="size-7 min-w-6 dark:hover:text-white text-zinc-accent" />
      <div>{button.displayName}</div>
    </Button>
  {/each}

  <Button class="flex flex-col text-xs gap-px h-16 w-full" size="icon" name="wallet" type="button" variant="ghost" on:click={() => onWalletClick()}>
    <WalletMinimalIcon class={navigationIconStyle} />
    Wallet
  </Button>
</footer>
