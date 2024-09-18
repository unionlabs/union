import type { Icon } from "@astrojs/starlight/components"

type IconName = Parameters<typeof Icon>[number]["name"]

export const commandType = [
  "add",
  "run",
  "shell",
  "remove",
  "update",
  "create",
  "update",
  "dynamic"
] as const
export type CommandType = (typeof commandType)[number]

export const packageManagers = ["npm", "pnpm", "bun", "yarn"] as const
export type PackageManager = (typeof packageManagers)[number]

export interface Command {
  install: { short: string; standard: string }
  exec: { shell: string; dynamic: string }
  create: string
}

export const commands = {
  pnpm: {
    icon: "pnpm",
    documentation: "https://pnpm.io",
    commands: {
      run: "",
      add: "add",
      shell: "exec",
      dynamic: "dlx",
      create: "create",
      remove: "remove",
      update: "update"
    }
  },
  bun: {
    icon: "bun",
    documentation: "https://bun.sh",
    commands: {
      run: "",
      add: "add",
      dynamic: "x",
      shell: "exec",
      create: "create",
      remove: "remove",
      update: "update"
    }
  },
  npm: {
    icon: "seti:npm",
    documentation: "https://npm.im",
    commands: {
      run: "run",
      dynamic: "x",
      shell: "exec",
      add: "install",
      update: "update",
      create: "create",
      remove: "uninstall"
    }
  },
  yarn: {
    icon: "seti:yarn",
    documentation: "https://yarnpkg.com",
    commands: {
      run: "",
      add: "add",
      shell: "exec",
      dynamic: "dlx",
      create: "create",
      remove: "remove",
      update: "update"
    }
  }
} as const satisfies Record<
  PackageManager,
  {
    icon: IconName
    documentation: string
    commands: Record<CommandType, string>
  }
>
