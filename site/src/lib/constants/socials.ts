import type { Icon } from "astro-icon/components"

type IconProps = Parameters<typeof Icon>[0]

export const socialLinks: Array<{
  href: string
  icon: IconProps["name"]
  cta: string
}> = [
  {
    href: "https://x.com/union_build",
    icon: "fa6-brands:x-twitter",
    cta: "Follow us on X"
  },
  {
    href: "https://discord.union.build",
    icon: "tabler:brand-discord-filled",
    cta: "Join our Discord"
  }
]
