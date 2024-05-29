<script lang="ts">
import { navigating } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import HomeIcon from "virtual:icons/lucide/home"
import MenuIcon from "virtual:icons/lucide/menu"
import WalletMinimalIcon from "virtual:icons/lucide/wallet"
import { Button } from "$lib/components/ui/button/index.ts"
import ArrowDownUpIcon from "virtual:icons/lucide/arrow-up-down"
import TelescopeIcon from "virtual:icons/lucide/telescope"
import FaucetDripIcon from "virtual:icons/fa6-solid/faucet-drip"
import { Navigation } from "$lib/components/navigation/index.ts"

const onWalletClick = () => document.querySelector("button[data-dialog-trigger]")?.click()

let navigationDrawerOpen = false
$: if ($navigating) navigationDrawerOpen = false

const navigationIconStyle = "size-7 min-w-6 dark:hover:text-white text-zinc-accent"
</script>

<footer
  class={cn(
    'overflow-hidden fixed left-0 bottom-0 right-0 w-screen h-16',
    'border-t-[1px] border-solid border-[#fafafa25] border-opacity-90 backdrop-blur-lg',
    'grid lg:hidden grid-cols-5 place-content-center divide-x-[1px] divide-[#fafafa25] items-center',
    // styling children
    '*:flex *:hover:bg-transparent *:text-xs *:gap-px *:flex-col *:h-16 *:w-full *:rounded-none *:border-solid *:border-t-0',
  )}
>
  <Button
    href="/"
    size="icon"
    name="home"
    variant="link"
    aria-label="Home page link"
  >
    <HomeIcon class={navigationIconStyle} />
    <div>Home</div>
  </Button>
  <Button
    size="icon"
    name="send"
    href="/transfer"
    variant="link"
    aria-label="send page link"
  >
    <ArrowDownUpIcon class={navigationIconStyle} />
    Transfer
  </Button>
  <Button
    size="icon"
    name="send"
    href="/explorer"
    variant="link"
    aria-label="send page link"
  >
    <TelescopeIcon class={navigationIconStyle} />
    Explorer
  </Button>
  <Button
    size="icon"
    name="send"
    href="/faucet"
    variant="link"
    aria-label="send page link"
  >
    <FaucetDripIcon class={navigationIconStyle} />
    Faucet
  </Button>
  <Button
    size="icon"
    name="wallet"
    type="button"
    variant="ghost"
    on:click={() => onWalletClick()}
  >
    <WalletMinimalIcon class={navigationIconStyle} />
    Wallet
  </Button>
</footer>
