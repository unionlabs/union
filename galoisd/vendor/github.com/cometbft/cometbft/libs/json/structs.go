package json

import (
	"fmt"
	"reflect"
	"strings"
	"unicode"

	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

var (
	// cache caches struct info.
	cache = newStructInfoCache()
)

// structCache is a cache of struct info.
type structInfoCache struct {
	cmtsync.RWMutex
	structInfos map[reflect.Type]*structInfo
}

func newStructInfoCache() *structInfoCache {
	return &structInfoCache{
		structInfos: make(map[reflect.Type]*structInfo),
	}
}

func (c *structInfoCache) get(rt reflect.Type) *structInfo {
	c.RLock()
	defer c.RUnlock()
	return c.structInfos[rt]
}

func (c *structInfoCache) set(rt reflect.Type, sInfo *structInfo) {
	c.Lock()
	defer c.Unlock()
	c.structInfos[rt] = sInfo
}

// structInfo contains JSON info for a struct.
type structInfo struct {
	fields []*fieldInfo
}

// fieldInfo contains JSON info for a struct field.
type fieldInfo struct {
	jsonName  string
	omitEmpty bool
	hidden    bool
}

// makeStructInfo generates structInfo for a struct as a reflect.Value.
func makeStructInfo(rt reflect.Type) *structInfo {
	if rt.Kind() != reflect.Struct {
		panic(fmt.Sprintf("can't make struct info for non-struct value %v", rt))
	}
	if sInfo := cache.get(rt); sInfo != nil {
		return sInfo
	}
	fields := make([]*fieldInfo, 0, rt.NumField())
	for i := 0; i < cap(fields); i++ {
		frt := rt.Field(i)
		fInfo := &fieldInfo{
			jsonName:  frt.Name,
			omitEmpty: false,
			hidden:    frt.Name == "" || !unicode.IsUpper(rune(frt.Name[0])),
		}
		o := frt.Tag.Get("json")
		if o == "-" {
			fInfo.hidden = true
		} else if o != "" {
			opts := strings.Split(o, ",")
			if opts[0] != "" {
				fInfo.jsonName = opts[0]
			}
			for _, o := range opts[1:] {
				if o == "omitempty" {
					fInfo.omitEmpty = true
				}
			}
		}
		fields = append(fields, fInfo)
	}
	sInfo := &structInfo{fields: fields}
	cache.set(rt, sInfo)
	return sInfo
}
