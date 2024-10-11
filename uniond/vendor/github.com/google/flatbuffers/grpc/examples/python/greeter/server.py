from concurrent import futures
import sys		
import argparse
import grpc

sys.path.insert(0, '../../../../../flatbuffers/python')

import flatbuffers
from models import HelloReply, HelloRequest, greeter_grpc_fb

parser = argparse.ArgumentParser()
parser.add_argument("port", help="server on port", default=3000)

def build_reply(message):
    builder = flatbuffers.Builder()		
    ind = builder.CreateString(message)
    HelloReply.HelloReplyStart(builder)
    HelloReply.HelloReplyAddMessage(builder, ind)
    root = HelloReply.HelloReplyEnd(builder)
    builder.Finish(root)
    return bytes(builder.Output())

class GreeterServicer(greeter_grpc_fb.GreeterServicer):

    def __init__(self):
        self.greetings = ["Hi", "Hallo", "Ciao"]

    def SayHello(self, request, context):
        r = HelloRequest.HelloRequest().GetRootAs(request, 0)
        reply = "Unknown"
        if r.Name():
            reply = r.Name()
        return build_reply("welcome " + reply.decode('UTF-8'))

    def SayManyHellos(self, request, context):
        r = HelloRequest.HelloRequest().GetRootAs(request, 0)
        reply = "Unknown"
        if r.Name():
            reply = r.Name()

        for greeting in self.greetings:
            print(type(reply))
            yield build_reply(greeting + " " + reply.decode('UTF-8'))
        

def serve():
    args = parser.parse_args()
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    greeter_grpc_fb.add_GreeterServicer_to_server(
        GreeterServicer(), server
    )
    server.add_insecure_port('[::]:' + args.port)
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    serve()