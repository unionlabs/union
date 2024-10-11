/*
 *
 * Copyright 2015 gRPC authors.
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
 *
 */

#include <map>
#include <sstream>

#include "flatbuffers/util.h"
#include "src/compiler/python_generator.h"

namespace grpc_python_generator {
namespace {

static grpc::string GenerateMethodType(const grpc_generator::Method *method) {

  if (method->NoStreaming())
    return "unary_unary";

  if (method->ServerStreaming())
    return "unary_stream";

  if (method->ClientStreaming())
    return "stream_unary";

  return "stream_stream";
}

grpc::string GenerateMethodInput(const grpc_generator::Method *method) {

  if (method->NoStreaming() || method->ServerStreaming())
    return "self, request, context";

  return "self, request_iterator, context";
}

void GenerateStub(const grpc_generator::Service *service,
                  grpc_generator::Printer *printer,
                  std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  printer->Print(vars, "class $ServiceName$Stub(object):\n");
  printer->Indent();
  printer->Print("\"\"\" Interface exported by the server. \"\"\"");
  printer->Print("\n\n");
  printer->Print("def __init__(self, channel):\n");
  printer->Indent();
  printer->Print("\"\"\" Constructor. \n\n");
  printer->Print("Args: \n");
  printer->Print("channel: A grpc.Channel. \n");
  printer->Print("\"\"\"\n\n");

  for (int j = 0; j < service->method_count(); j++) {
    auto method = service->method(j);
    vars["MethodName"] = method->name();
    vars["MethodType"] = GenerateMethodType(&*method);
    printer->Print(vars, "self.$MethodName$ = channel.$MethodType$(\n");
    printer->Indent();
    printer->Print(vars, "\"/$PATH$$ServiceName$/$MethodName$\"\n");
    printer->Print(")\n");
    printer->Outdent();
    printer->Print("\n");
  }
  printer->Outdent();
  printer->Outdent();
  printer->Print("\n");
}

void GenerateServicer(const grpc_generator::Service *service,
                      grpc_generator::Printer *printer,
                      std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  printer->Print(vars, "class $ServiceName$Servicer(object):\n");
  printer->Indent();
  printer->Print("\"\"\" Interface exported by the server. \"\"\"");
  printer->Print("\n\n");

  for (int j = 0; j < service->method_count(); j++) {
    auto method = service->method(j);
    vars["MethodName"] = method->name();
    vars["MethodInput"] = GenerateMethodInput(&*method);
    printer->Print(vars, "def $MethodName$($MethodInput$):\n");
    printer->Indent();
    printer->Print("context.set_code(grpc.StatusCode.UNIMPLEMENTED)\n");
    printer->Print("context.set_details('Method not implemented!')\n");
    printer->Print("raise NotImplementedError('Method not implemented!')\n");
    printer->Outdent();
    printer->Print("\n\n");
  }
  printer->Outdent();
  printer->Print("\n");

}

void GenerateRegister(const grpc_generator::Service *service,
                      grpc_generator::Printer *printer,
                      std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  printer->Print(vars, "def add_$ServiceName$Servicer_to_server(servicer, server):\n");
  printer->Indent();
  printer->Print("rpc_method_handlers = {\n");
  printer->Indent();
  for (int j = 0; j < service->method_count(); j++) {
    auto method = service->method(j);
    vars["MethodName"] = method->name();
    vars["MethodType"] = GenerateMethodType(&*method);
    printer->Print(vars, "'$MethodName$': grpc.$MethodType$_rpc_method_handler(\n");
    printer->Indent();
    printer->Print(vars, "servicer.$MethodName$\n");
    printer->Outdent();
    printer->Print("),\n");
  }
  printer->Outdent();
  printer->Print("}\n");
  printer->Print(vars, "generic_handler = grpc.method_handlers_generic_handler(\n");
  printer->Indent();
  printer->Print(vars, "'$PATH$$ServiceName$', rpc_method_handlers)\n");
  printer->Outdent();
  printer->Print("server.add_generic_rpc_handlers((generic_handler,))");
  printer->Outdent();
  printer->Print("\n");
}
} // namespace

grpc::string Generate(grpc_generator::File *file,
                      const grpc_generator::Service *service) {
  grpc::string output;
  std::map<grpc::string, grpc::string> vars;
  vars["PATH"] = file->package();
  if (!file->package().empty()) { vars["PATH"].append("."); }
  vars["ServiceName"] = service->name();
  auto printer = file->CreatePrinter(&output);
  GenerateStub(service, &*printer, &vars);
  GenerateServicer(service, &*printer, &vars);
  GenerateRegister(service, &*printer, &vars);
  return output;
}

}  // namespace grpc_python_generator
