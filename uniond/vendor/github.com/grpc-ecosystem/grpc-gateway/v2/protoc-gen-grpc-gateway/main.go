// Command protoc-gen-grpc-gateway is a plugin for Google protocol buffer
// compiler to generate a reverse-proxy, which converts incoming RESTful
// HTTP/1 requests gRPC invocation.
// You rarely need to run this program directly. Instead, put this program
// into your $PATH with a name "protoc-gen-grpc-gateway" and run
//
//	protoc --grpc-gateway_out=output_directory path/to/input.proto
//
// See README.md for more details.
package main

import (
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/golang/glog"
	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/codegenerator"
	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/descriptor"
	"github.com/grpc-ecosystem/grpc-gateway/v2/protoc-gen-grpc-gateway/internal/gengateway"
	"google.golang.org/protobuf/compiler/protogen"
)

var (
	registerFuncSuffix         = flag.String("register_func_suffix", "Handler", "used to construct names of generated Register*<Suffix> methods.")
	useRequestContext          = flag.Bool("request_context", true, "determine whether to use http.Request's context or not")
	allowDeleteBody            = flag.Bool("allow_delete_body", false, "unless set, HTTP DELETE methods may not have a body")
	grpcAPIConfiguration       = flag.String("grpc_api_configuration", "", "path to gRPC API Configuration in YAML format")
	_                          = flag.Bool("allow_repeated_fields_in_body", true, "allows to use repeated field in `body` and `response_body` field of `google.api.http` annotation option. DEPRECATED: the value is ignored and always behaves as `true`.")
	repeatedPathParamSeparator = flag.String("repeated_path_param_separator", "csv", "configures how repeated fields should be split. Allowed values are `csv`, `pipes`, `ssv` and `tsv`.")
	allowPatchFeature          = flag.Bool("allow_patch_feature", true, "determines whether to use PATCH feature involving update masks (using google.protobuf.FieldMask).")
	omitPackageDoc             = flag.Bool("omit_package_doc", false, "if true, no package comment will be included in the generated code")
	standalone                 = flag.Bool("standalone", false, "generates a standalone gateway package, which imports the target service package")
	versionFlag                = flag.Bool("version", false, "print the current version")
	warnOnUnboundMethods       = flag.Bool("warn_on_unbound_methods", false, "emit a warning message if an RPC method has no HttpRule annotation")
	generateUnboundMethods     = flag.Bool("generate_unbound_methods", false, "generate proxy methods even for RPC methods that have no HttpRule annotation")
)

// Variables set by goreleaser at build time
var (
	version = "dev"
	commit  = "unknown"
	date    = "unknown"
)

func main() {
	flag.Parse()
	defer glog.Flush()

	if *versionFlag {
		fmt.Printf("Version %v, commit %v, built at %v\n", version, commit, date)
		os.Exit(0)
	}

	protogen.Options{
		ParamFunc: flag.CommandLine.Set,
	}.Run(func(gen *protogen.Plugin) error {
		reg := descriptor.NewRegistry()

		if err := applyFlags(reg); err != nil {
			return err
		}

		codegenerator.SetSupportedFeaturesOnPluginGen(gen)

		generator := gengateway.New(reg, *useRequestContext, *registerFuncSuffix, *allowPatchFeature, *standalone)

		glog.V(1).Infof("Parsing code generator request")

		if err := reg.LoadFromPlugin(gen); err != nil {
			return err
		}

		unboundHTTPRules := reg.UnboundExternalHTTPRules()
		if len(unboundHTTPRules) != 0 {
			return fmt.Errorf("HTTP rules without a matching selector: %s", strings.Join(unboundHTTPRules, ", "))
		}

		targets := make([]*descriptor.File, 0, len(gen.Request.FileToGenerate))
		for _, target := range gen.Request.FileToGenerate {
			f, err := reg.LookupFile(target)
			if err != nil {
				return err
			}
			targets = append(targets, f)
		}

		files, err := generator.Generate(targets)
		for _, f := range files {
			glog.V(1).Infof("NewGeneratedFile %q in %s", f.GetName(), f.GoPkg)
			genFile := gen.NewGeneratedFile(f.GetName(), protogen.GoImportPath(f.GoPkg.Path))
			if _, err := genFile.Write([]byte(f.GetContent())); err != nil {
				return err
			}
		}

		glog.V(1).Info("Processed code generator request")

		return err
	})
}

func applyFlags(reg *descriptor.Registry) error {
	if *grpcAPIConfiguration != "" {
		if err := reg.LoadGrpcAPIServiceFromYAML(*grpcAPIConfiguration); err != nil {
			return err
		}
	}
	if *warnOnUnboundMethods && *generateUnboundMethods {
		glog.Warningf("Option warn_on_unbound_methods has no effect when generate_unbound_methods is used.")
	}
	reg.SetStandalone(*standalone)
	reg.SetAllowDeleteBody(*allowDeleteBody)

	flag.Visit(func(f *flag.Flag) {
		if f.Name == "allow_repeated_fields_in_body" {
			glog.Warning("The `allow_repeated_fields_in_body` flag is deprecated and will always behave as `true`.")
		}
	})

	reg.SetOmitPackageDoc(*omitPackageDoc)
	reg.SetWarnOnUnboundMethods(*warnOnUnboundMethods)
	reg.SetGenerateUnboundMethods(*generateUnboundMethods)
	return reg.SetRepeatedPathParamSeparator(*repeatedPathParamSeparator)
}
