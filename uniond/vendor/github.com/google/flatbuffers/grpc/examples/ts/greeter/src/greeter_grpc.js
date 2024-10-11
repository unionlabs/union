// Generated GRPC code for FlatBuffers TS *** DO NOT EDIT ***
import * as flatbuffers from 'flatbuffers';
import { HelloReply as models_HelloReply } from './models/hello-reply';
import { HelloRequest as models_HelloRequest } from './models/hello-request';

var grpc = require('@grpc/grpc-js');

function serialize_models_HelloReply(buffer_args) {
  if (!(buffer_args instanceof models_HelloReply)) {
    throw new Error('Expected argument of type HelloReply');
  }
  return Buffer.from(buffer_args.serialize());
}

function deserialize_models_HelloReply(buffer) {
  return models_HelloReply.getRootAsHelloReply(new flatbuffers.ByteBuffer(buffer))
}


function serialize_models_HelloRequest(buffer_args) {
  if (!(buffer_args instanceof models_HelloRequest)) {
    throw new Error('Expected argument of type HelloRequest');
  }
  return Buffer.from(buffer_args.serialize());
}

function deserialize_models_HelloRequest(buffer) {
  return models_HelloRequest.getRootAsHelloRequest(new flatbuffers.ByteBuffer(buffer))
}


var GreeterService = exports.GreeterService = {
  SayHello: {
    path: '/models.Greeter/SayHello',
    requestStream: false,
    responseStream: false,
    requestType: flatbuffers.ByteBuffer,
    responseType: models_HelloReply,
    requestSerialize: serialize_models_HelloRequest,
    requestDeserialize: deserialize_models_HelloRequest,
    responseSerialize: serialize_models_HelloReply,
    responseDeserialize: deserialize_models_HelloReply,
  },
  SayManyHellos: {
    path: '/models.Greeter/SayManyHellos',
    requestStream: false,
    responseStream: true,
    requestType: flatbuffers.ByteBuffer,
    responseType: models_HelloReply,
    requestSerialize: serialize_models_HelloRequest,
    requestDeserialize: deserialize_models_HelloRequest,
    responseSerialize: serialize_models_HelloReply,
    responseDeserialize: deserialize_models_HelloReply,
  },
};
exports.GreeterClient = grpc.makeGenericClientConstructor(GreeterService);
