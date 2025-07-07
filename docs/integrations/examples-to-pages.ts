import type { StarlightPlugin } from "@astrojs/starlight/types"
import * as A from "effect/Array"
import { pipe } from "effect/Function"
import * as R from "effect/Record"
import * as Glob from "glob"
import * as NFS from "node:fs"
import * as Path from "node:path"
import { Project, ScriptTarget, SyntaxKind } from "ts-morph"

const yaml = (o: Record<string, unknown>, n = 0): string =>
  Object.entries(o)
    .filter(([, v]) => v !== undefined && v !== "")
    .map(([k, v]) =>
      typeof v === "object"
        ? " ".repeat(n) + k + ":\n" + yaml(v as Record<string, unknown>, n + 2)
        : " ".repeat(n) + `${k}: ${String(v).replace(/\n/g, " ")}`
    )
    .join("\n")

interface Options {
  readonly baseDir: string | string[]
  /** Glob patterns or literal paths (relative to your repo root). */
  readonly entryPoints: string[]
  /** Sub‑folder under `src/content/docs/` (default `examples`).   */
  readonly outDir?: string
}

export default function examplesToPages({
  baseDir,
  entryPoints,
  outDir = "examples",
}: Options): StarlightPlugin {
  if (!entryPoints?.length) {
    throw new Error("[examples‑to‑pages] entryPoints is required")
  }

  const bases = pipe(
    A.ensure(baseDir),
    A.map((xs) => Path.resolve(xs)),
  )

  console.log({ bases })

  return {
    name: "starlight-examples-to-pages",
    hooks: {
      async "config:setup"({ command, logger }) {
        if (command !== "build" && command !== "dev") {
          return
        }

        const files = await Glob.glob(entryPoints, {
          cwd: process.cwd(),
          absolute: true,
          nodir: true,
        })

        if (!files.length) {
          logger.warn("No .ts files matched entryPoints")
          return
        }
        logger.info(`Generating docs pages: ${JSON.stringify(files, null, 2)}`)

        const project = new Project({ compilerOptions: { target: ScriptTarget.ESNext } })

        const docsRoot = Path.join(process.cwd(), outDir)
        await NFS.promises.mkdir(docsRoot, { recursive: true })

        await Promise.all(
          files.map(async (abs) => {
            const src = project.addSourceFileAtPath(abs)
            const jsdoc = src.getDescendantsOfKind(SyntaxKind.JSDoc)[0]

            const tag = (n: string) =>
              jsdoc?.getTags().find((t) => t.getTagName() === n)?.getCommentText() ?? ""

            const title = tag("title") || Path.basename(abs, ".ts").replace(/[-_]/g, " ")
            const description = tag("description") || ""
            const summary = tag("summary") || ""
            const badge = tag("badge") || ""
            const text = badge.split(":")[0]
            const variant = badge.split(":")[1]

            const sidebar = badge
              ? {
                badge: { text, variant },
              }
              : undefined

            const body = `---\n`
              + yaml({
                title,
                description,
                sidebar,
              })
              + `\n---\n\n`
              + `${summary}\n`
              + "```ts twoslash\n"
              + (await NFS.promises.readFile(abs, "utf8")).replace(/```/g, "``\\`")
              + "\n```"

            const base = bases.find((b) => abs.startsWith(b))
            const rel = base ? Path.relative(base, abs) : Path.basename(abs)
            const outPath = Path.join(docsRoot, rel.replace(/\.ts$/, ".md"))

            try {
              const existing = await NFS.promises.readFile(outPath, "utf8")
              if (existing === body) {
                return
              }
            } catch {}

            await NFS.promises.mkdir(Path.dirname(outPath), { recursive: true })
            await NFS.promises.writeFile(outPath, body, "utf8")
          }),
        )
      },
    },
  }
}
