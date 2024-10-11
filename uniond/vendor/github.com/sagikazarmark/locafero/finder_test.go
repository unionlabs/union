package locafero

import (
	"fmt"
	"path"
	"testing"

	"github.com/spf13/afero"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func Example() {
	fsys := afero.NewBasePathFs(afero.NewOsFs(), "testdata")

	finder := Finder{
		Paths: []string{
			"home/user",
			"etc",
		},
		Names: []string{"config.*"},
		Type:  FileTypeFile,
	}

	results, err := finder.Find(fsys)
	if err != nil {
		panic(err)
	}

	fmt.Print(results)

	// Output: [home/user/config.yaml etc/config.yaml]
}

func TestFinder_Find(t *testing.T) {
	fsys := afero.NewMemMapFs()

	files := []string{
		"home/user/.config/app/config.yaml",
		"home/user/app/config.yaml",
		"home/user/config.json",
		"home/user/config.yaml",
		"home/user/config/app.yaml",
		"home/user/config/config.yaml",
		"etc/app/config.yaml",
		"etc/config.json",
		"etc/config.yaml",
		"etc/config/app.yaml",
		"etc/config/config.yaml",
	}

	for _, file := range files {
		dir := path.Dir(file)

		err := fsys.MkdirAll(dir, 0o777)
		require.NoError(t, err)

		_, err = fsys.Create(file)
		require.NoError(t, err)
	}

	tests := []struct {
		name    string
		finder  Finder
		results []string
	}{
		{
			name:    "nothing to find",
			finder:  Finder{},
			results: nil,
		},
		{
			name: "no names to find",
			finder: Finder{
				Paths: []string{"home/user"},
			},
			results: nil,
		},
		{
			name: "no paths to find in",
			finder: Finder{
				Names: []string{"config.yaml"},
			},
			results: nil,
		},
		{
			name: "find in path",
			finder: Finder{
				Paths: []string{"home/user"},
				Names: []string{"config.yaml"},
			},
			results: []string{
				"home/user/config.yaml",
			},
		},
		{
			name: "find in multiple paths",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config.yaml"},
			},
			results: []string{
				"home/user/config.yaml",
				"etc/config.yaml",
			},
		},
		{
			name: "find multiple names in multiple paths",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config", "config.yaml"},
			},
			results: []string{
				"home/user/config",
				"home/user/config.yaml",
				"etc/config",
				"etc/config.yaml",
			},
		},
		{
			name: "find in subdirs of each other",
			finder: Finder{
				Paths: []string{"home/user", "home/user/app"},
				Names: []string{"config.yaml"},
			},
			results: []string{
				"home/user/config.yaml",
				"home/user/app/config.yaml",
			},
		},
		{
			name: "find files only",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config", "config.yaml"},
				Type:  FileTypeFile,
			},
			results: []string{
				"home/user/config.yaml",
				"etc/config.yaml",
			},
		},
		{
			name: "find dirs only",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config", "config.yaml"},
				Type:  FileTypeDir,
			},
			results: []string{
				"home/user/config",
				"etc/config",
			},
		},
		{
			name: "glob match",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config*"},
			},
			results: []string{
				"home/user/config",
				"home/user/config.json",
				"home/user/config.yaml",
				"etc/config",
				"etc/config.json",
				"etc/config.yaml",
			},
		},
		{
			name: "glob match",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config.*"},
			},
			results: []string{
				"home/user/config.json",
				"home/user/config.yaml",
				"etc/config.json",
				"etc/config.yaml",
			},
		},
		{
			name: "glob match files",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config*"},
				Type:  FileTypeFile,
			},
			results: []string{
				"home/user/config.json",
				"home/user/config.yaml",
				"etc/config.json",
				"etc/config.yaml",
			},
		},
		{
			name: "glob match dirs",
			finder: Finder{
				Paths: []string{"home/user", "etc"},
				Names: []string{"config*"},
				Type:  FileTypeDir,
			},
			results: []string{
				"home/user/config",
				"etc/config",
			},
		},
		{
			name: "glob match in subdirs of each other",
			finder: Finder{
				Paths: []string{"home/user", "home/user/config", "etc", "etc/config"},
				Names: []string{"config*"},
			},
			results: []string{
				"home/user/config",
				"home/user/config.json",
				"home/user/config.yaml",
				"home/user/config/config.yaml",
				"etc/config",
				"etc/config.json",
				"etc/config.yaml",
				"etc/config/config.yaml",
			},
		},
		{
			name: "glob match files in subdirs of each other",
			finder: Finder{
				Paths: []string{"home/user", "home/user/config", "etc", "etc/config"},
				Names: []string{"config*"},
				Type:  FileTypeFile,
			},
			results: []string{
				"home/user/config.json",
				"home/user/config.yaml",
				"home/user/config/config.yaml",
				"etc/config.json",
				"etc/config.yaml",
				"etc/config/config.yaml",
			},
		},
		{
			name: "glob match dirs in subdirs of each other",
			finder: Finder{
				Paths: []string{"home/user", "home/user/config", "etc", "etc/config"},
				Names: []string{"config*"},
				Type:  FileTypeDir,
			},
			results: []string{
				"home/user/config",
				"etc/config",
			},
		},
	}

	for _, tt := range tests {
		tt := tt

		t.Run(tt.name, func(t *testing.T) {
			results, err := tt.finder.Find(fsys)
			require.NoError(t, err)

			assert.Equal(t, tt.results, results)
		})
	}
}
