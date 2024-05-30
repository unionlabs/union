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

const isCurrentPage = (route: string) => $page.route.id === route
</script>

<footer
  class={cn(
    'overflow-hidden fixed left-0 bottom-0 right-0 w-screen h-16',
    'border-t-[1px] border-solid border-secondary border-opacity-90 backdrop-blur-lg',
    'grid lg:hidden grid-cols-5 place-content-center items-center',
    // styling children
    '*:flex *:hover:bg-transparent *:text-xs *:gap-px *:decoration-primary/75 *:flex-col *:h-16 *:w-full *:rounded-none *:border-solid *:border-t-0',
  )}
>
  <Button
    href="/"
    size="icon"
    name="home"
    variant="link"
    aria-label="Home page link"
    class={cn(isCurrentPage('/') ? 'bg-secondary/30' : 'bg-transparent')}
  >
    <HomeIcon class={navigationIconStyle} />
    <div>Home</div>
  </Button>
  <Button
    size="icon"
    variant="link"
    name="transfer"
    href="/transfer"
    aria-label="send page link"
    class={cn(isCurrentPage('/transfer') ? 'bg-secondary/30' : 'bg-transparent')}
  >
    <ArrowDownUpIcon class={navigationIconStyle} />
    Transfer
  </Button>
  <Button
    size="icon"
    variant="link"
    name="explorer"
    href="/explorer"
    aria-label="send page link"
    class={cn(isCurrentPage('/explorer') ? 'bg-secondary/30' : 'bg-transparent')}
  >
    <TelescopeIcon class={navigationIconStyle} />
    Explorer
  </Button>
  <Button
    size="icon"
    name="faucet"
    href="/faucet"
    variant="link"
    aria-label="send page link"
    class={cn(isCurrentPage('/faucet') ? 'bg-secondary/30' : 'bg-transparent')}
  >
    <FaucetDripIcon class={navigationIconStyle} />
    Faucet
  </Button>
  <Button size="icon" name="wallet" type="button" variant="ghost" on:click={() => onWalletClick()}>
    <WalletMinimalIcon class={navigationIconStyle} />
    Wallet
  </Button>
</footer>
