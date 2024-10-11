/*
 * Copyright 2023 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import FlatBuffers
import GRPC
import Logging
import Model
import NIO

class Greeter: models_GreeterProvider {

  var interceptors: models_GreeterServerInterceptorFactoryProtocol?

  let greetings: [String]

  init() {
    greetings = ["Hi", "Hallo", "Ciao"]
  }

  func SayHello(
    request: Message<models_HelloRequest>,
    context: StatusOnlyCallContext)
    -> EventLoopFuture<Message<models_HelloReply>>
  {
    let recipient = request.object.name ?? "Stranger"

    var builder = FlatBufferBuilder()
    let off = builder.create(string: "Hello \(recipient)")
    let root = models_HelloReply.createHelloReply(&builder, messageOffset: off)
    builder.finish(offset: root)
    return context.eventLoop
      .makeSucceededFuture(Message<models_HelloReply>(builder: &builder))
  }

  func SayManyHellos(
    request: Message<models_HelloRequest>,
    context: StreamingResponseCallContext<Message<models_HelloReply>>)
    -> EventLoopFuture<GRPCStatus>
  {
    for name in greetings {
      var builder = FlatBufferBuilder()
      let off = builder
        .create(string: "\(name) \(request.object.name ?? "Unknown")")
      let root = models_HelloReply.createHelloReply(
        &builder,
        messageOffset: off)
      builder.finish(offset: root)
      _ = context.sendResponse(Message<models_HelloReply>(builder: &builder))
    }
    return context.eventLoop.makeSucceededFuture(.ok)
  }
}

// Quieten the logs.
LoggingSystem.bootstrap {
  var handler = StreamLogHandler.standardOutput(label: $0)
  handler.logLevel = .critical
  return handler
}

let group = MultiThreadedEventLoopGroup(numberOfThreads: 1)
defer {
  try! group.syncShutdownGracefully()
}

// Create some configuration for the server:
let configuration = Server.Configuration(
  target: .hostAndPort("localhost", 0),
  eventLoopGroup: group,
  serviceProviders: [Greeter()])

// Start the server and print its address once it has started.
let server = Server.start(configuration: configuration)
server.map {
  $0.channel.localAddress
}.whenSuccess { address in
  print("server started on port \(address!.port!)")
}

// Wait on the server's `onClose` future to stop the program from exiting.
_ = try server.flatMap {
  $0.onClose
}.wait()
