import { assertEquals, assertExists } from "jsr:@std/assert"
import { afterAll, beforeAll, describe, it } from "jsr:@std/testing/bdd"
import { expect } from "jsr:@std/expect"
import { Application } from "jsr:@oak/oak/application"
import { Router } from "jsr:@oak/oak/router"

import { app } from "./main.ts"
import routeStaticFilesFrom from "./util/routeStaticFilesFrom.ts"

describe("Application", () => {
  let serverInfo: { baseUrl: string; abortController: AbortController }
  beforeAll(async () => {
    console.log("Starting server")
    serverInfo = await serve()
  })

  afterAll(() => {
    console.log("Shutting down server")
    serverInfo.abortController.abort()
  })

  it("can be created", () => {
    assertExists(app)
    assertEquals(app instanceof Application, true)
  })

  it("router accepts routes without throwing errors", () => {
    const router = new Router()
    app.use(router.routes())
    assertExists(router)
  })

  it("can configure static routes", () => {
    const staticFileMiddleware = routeStaticFilesFrom([
      `${Deno.cwd()}/client/dist`,
      `${Deno.cwd()}/client/public`
    ])
    app.use(staticFileMiddleware)
    assertExists(staticFileMiddleware)
  })

  it("can request home page from running server", async () => {
    const response = await fetch(serverInfo.baseUrl)
    const body = await response.text()

    assertEquals(response.status, 200)
    expect(body).toContain("<title>Vite + React + TS</title>")
  })
})

async function serve(abortController = new AbortController()) {
  let randomPort = 0

  app.listen({ port: randomPort, signal: abortController.signal })

  await new Promise<void>(resolve => {
    app.addEventListener("listen", ev => {
      randomPort = ev.port
      console.log(`Server running on http://localhost:${ev.port}`)
      resolve()
    })
  })

  return {
    baseUrl: `http://localhost:${randomPort}`,
    abortController: abortController
  }
}
