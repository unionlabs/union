import * as grpc from '@grpc/grpc-js';
import * as flatbuffers from 'flatbuffers';
import { HelloReply } from './models/hello-reply';
import { HelloRequest } from './models/hello-request';
import { IGreeterServer, GreeterService } from './greeter_grpc';

const greeter: IGreeterServer = {
    SayHello(call: grpc.ServerUnaryCall<HelloRequest, HelloReply>, callback: grpc.sendUnaryData<HelloReply>): void {
        console.log(`SayHello ${call.request.name()}`);
        const builder = new flatbuffers.Builder();
        const offset = builder.createString(`welcome ${call.request.name()}`);
        const root = HelloReply.createHelloReply(builder, offset);
        builder.finish(root);
        callback(null, HelloReply.getRootAsHelloReply(new flatbuffers.ByteBuffer(builder.asUint8Array())));
    },
    async SayManyHellos(call: grpc.ServerWritableStream<HelloRequest, HelloReply>): Promise<void> {
        const name = call.request.name();
        console.log(`${call.request.name()} saying hi in different langagues`);
        ['Hi', 'Hallo', 'Ciao'].forEach(element => {
            const builder = new flatbuffers.Builder();
            const offset = builder.createString(`${element} ${name}`);
            const root = HelloReply.createHelloReply(builder, offset);
            builder.finish(root);
            call.write(HelloReply.getRootAsHelloReply(new flatbuffers.ByteBuffer(builder.asUint8Array())))
        });
        call.end();
    }
}

function serve(): void {
    const PORT = 3000;
    const server = new grpc.Server();
    server.addService(GreeterService, greeter);
    console.log(`Listening on ${PORT}`);
    server.bindAsync(
        `localhost:${PORT}`,
        grpc.ServerCredentials.createInsecure(),
        (err: Error | null, port: number) => {
          if (err) {
            console.error(`Server error: ${err.message}`);
          } else {
            console.log(`Server bound on port: ${port}`);
            server.start();
          }
        }
      );
}

serve();