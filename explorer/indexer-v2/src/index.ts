import { HttpApiBuilder, HttpApiSwagger, HttpMiddleware, HttpServer } from "@effect/platform"
import { BunHttpServer } from "@effect/platform-bun"
import { Config, Effect, Layer, Logger } from "effect"
import { IndexerConfigLive } from "./config.js"
import { DatabaseLive } from "./Db.js"
import { HttpApiLive } from "./Http.js"
import { Sync, SyncLive } from "./Sync.js"

// Log server address with docs URL
const withLogAddress = <A, E, R>(
  layer: Layer.Layer<A, E, R>,
): Layer.Layer<A, E, R | HttpServer.HttpServer> =>
  Layer.effectDiscard(
    HttpServer.addressFormattedWith((address) =>
      Effect.annotateLogs(Effect.logInfo(`Server listening`), {
        address,
        docs: `${address}/docs`,
        openapi: `${address}/openapi.json`,
      })
    ),
  ).pipe(Layer.provideMerge(layer))

// Bun HTTP server
const ServerLive = BunHttpServer.layerConfig(
  Config.all({
    port: Config.number("PORT").pipe(Config.withDefault(3002)),
  }),
)

// Start sync in background (fork so it doesn't block HTTP server)
const SyncStartLive = Layer.effectDiscard(
  Effect.gen(function*() {
    const sync = yield* Sync
    yield* Effect.forkDaemon(sync.start())
  }),
)

// Logging
const LoggingLive = Logger.replace(
  Logger.defaultLogger,
  Logger.prettyLogger({ colors: true }),
)

// Config layer
const ConfigLayer = IndexerConfigLive

// Database layer (needs config) - scoped for proper cleanup
const DbLayer = DatabaseLive.pipe(Layer.provide(ConfigLayer))

// Sync layer (needs db and config)
const SyncLayer = SyncLive.pipe(Layer.provide(DbLayer), Layer.provide(ConfigLayer))

// Services needed by HTTP handlers
const ServicesLive = Layer.mergeAll(DbLayer, SyncLayer, ConfigLayer)

// Sync startup layer
const SyncStartupLive = SyncStartLive.pipe(Layer.provide(ServicesLive))

// Full HTTP server with Swagger
const HttpLive = HttpApiBuilder.serve(HttpMiddleware.logger).pipe(
  Layer.provide(HttpApiSwagger.layer()),
  Layer.provide(HttpApiBuilder.middlewareOpenApi()),
  Layer.provide(HttpApiBuilder.middlewareCors()),
  Layer.provide(HttpApiLive),
  Layer.provide(ServicesLive),
  Layer.provide(SyncStartupLive),
  withLogAddress,
  Layer.provide(ServerLive),
  Layer.provide(LoggingLive),
)

// Graceful shutdown handler
const main = Effect.gen(function*() {
  yield* Effect.logInfo("Starting Cosmos Indexer...")

  // Add finalizer for graceful shutdown
  yield* Effect.addFinalizer(() =>
    Effect.gen(function*() {
      yield* Effect.logInfo("Shutting down gracefully...")
    })
  )

  // Launch the HTTP server
  yield* Layer.launch(HttpLive)
})

// Run with proper scope management
Effect.runPromise(
  Effect.scoped(main).pipe(
    Effect.catchAllDefect((defect) =>
      Effect.gen(function*() {
        yield* Effect.logError(`Fatal error: ${defect}`)
        process.exit(1)
      })
    ),
  ),
).catch((error) => {
  console.error("Startup failed:", error)
  process.exit(1)
})
