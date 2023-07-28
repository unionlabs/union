//go:build !viper_yaml2
// +build !viper_yaml2

package yaml

import yamlv3 "gopkg.in/yaml.v3"

var yaml = struct {
	Marshal   func(in interface{}) (out []byte, err error)
	Unmarshal func(in []byte, out interface{}) (err error)
}{
	Marshal:   yamlv3.Marshal,
	Unmarshal: yamlv3.Unmarshal,
}
