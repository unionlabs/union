import { type VariantProps, tv } from "tailwind-variants"
import type { Button as ButtonPrimitive } from "bits-ui"
import Root from "./button.svelte"

const buttonVariants = tv({
  base: "inline-flex items-center justify-center whitespace-nowrap text-sm font-supermolot font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
  variants: {
    variant: {
      default: "bg-primary text-primary-foreground hover:bg-primary/90 uppercase font-semibold",
      destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
      outline: "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
      secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
      ghost: "hover:bg-accent hover:text-accent-foreground font-sans ",
      link: "text-primary no-underline hover:no-underline font-supermolot font-bold text-md decoration-transparent uppercase rounded-none border-solid"
    },
    size: {
      default: "h-10 px-4 py-2",
      sm: "h-9 px-3",
      lg: "h-11 px-8",
      icon: "h-10 w-10"
    }
  },
  defaultVariants: {
    variant: "default",
    size: "default"
  }
})

type Variant = VariantProps<typeof buttonVariants>["variant"]
type Size = VariantProps<typeof buttonVariants>["size"]

type Props = ButtonPrimitive.Props & {
  variant?: Variant
  size?: Size
  name?: string
  value?: any
}

type Events = ButtonPrimitive.Events

export {
  Root,
  type Props,
  type Events,
  //
  Root as Button,
  type Props as ButtonProps,
  type Events as ButtonEvents,
  buttonVariants
}
