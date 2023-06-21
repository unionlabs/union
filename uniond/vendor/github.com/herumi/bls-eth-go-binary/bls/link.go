package bls

/*
#cgo LDFLAGS:-lbls384_256 -lstdc++ -lm
#cgo ios LDFLAGS:-L${SRCDIR}/lib/ios
#cgo android,arm64 LDFLAGS:-L${SRCDIR}/lib/linux/arm64
#cgo android,arm LDFLAGS:-L${SRCDIR}/lib/android/armeabi-v7a
#cgo android,amd64 LDFLAGS:-L${SRCDIR}/lib/linux/amd64
#cgo linux,amd64 LDFLAGS:-L${SRCDIR}/lib/linux/amd64
#cgo linux,arm64 LDFLAGS:-L${SRCDIR}/lib/linux/arm64
#cgo linux,mipsle LDFLAGS:-L${SRCDIR}/lib/linux/mipsel
#cgo linux,arm LDFLAGS:-L${SRCDIR}/lib/linux/arm
#cgo linux,s390x LDFLAGS:-L${SRCDIR}/lib/linux/s390x
#cgo darwin,amd64 LDFLAGS:-L${SRCDIR}/lib/darwin/amd64
#cgo darwin,arm64 LDFLAGS:-L${SRCDIR}/lib/darwin/arm64
#cgo windows,amd64 LDFLAGS:-L${SRCDIR}/lib/windows/amd64
#cgo openbsd,amd64 LDFLAGS:-L${SRCDIR}/lib/openbsd/amd64
#cgo freebsd,amd64 LDFLAGS:-L${SRCDIR}/lib/freebsd/amd64
*/
import "C"
