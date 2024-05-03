<script lang="ts">
import { toast } from "svelte-sonner"
import { cn } from "$/lib/utilities/shadcn.ts"
import MenuIcon from "lucide-svelte/icons/menu"
import HomeIcon from "lucide-svelte/icons/home"
import Button from "$/lib/components/ui/button/button.svelte"
import ArrowDownUpIcon from "lucide-svelte/icons/arrow-down-up"
import WalletMinimalIcon from "lucide-svelte/icons/wallet-minimal"

const onWalletClick = () => document.querySelector("button[data-dialog-trigger]")?.click()

const navigationButtonStyle =
  "my-auto self-center hover:bg-transparent h-16 w-full rounded-none hover:bg-muted border-solid"
const navigationIconStyle = "size-7 min-w-6 hover:text-white text-zinc-300"
</script>

<footer
  class={cn([
    'overflow-hidden fixed left-0 bottom-0 right-0 w-screen h-16 py-2',
    'grid md:hidden grid-cols-4 gap-y-2 place-content-center divide-x-2 divide-[#fafafa25]',
    'border-t-[1px] border-solid border-[#fafafa25] border-opacity-90 backdrop-blur-sm',
  ])}
>
  <Button
    href="/"
    size="lg"
    name="home"
    variant="link"
    aria-label="Home page link"
    class={navigationButtonStyle}
  >
    <HomeIcon class={navigationIconStyle} />
  </Button>
  <Button
    size="lg"
    name="send"
    href="/send"
    variant="link"
    aria-label="send page link"
    class={navigationButtonStyle}
  >
    <ArrowDownUpIcon class={navigationIconStyle} />
  </Button>
  <Button
    size="lg"
    name="wallet"
    type="button"
    variant="ghost"
    on:click={() => onWalletClick()}
    class={navigationButtonStyle}
  >
    <WalletMinimalIcon class={navigationIconStyle} />
  </Button>

  <Button
    size="lg"
    name="menu"
    type="button"
    variant="ghost"
    on:click={() => {
      toast.message("I haven't been configured yet", {
        class:
          'h-10 p-3 border-[1.75px] border-solid border-accent-foreground bg-accent-foreground bg-opacity-10',
        position: 'top-center',
        cancel: { label: 'X' },
      })
    }}
    class={navigationButtonStyle}
  >
    <MenuIcon class={navigationIconStyle} />
  </Button>
</footer>
