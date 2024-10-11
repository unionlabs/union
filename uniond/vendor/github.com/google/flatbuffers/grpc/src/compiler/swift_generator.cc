/*
 * Copyright 2020 Google Inc. All rights reserved.
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

/*
 * NOTE: The following implementation is a translation for the Swift-grpc
 * generator since flatbuffers doesnt allow plugins for now. if an issue arises
 * please open an issue in the flatbuffers repository. This file should always
 * be maintained according to the Swift-grpc repository
 */
#include <map>
#include <sstream>

#include "flatbuffers/util.h"
#include "src/compiler/schema_interface.h"
#include "src/compiler/swift_generator.h"

namespace grpc_swift_generator {
namespace {

static std::string WrapInNameSpace(const std::vector<std::string> &components,
                            const grpc::string &name) {
  std::string qualified_name;
  for (auto it = components.begin(); it != components.end(); ++it)
    qualified_name += *it + "_";
  return qualified_name + name;
}

static grpc::string GenerateMessage(const std::vector<std::string> &components,
                             const grpc::string &name) {
  return "Message<" + WrapInNameSpace(components, name) + ">";
}

// MARK: - Client

static void GenerateClientFuncName(const grpc_generator::Method *method,
                            grpc_generator::Printer *printer,
                            std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  if (method->NoStreaming()) {
    printer->Print(vars,
                   "  $GenAccess$func $MethodName$(\n"
                   "    _ request: $Input$\n"
                   "    , callOptions: CallOptions?$isNil$\n"
                   "  ) -> UnaryCall<$Input$, $Output$>");
    return;
  }

  if (method->ServerStreaming()) {
    printer->Print(vars,
                   "  $GenAccess$func $MethodName$(\n"
                   "    _ request: $Input$\n"
                   "    , callOptions: CallOptions?$isNil$,\n"
                   "    handler: @escaping ($Output$) -> Void\n"
                   "  ) -> ServerStreamingCall<$Input$, $Output$>");
    return;
  }

  if (method->ClientStreaming()) {
    printer->Print(vars,
                   "  $GenAccess$func $MethodName$(\n"
                   "    callOptions: CallOptions?$isNil$\n"
                   "  ) -> ClientStreamingCall<$Input$, $Output$>");
    return;
  }

  printer->Print(vars,
                 "  $GenAccess$func $MethodName$(\n"
                 "    callOptions: CallOptions?$isNil$,\n"
                 "    handler: @escaping ($Output$ ) -> Void\n"
                 "  ) -> BidirectionalStreamingCall<$Input$, $Output$>");
}

static void GenerateClientFuncBody(const grpc_generator::Method *method,
                            grpc_generator::Printer *printer,
                            std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  vars["Interceptor"] =
      "interceptors: self.interceptors?.make$MethodName$Interceptors() ?? []";
  if (method->NoStreaming()) {
    printer->Print(
        vars,
        "    return self.makeUnaryCall(\n"
        "      path: \"/$PATH$$ServiceName$/$MethodName$\",\n"
        "      request: request,\n"
        "      callOptions: callOptions ?? self.defaultCallOptions,\n"
        "      $Interceptor$\n"
        "    )\n");
    return;
  }

  if (method->ServerStreaming()) {
    printer->Print(
        vars,
        "    return self.makeServerStreamingCall(\n"
        "      path: \"/$PATH$$ServiceName$/$MethodName$\",\n"
        "      request: request,\n"
        "      callOptions: callOptions ?? self.defaultCallOptions,\n"
        "      $Interceptor$,\n"
        "      handler: handler\n"
        "    )\n");
    return;
  }

  if (method->ClientStreaming()) {
    printer->Print(
        vars,
        "    return self.makeClientStreamingCall(\n"
        "      path: \"/$PATH$$ServiceName$/$MethodName$\",\n"
        "      callOptions: callOptions ?? self.defaultCallOptions,\n"
        "      $Interceptor$\n"
        "    )\n");
    return;
  }
  printer->Print(vars,
                 "    return self.makeBidirectionalStreamingCall(\n"
                 "      path: \"/$PATH$$ServiceName$/$MethodName$\",\n"
                 "      callOptions: callOptions ?? self.defaultCallOptions,\n"
                 "      $Interceptor$,\n"
                 "      handler: handler\n"
                 "    )\n");
}

void GenerateClientProtocol(const grpc_generator::Service *service,
                            grpc_generator::Printer *printer,
                            std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  printer->Print(
      vars,
      "$ACCESS$ protocol $ServiceQualifiedName$ClientProtocol: GRPCClient {");
  printer->Print("\n\n");
  printer->Print("  var serviceName: String { get }");
  printer->Print("\n\n");
  printer->Print(
      vars,
      "  var interceptors: "
      "$ServiceQualifiedName$ClientInterceptorFactoryProtocol? { get }");
  printer->Print("\n\n");

  vars["GenAccess"] = "";
  for (auto it = 0; it < service->method_count(); it++) {
    auto method = service->method(it);
    vars["Input"] = GenerateMessage(method->get_input_namespace_parts(),
                                    method->get_input_type_name());
    vars["Output"] = GenerateMessage(method->get_output_namespace_parts(),
                                     method->get_output_type_name());
    vars["MethodName"] = method->name();
    vars["isNil"] = "";
    GenerateClientFuncName(method.get(), &*printer, &vars);
    printer->Print("\n\n");
  }
  printer->Print("}\n\n");

  printer->Print(vars, "extension $ServiceQualifiedName$ClientProtocol {");
  printer->Print("\n\n");
  printer->Print(vars,
                 "  $ACCESS$ var serviceName: String { "
                 "\"$PATH$$ServiceName$\" }\n");

  vars["GenAccess"] = service->is_internal() ? "internal " : "public ";
  for (auto it = 0; it < service->method_count(); it++) {
    auto method = service->method(it);
    vars["Input"] = GenerateMessage(method->get_input_namespace_parts(),
                                    method->get_input_type_name());
    vars["Output"] = GenerateMessage(method->get_output_namespace_parts(),
                                     method->get_output_type_name());
    vars["MethodName"] = method->name();
    vars["isNil"] = " = nil";
    printer->Print("\n");
    GenerateClientFuncName(method.get(), &*printer, &vars);
    printer->Print(" {\n");
    GenerateClientFuncBody(method.get(), &*printer, &vars);
    printer->Print("  }\n");
  }
  printer->Print("}\n\n");

  printer->Print(vars,
                 "$ACCESS$ protocol "
                 "$ServiceQualifiedName$ClientInterceptorFactoryProtocol {\n");

  for (auto it = 0; it < service->method_count(); it++) {
    auto method = service->method(it);
    vars["Input"] = GenerateMessage(method->get_input_namespace_parts(),
                                    method->get_input_type_name());
    vars["Output"] = GenerateMessage(method->get_output_namespace_parts(),
                                     method->get_output_type_name());
    vars["MethodName"] = method->name();
    printer->Print(
        vars,
        "  /// - Returns: Interceptors to use when invoking '$MethodName$'.\n");
    printer->Print(vars,
                   "  func make$MethodName$Interceptors() -> "
                   "[ClientInterceptor<$Input$, $Output$>]\n\n");
  }
  printer->Print("}\n\n");
}

void GenerateClientClass(grpc_generator::Printer *printer,
                         std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  printer->Print(vars,
                 "$ACCESS$ final class $ServiceQualifiedName$ServiceClient: "
                 "$ServiceQualifiedName$ClientProtocol {\n");
  printer->Print(vars, "  $ACCESS$ let channel: GRPCChannel\n");
  printer->Print(vars, "  $ACCESS$ var defaultCallOptions: CallOptions\n");
  printer->Print(vars,
                 "  $ACCESS$ var interceptors: "
                 "$ServiceQualifiedName$ClientInterceptorFactoryProtocol?\n");
  printer->Print("\n");
  printer->Print(
      vars,
      "  $ACCESS$ init(\n"
      "    channel: GRPCChannel,\n"
      "    defaultCallOptions: CallOptions = CallOptions(),\n"
      "    interceptors: "
      "$ServiceQualifiedName$ClientInterceptorFactoryProtocol? = nil\n"
      "  ) {\n");
  printer->Print("    self.channel = channel\n");
  printer->Print("    self.defaultCallOptions = defaultCallOptions\n");
  printer->Print("    self.interceptors = interceptors\n");
  printer->Print("  }");
  printer->Print("\n");
  printer->Print("}\n");
}

// MARK: - Server

grpc::string GenerateServerFuncName(const grpc_generator::Method *method) {
  if (method->NoStreaming()) {
    return "func $MethodName$(request: $Input$"
           ", context: StatusOnlyCallContext) -> EventLoopFuture<$Output$>";
  }

  if (method->ClientStreaming()) {
    return "func $MethodName$(context: UnaryResponseCallContext<$Output$>) -> "
           "EventLoopFuture<(StreamEvent<$Input$"
           ">) -> Void>";
  }

  if (method->ServerStreaming()) {
    return "func $MethodName$(request: $Input$"
           ", context: StreamingResponseCallContext<$Output$>) -> "
           "EventLoopFuture<GRPCStatus>";
  }
  return "func $MethodName$(context: StreamingResponseCallContext<$Output$>) "
         "-> EventLoopFuture<(StreamEvent<$Input$>) -> Void>";
}

grpc::string GenerateServerExtensionBody(const grpc_generator::Method *method) {
  grpc::string start = "    case \"$MethodName$\":\n    ";
  grpc::string interceptors =
      "      interceptors: self.interceptors?.make$MethodName$Interceptors() "
      "?? [],\n";
  if (method->NoStreaming()) {
    return start +
           "return UnaryServerHandler(\n"
           "      context: context,\n"
           "      requestDeserializer: GRPCPayloadDeserializer<$Input$>(),\n"
           "      responseSerializer: GRPCPayloadSerializer<$Output$>(),\n" +
           interceptors +
           "      userFunction: self.$MethodName$(request:context:))\n";
  }
  if (method->ServerStreaming()) {
    return start +
           "return ServerStreamingServerHandler(\n"
           "      context: context,\n"
           "      requestDeserializer: GRPCPayloadDeserializer<$Input$>(),\n"
           "      responseSerializer: GRPCPayloadSerializer<$Output$>(),\n" +
           interceptors +
           "      userFunction: self.$MethodName$(request:context:))\n";
  }
  if (method->ClientStreaming()) {
    return start +
           "return ClientStreamingServerHandler(\n"
           "      context: context,\n"
           "      requestDeserializer: GRPCPayloadDeserializer<$Input$>(),\n"
           "      responseSerializer: GRPCPayloadSerializer<$Output$>(),\n" +
           interceptors +
           "      observerFactory: self.$MethodName$(context:))\n";
  }
  if (method->BidiStreaming()) {
    return start +
           "return BidirectionalStreamingServerHandler(\n"
           "      context: context,\n"
           "      requestDeserializer: GRPCPayloadDeserializer<$Input$>(),\n"
           "      responseSerializer: GRPCPayloadSerializer<$Output$>(),\n" +
           interceptors +
           "      observerFactory: self.$MethodName$(context:))\n";
  }
  return "";
}

void GenerateServerProtocol(const grpc_generator::Service *service,
                            grpc_generator::Printer *printer,
                            std::map<grpc::string, grpc::string> *dictonary) {
  auto vars = *dictonary;
  printer->Print(vars,
                 "$ACCESS$ protocol $ServiceQualifiedName$Provider: "
                 "CallHandlerProvider {\n");
  printer->Print(
      vars,
      "  var interceptors: "
      "$ServiceQualifiedName$ServerInterceptorFactoryProtocol? { get }\n");
  for (auto it = 0; it < service->method_count(); it++) {
    auto method = service->method(it);
    vars["Input"] = GenerateMessage(method->get_input_namespace_parts(),
                                    method->get_input_type_name());
    vars["Output"] = GenerateMessage(method->get_output_namespace_parts(),
                                     method->get_output_type_name());
    vars["MethodName"] = method->name();
    printer->Print("  ");
    auto func = GenerateServerFuncName(method.get());
    printer->Print(vars, func.c_str());
    printer->Print("\n");
  }
  printer->Print("}\n\n");

  printer->Print(vars, "$ACCESS$ extension $ServiceQualifiedName$Provider {\n");
  printer->Print("\n");
  printer->Print(vars,
                 "  var serviceName: Substring { return "
                 "\"$PATH$$ServiceName$\" }\n");
  printer->Print("\n");
  printer->Print(
      "  func handle(method name: Substring, context: "
      "CallHandlerContext) -> GRPCServerHandlerProtocol? {\n");
  printer->Print("    switch name {\n");
  for (auto it = 0; it < service->method_count(); it++) {
    auto method = service->method(it);
    vars["Input"] = GenerateMessage(method->get_input_namespace_parts(),
                                    method->get_input_type_name());
    vars["Output"] = GenerateMessage(method->get_output_namespace_parts(),
                                     method->get_output_type_name());
    vars["MethodName"] = method->name();
    auto body = GenerateServerExtensionBody(method.get());
    printer->Print(vars, body.c_str());
    printer->Print("\n");
  }
  printer->Print("    default: return nil;\n");
  printer->Print("    }\n");
  printer->Print("  }\n\n");
  printer->Print("}\n\n");

  printer->Print(vars,
                 "$ACCESS$ protocol "
                 "$ServiceQualifiedName$ServerInterceptorFactoryProtocol {\n");
  for (auto it = 0; it < service->method_count(); it++) {
    auto method = service->method(it);
    vars["Input"] = GenerateMessage(method->get_input_namespace_parts(),
                                    method->get_input_type_name());
    vars["Output"] = GenerateMessage(method->get_output_namespace_parts(),
                                     method->get_output_type_name());
    vars["MethodName"] = method->name();
    printer->Print(
        vars,
        "  /// - Returns: Interceptors to use when handling '$MethodName$'.\n"
        "  ///   Defaults to calling `self.makeInterceptors()`.\n");
    printer->Print(vars,
                   "  func make$MethodName$Interceptors() -> "
                   "[ServerInterceptor<$Input$, $Output$>]\n\n");
  }
  printer->Print("}");
}
} // namespace

grpc::string Generate(grpc_generator::File *file,
                      const grpc_generator::Service *service) {
  grpc::string output;
  std::map<grpc::string, grpc::string> vars;
  vars["PATH"] = file->package();
  if (!file->package().empty()) { vars["PATH"].append("."); }
  vars["ServiceQualifiedName"] =
      WrapInNameSpace(service->namespace_parts(), service->name());
  vars["ServiceName"] = service->name();
  vars["ACCESS"] = service->is_internal() ? "internal" : "public";
  auto printer = file->CreatePrinter(&output);
  printer->Print(
      vars,
      "/// Usage: instantiate $ServiceQualifiedName$ServiceClient, then call "
      "methods of this protocol to make API calls.\n");
  GenerateClientProtocol(service, &*printer, &vars);
  GenerateClientClass(&*printer, &vars);
  printer->Print("\n");
  GenerateServerProtocol(service, &*printer, &vars);
  return output;
}

grpc::string GenerateHeader() {
  grpc::string code;
  code +=
      "/// The following code is generated by the Flatbuffers library which "
      "might not be in sync with grpc-swift\n";
  code +=
      "/// in case of an issue please open github issue, though it would be "
      "maintained\n";
  code += "\n";
  code += "// swiftlint:disable all\n";
  code += "// swiftformat:disable all\n";
  code += "\n";
  code += "import Foundation\n";
  code += "import GRPC\n";
  code += "import NIO\n";
  code += "import NIOHTTP1\n";
  code += "import FlatBuffers\n";
  code += "\n";
  code +=
      "public protocol GRPCFlatBufPayload: GRPCPayload, FlatBufferGRPCMessage "
      "{}\n";

  code += "public extension GRPCFlatBufPayload {\n";
  code += "  init(serializedByteBuffer: inout NIO.ByteBuffer) throws {\n";
  code +=
      "    self.init(byteBuffer: FlatBuffers.ByteBuffer(contiguousBytes: "
      "serializedByteBuffer.readableBytesView, count: "
      "serializedByteBuffer.readableBytes))\n";
  code += "  }\n";

  code += "  func serialize(into buffer: inout NIO.ByteBuffer) throws {\n";
  code +=
      "    let buf = UnsafeRawBufferPointer(start: self.rawPointer, count: "
      "Int(self.size))\n";
  code += "    buffer.writeBytes(buf)\n";
  code += "  }\n";
  code += "}\n";
  code += "extension Message: GRPCFlatBufPayload {}\n";
  return code;
}
}  // namespace grpc_swift_generator
