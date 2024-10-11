package appconfig

import (
	"fmt"
	"reflect"
	"strings"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/encoding/protojson"
	protov2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
	"sigs.k8s.io/yaml"

	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig/v1alpha1"
	internal "cosmossdk.io/depinject/internal/appconfig"
)

// LoadJSON loads an app config in JSON format.
func LoadJSON(bz []byte) depinject.Config {
	// in order to avoid a direct dependency on api types, but in order to also be able to support
	// either gogo or google.golang.org/protobuf types, we use protojson and dynamicpb to unmarshal
	// from JSON
	resolver := gogoproto.HybridResolver
	desc, err := resolver.FindDescriptorByName(protoreflect.FullName(gogoproto.MessageName(&v1alpha1.Config{})))
	if err != nil {
		return depinject.Error(err)
	}

	config := dynamicpb.NewMessage(desc.(protoreflect.MessageDescriptor))
	err = protojson.UnmarshalOptions{
		Resolver: dynamicTypeResolver{resolver: gogoproto.HybridResolver},
	}.Unmarshal(bz, config)
	if err != nil {
		return depinject.Error(err)
	}

	return Compose(config)
}

// LoadYAML loads an app config in YAML format.
func LoadYAML(bz []byte) depinject.Config {
	j, err := yaml.YAMLToJSON(bz)
	if err != nil {
		return depinject.Error(err)
	}

	return LoadJSON(j)
}

// WrapAny marshals a proto message into a proto Any instance
func WrapAny(config gogoproto.Message) *anypb.Any {
	pbz, err := gogoproto.Marshal(config)
	if err != nil {
		panic(err)
	}

	return &anypb.Any{
		TypeUrl: "/" + gogoproto.MessageName(config),
		Value:   pbz,
	}
}

// Compose composes an app config into a container option by resolving
// the required modules and composing their options. appConfig should be an instance
// of cosmos.app.v1alpha1.Config (it doesn't matter whether you use gogo proto or
// google.golang.org/protobuf types).
func Compose(appConfig gogoproto.Message) depinject.Config {
	appConfigConcrete, ok := appConfig.(*v1alpha1.Config)
	if !ok {
		// we convert any other proto type that was passed (such as an api module type) to the concrete
		// type we're using here
		appConfigConcrete = &v1alpha1.Config{}
		bz, err := gogoproto.Marshal(appConfig)
		if err != nil {
			return depinject.Error(err)
		}

		err = gogoproto.Unmarshal(bz, appConfigConcrete)
		if err != nil {
			return depinject.Error(err)
		}
	}

	opts := []depinject.Config{
		depinject.Supply(appConfig),
	}

	modules, err := internal.ModulesByModuleTypeName()
	if err != nil {
		return depinject.Error(err)
	}

	for _, module := range appConfigConcrete.Modules {
		if module.Name == "" {
			return depinject.Error(fmt.Errorf("module is missing name"))
		}

		if module.Config == nil {
			return depinject.Error(fmt.Errorf("module %q is missing a config object", module.Name))
		}

		msgName := module.Config.TypeUrl
		// strip type URL prefix
		if slashIdx := strings.LastIndex(msgName, "/"); slashIdx >= 0 {
			msgName = msgName[slashIdx+1:]
		}
		if msgName == "" {
			return depinject.Error(fmt.Errorf("module %q is missing a type URL", module.Name))
		}

		init, ok := modules[msgName]
		if !ok {
			if msgDesc, err := gogoproto.HybridResolver.FindDescriptorByName(protoreflect.FullName(msgName)); err == nil {
				modDesc, err := internal.GetModuleDescriptor(msgDesc)
				if err != nil {
					return depinject.Error(err)
				}

				if modDesc == nil {
					return depinject.Error(fmt.Errorf("no module registered for type URL %s and that protobuf type does not have the option %s\n\n%s",
						module.Config.TypeUrl, v1alpha1.E_Module.Name, dumpRegisteredModules(modules)))
				}

				return depinject.Error(fmt.Errorf("no module registered for type URL %s, did you forget to import %s: find more information on how to make a module ready for app wiring: https://docs.cosmos.network/main/building-modules/depinject\n\n%s",
					module.Config.TypeUrl, modDesc.GoImport, dumpRegisteredModules(modules)))
			}
		}

		var config gogoproto.Message
		if configInit, ok := init.ConfigProtoMessage.(protov2.Message); ok {
			config = configInit.ProtoReflect().Type().New().Interface().(gogoproto.Message)
		} else {
			config = reflect.New(init.ConfigGoType.Elem()).Interface().(gogoproto.Message)
		}
		// as of gogo v1.5.0 this should work with either gogoproto or golang v2 proto
		err = gogoproto.Unmarshal(module.Config.Value, config)
		if err != nil {
			return depinject.Error(err)
		}

		opts = append(opts, depinject.Supply(config))

		for _, provider := range init.Providers {
			opts = append(opts, depinject.ProvideInModule(module.Name, provider))
		}

		for _, invoker := range init.Invokers {
			opts = append(opts, depinject.InvokeInModule(module.Name, invoker))
		}

		for _, binding := range module.GolangBindings {
			opts = append(opts, depinject.BindInterfaceInModule(module.Name, binding.InterfaceType, binding.Implementation))
		}
	}

	for _, binding := range appConfigConcrete.GolangBindings {
		opts = append(opts, depinject.BindInterface(binding.InterfaceType, binding.Implementation))
	}

	return depinject.Configs(opts...)
}

func dumpRegisteredModules(modules map[string]*internal.ModuleInitializer) string {
	var mods []string
	for name := range modules {
		mods = append(mods, "  "+name)
	}
	return fmt.Sprintf("registered modules are:\n%s", strings.Join(mods, "\n"))
}
