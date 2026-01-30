import { HttpApiBuilder, HttpApiSwagger, HttpMiddleware, HttpServer } from "@effect/platform"
import { BunHttpServer } from "@effect/platform-bun"
import { Config, Effect, Layer, Logger } from "effect"
import { Database, DatabaseLive } from "./Db.js"
import { Sync, SyncLive } from "./Sync.js"
import { IndexerConfigService, IndexerConfigLive } from "./config.js"
import { IndexerApi, HttpApiLive } from "./Http.js"

// Log server address with docs URL
const withLogAddress = <A, E, R>(layer: Layer.Layer<A, E, R>): Layer.Layer<A, E, R | HttpServer.HttpServer> =>
  Layer.effectDiscard(
    HttpServer.addressFormattedWith((address) =>
      Effect.annotateLogs(Effect.logInfo(`Server listening`), {
        address,
        docs: `${address}/docs`,
        openapi: `${address}/openapi.json`,
      })
    )
  ).pipe(Layer.provideMerge(layer))

// Bun HTTP server
const ServerLive = BunHttpServer.layerConfig({
  port: Config.number("PORT").pipe(Config.withDefault(3002)),
})

// Start sync in background (fork so it doesn't block HTTP server)
const SyncStartLive = Layer.effectDiscard(
  Effect.gen(function* () {
    const sync = yield* Sync
    yield* Effect.forkDaemon(sync.start())
  })
)

// Logging
const LoggingLive = Logger.replace(
  Logger.defaultLogger,
  Logger.prettyLogger({ colors: true })
)

// Config layer
const ConfigLayer = IndexerConfigLive

// Database layer (needs config)
const DbLayer = DatabaseLive.pipe(Layer.provide(ConfigLayer))

// Sync layer (needs db and config)
const SyncLayer = SyncLive.pipe(
  Layer.provide(DbLayer),
  Layer.provide(ConfigLayer)
)

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
  Layer.provide(LoggingLive)
)

// Start everything
Layer.launch(HttpLive).pipe(Effect.runPromise).catch(console.error)
