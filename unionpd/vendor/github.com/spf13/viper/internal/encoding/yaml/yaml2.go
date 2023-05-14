//go:build viper_yaml2
// +build viper_yaml2

package yaml

import yamlv2 "gopkg.in/yaml.v2"

var yaml = struct {
	Marshal   func(in interface{}) (out []byte, err error)
	Unmarshal func(in []byte, out interface{}) (err error)
}{
	Marshal:   yamlv2.Marshal,
	Unmarshal: yamlv2.Unmarshal,
}
