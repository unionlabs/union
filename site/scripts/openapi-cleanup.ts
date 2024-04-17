#!/usr/bin/env bun
import bun from "bun"

const [, , filepath] = bun.argv

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error)
    process.exit(1)
  })

async function main() {
  const openApiSchema = await bun.file(filepath).json()

  const removablePaths = ["circuit/", "nft/"]

  for (const key in openApiSchema.paths) {
    if (removablePaths.some(path => key.includes(path))) {
      delete openApiSchema.paths[key as keyof typeof openApiSchema.paths]
    }
  }

  await Bun.write("union_unused_paths_removed.json", JSON.stringify(openApiSchema, undefined, 2))

  const validateCommand =
    await bun.$`openapi-generator-cli validate --input-spec ./union_unused_paths_removed.json --recommend`.text()

  const warningsArray = validateCommand.split("\n").slice(2).slice(0, -3)

  const unusedModels: Array<string> = []
  for (const warning of warningsArray) {
    warning.trim()
    if (warning.startsWith("\t  ibc.")) {
      unusedModels.push(warning.slice(warning.indexOf("  ") + 1).trim())
      continue
    }
    const sliceableText = "\t- Unused model: "
    const modelName = warning.slice(warning.indexOf(sliceableText) + sliceableText.length)
    if (modelName && !modelName.includes(" ")) unusedModels.push(modelName)
  }

  for (const modelName of unusedModels) {
    if (Object.hasOwn(openApiSchema.components.schemas, modelName)) {
      delete openApiSchema.components.schemas[
        modelName as keyof typeof openApiSchema.components.schemas
      ]
    }
  }

  /**
   * for each path in paths, if there's a `summary` key in path.{get,pot,post,put,delete},
   * rename it to `description`
   */
  for (const path in openApiSchema.paths) {
    for (const method in openApiSchema.paths[path]) {
      if (openApiSchema.paths[path][method].summary) {
        openApiSchema.paths[path][method].description = openApiSchema.paths[path][method].summary
        delete openApiSchema.paths[path][method].summary
      }
    }
  }

  console.log(JSON.stringify(openApiSchema))
}
