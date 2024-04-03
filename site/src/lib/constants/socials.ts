import type { Icon } from "@astrojs/starlight/components"

type IconProps = Parameters<typeof Icon>[0]

export const socialLinks: Array<{
  href: string
  icon: IconProps["name"]
  cta: string
}> = [
  {
    href: "https://x.com/union_build",
    icon: "x.com",
    cta: "Follow our X"
  },
  {
    href: "https://discord.union.build",
    icon: "discord",
    cta: "Join Discord"
  },
]
