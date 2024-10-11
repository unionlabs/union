//go:build !develop

package blockchain

func init() {
	log.Fatal("Tests in this package require extra build tag: re-run with `-tags develop`")
}
