package fs

import "os"

const (
	defaultRootDirMode = os.ModeDir | 0777
	defaultSymlinkMode = os.ModeSymlink | 0666
)

func newResourceFromInfo(info os.FileInfo) resource {
	return resource{mode: info.Mode()}
}

func (p *filePath) SetMode(mode os.FileMode) {
	bits := mode & 0600
	p.file.mode = bits + bits/010 + bits/0100
}

// TODO: is mode ignored on windows?
func (p *directoryPath) SetMode(mode os.FileMode) {
	p.directory.mode = defaultRootDirMode
}
