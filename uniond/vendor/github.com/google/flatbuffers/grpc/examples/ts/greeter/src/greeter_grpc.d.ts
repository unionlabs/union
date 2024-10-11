// Generated GRPC code for FlatBuffers TS *** DO NOT EDIT ***
import * as flatbuffers from 'flatbuffers';
import { HelloReply as models_HelloReply } from './models/hello-reply';
import { HelloRequest as models_HelloRequest } from './models/hello-request';

import * as grpc from '@grpc/grpc-js';

interface IGreeterService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  SayHello: IGreeterService_ISayHello;
  SayManyHellos: IGreeterService_ISayManyHellos;
}
interface IGreeterService_ISayHello extends grpc.MethodDefinition<models_HelloRequest, models_HelloReply> {
  path: string; // /models.Greeter/SayHello
  requestStream: boolean; // false
  responseStream: boolean; // false
  requestSerialize: grpc.serialize<models_HelloRequest>;
  requestDeserialize: grpc.deserialize<models_HelloRequest>;
  responseSerialize: grpc.serialize<models_HelloReply>;
  responseDeserialize: grpc.deserialize<models_HelloReply>;
}

interface IGreeterService_ISayManyHellos extends grpc.MethodDefinition<models_HelloRequest, models_HelloReply> {
  path: string; // /models.Greeter/SayManyHellos
  requestStream: boolean; // false
  responseStream: boolean; // true
  requestSerialize: grpc.serialize<models_HelloRequest>;
  requestDeserialize: grpc.deserialize<models_HelloRequest>;
  responseSerialize: grpc.serialize<models_HelloReply>;
  responseDeserialize: grpc.deserialize<models_HelloReply>;
}


export const GreeterService: IGreeterService;

export interface IGreeterServer extends grpc.UntypedServiceImplementation {
  SayHello: grpc.handleUnaryCall<models_HelloRequest, models_HelloReply>;
  SayManyHellos: grpc.handleServerStreamingCall<models_HelloRequest, models_HelloReply>;
}

export interface IGreeterClient {
  SayHello(request: models_HelloRequest, callback: (error: grpc.ServiceError | null, response: models_HelloReply) => void): grpc.ClientUnaryCall;
  SayHello(request: models_HelloRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: models_HelloReply) => void): grpc.ClientUnaryCall;
  SayHello(request: models_HelloRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: models_HelloReply) => void): grpc.ClientUnaryCall;
  SayManyHellos(request: models_HelloRequest, metadata: grpc.Metadata): grpc.ClientReadableStream<models_HelloReply>;
  SayManyHellos(request: models_HelloRequest, options: Partial<grpc.CallOptions>): grpc.ClientReadableStream<models_HelloReply>;
}

export class GreeterClient extends grpc.Client implements IGreeterClient {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  public SayHello(request: models_HelloRequest, callback: (error: grpc.ServiceError | null, response: models_HelloReply) => void): grpc.ClientUnaryCall;
  public SayHello(request: models_HelloRequest, metadata: grpc.Metadata, callback: (error: grpc.ServiceError | null, response: models_HelloReply) => void): grpc.ClientUnaryCall;
  public SayHello(request: models_HelloRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: grpc.ServiceError | null, response: models_HelloReply) => void): grpc.ClientUnaryCall;
  public SayManyHellos(request: models_HelloRequest, metadata: grpc.Metadata): grpc.ClientReadableStream<models_HelloReply>;
  public SayManyHellos(request: models_HelloRequest, options: Partial<grpc.CallOptions>): grpc.ClientReadableStream<models_HelloReply>;
}

