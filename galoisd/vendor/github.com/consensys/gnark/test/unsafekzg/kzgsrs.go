// Package unsafekzg is a convenience package (to be use for test purposes only)
// to generate and cache SRS for the kzg scheme (and indirectly for PlonK setup).
//
// Functions in this package are thread safe.
package unsafekzg

import (
	"bufio"
	"crypto/rand"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
	"regexp"
	"sync"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/kzg"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/internal/utils"
	"github.com/consensys/gnark/logger"

	kzg_bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/kzg"
	kzg_bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/kzg"
	kzg_bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/kzg"
	kzg_bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/kzg"
	kzg_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/kzg"
	kzg_bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/kzg"
	kzg_bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/kzg"

	fft_bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr/fft"
	fft_bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr/fft"
	fft_bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr/fft"
	fft_bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/fr/fft"
	fft_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr/fft"
	fft_bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/fr/fft"
	fft_bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr/fft"

	fr_bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr"
	fr_bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	fr_bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr"
	fr_bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/fr"
	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	fr_bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/fr"
	fr_bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr"

	bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377"
	bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381"
	bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315"
	bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633"
	bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761"
)

var (
	cache           = make(map[string]cacheEntry)
	reCacheKey      = regexp.MustCompile(`kzgsrs-(.*?)-\d+`)
	memLock, fsLock sync.RWMutex
)

// NewSRS returns a pair of kzg.SRS; one in canonical form, the other in lagrange form.
// Default options use a memory cache, see Option for more details & options.
func NewSRS(ccs constraint.ConstraintSystem, opts ...Option) (canonical kzg.SRS, lagrange kzg.SRS, err error) {

	nbConstraints := ccs.GetNbConstraints()
	sizeSystem := nbConstraints + ccs.GetNbPublicVariables()

	sizeLagrange := ecc.NextPowerOfTwo(uint64(sizeSystem))
	sizeCanonical := sizeLagrange + 3

	curveID := utils.FieldToCurve(ccs.Field())

	log := logger.Logger().With().Str("package", "kzgsrs").Int("size", int(sizeCanonical)).Str("curve", curveID.String()).Logger()

	cfg, err := options(opts...)
	if err != nil {
		return nil, nil, err
	}

	key := cacheKey(curveID, sizeCanonical)
	log.Debug().Str("key", key).Msg("fetching SRS from mem cache")
	memLock.RLock()
	entry, ok := cache[key]
	memLock.RUnlock()
	if ok {
		log.Debug().Msg("SRS found in mem cache")
		return entry.canonical, entry.lagrange, nil
	}
	log.Debug().Msg("SRS not found in mem cache")

	if cfg.fsCache {
		log.Debug().Str("key", key).Str("cacheDir", cfg.cacheDir).Msg("fetching SRS from fs cache")
		fsLock.RLock()
		entry, err = fsRead(key, cfg.cacheDir)
		fsLock.RUnlock()
		if err == nil {
			log.Debug().Str("key", key).Msg("SRS found in fs cache")
			canonical, lagrange = entry.canonical, entry.lagrange
			memLock.Lock()
			cache[key] = cacheEntry{canonical, lagrange}
			memLock.Unlock()
			return
		} else {
			log.Debug().Str("key", key).Err(err).Msg("SRS not found in fs cache")
		}
	}

	log.Debug().Msg("SRS not found in cache, generating")

	// not in cache, generate
	canonical, lagrange, err = newSRS(curveID, sizeCanonical)
	if err != nil {
		return nil, nil, err
	}

	// cache it
	memLock.Lock()
	cache[key] = cacheEntry{canonical, lagrange}
	memLock.Unlock()

	if cfg.fsCache {
		log.Debug().Str("key", key).Str("cacheDir", cfg.cacheDir).Msg("writing SRS to fs cache")
		fsLock.Lock()
		fsWrite(key, cfg.cacheDir, canonical, lagrange)
		fsLock.Unlock()
	}

	return canonical, lagrange, nil
}

type cacheEntry struct {
	canonical kzg.SRS
	lagrange  kzg.SRS
}

func cacheKey(curveID ecc.ID, size uint64) string {
	return fmt.Sprintf("kzgsrs-%s-%d", curveID.String(), size)
}

func extractCurveID(key string) (ecc.ID, error) {
	matches := reCacheKey.FindStringSubmatch(key)

	if len(matches) < 2 {
		return ecc.UNKNOWN, fmt.Errorf("no curveID found in key")
	}
	return ecc.IDFromString(matches[1])
}

func newSRS(curveID ecc.ID, size uint64) (kzg.SRS, kzg.SRS, error) {

	tau, err := rand.Int(rand.Reader, curveID.ScalarField())
	if err != nil {
		return nil, nil, err
	}

	var srs kzg.SRS

	switch curveID {
	case ecc.BN254:
		srs, err = kzg_bn254.NewSRS(size, tau)
	case ecc.BLS12_381:
		srs, err = kzg_bls12381.NewSRS(size, tau)
	case ecc.BLS12_377:
		srs, err = kzg_bls12377.NewSRS(size, tau)
	case ecc.BW6_761:
		srs, err = kzg_bw6761.NewSRS(size, tau)
	case ecc.BLS24_317:
		srs, err = kzg_bls24317.NewSRS(size, tau)
	case ecc.BLS24_315:
		srs, err = kzg_bls24315.NewSRS(size, tau)
	case ecc.BW6_633:
		srs, err = kzg_bw6633.NewSRS(size, tau)
	default:
		panic("unrecognized R1CS curve type")
	}

	if err != nil {
		return nil, nil, err
	}

	return srs, toLagrange(srs, tau), nil
}

func toLagrange(canonicalSRS kzg.SRS, tau *big.Int) kzg.SRS {

	var lagrangeSRS kzg.SRS

	switch srs := canonicalSRS.(type) {
	case *kzg_bn254.SRS:
		newSRS := &kzg_bn254.SRS{Vk: srs.Vk}
		size := uint64(len(srs.Pk.G1)) - 3

		// instead of using ToLagrangeG1 we can directly do a fft on the powers of alpha
		// since we know the randomness in test.
		pAlpha := make([]fr_bn254.Element, size)
		pAlpha[0].SetUint64(1)
		pAlpha[1].SetBigInt(tau)
		for i := 2; i < len(pAlpha); i++ {
			pAlpha[i].Mul(&pAlpha[i-1], &pAlpha[1])
		}
		// do a fft on this.
		d := fft_bn254.NewDomain(size)
		d.FFTInverse(pAlpha, fft_bn254.DIF)
		fft_bn254.BitReverse(pAlpha)

		// bath scalar mul
		_, _, g1gen, _ := bn254.Generators()
		newSRS.Pk.G1 = bn254.BatchScalarMultiplicationG1(&g1gen, pAlpha)

		lagrangeSRS = newSRS
	case *kzg_bls12381.SRS:
		newSRS := &kzg_bls12381.SRS{Vk: srs.Vk}
		size := uint64(len(srs.Pk.G1)) - 3

		// instead of using ToLagrangeG1 we can directly do a fft on the powers of alpha
		// since we know the randomness in test.
		pAlpha := make([]fr_bls12381.Element, size)
		pAlpha[0].SetUint64(1)
		pAlpha[1].SetBigInt(tau)
		for i := 2; i < len(pAlpha); i++ {
			pAlpha[i].Mul(&pAlpha[i-1], &pAlpha[1])
		}
		// do a fft on this.
		d := fft_bls12381.NewDomain(size)
		d.FFTInverse(pAlpha, fft_bls12381.DIF)
		fft_bls12381.BitReverse(pAlpha)

		// bath scalar mul
		_, _, g1gen, _ := bls12381.Generators()
		newSRS.Pk.G1 = bls12381.BatchScalarMultiplicationG1(&g1gen, pAlpha)

		lagrangeSRS = newSRS
	case *kzg_bls12377.SRS:
		newSRS := &kzg_bls12377.SRS{Vk: srs.Vk}
		size := uint64(len(srs.Pk.G1)) - 3

		// instead of using ToLagrangeG1 we can directly do a fft on the powers of alpha
		// since we know the randomness in test.
		pAlpha := make([]fr_bls12377.Element, size)
		pAlpha[0].SetUint64(1)
		pAlpha[1].SetBigInt(tau)
		for i := 2; i < len(pAlpha); i++ {
			pAlpha[i].Mul(&pAlpha[i-1], &pAlpha[1])
		}
		// do a fft on this.
		d := fft_bls12377.NewDomain(size)
		d.FFTInverse(pAlpha, fft_bls12377.DIF)
		fft_bls12377.BitReverse(pAlpha)

		// bath scalar mul
		_, _, g1gen, _ := bls12377.Generators()
		newSRS.Pk.G1 = bls12377.BatchScalarMultiplicationG1(&g1gen, pAlpha)

		lagrangeSRS = newSRS
	case *kzg_bw6761.SRS:
		newSRS := &kzg_bw6761.SRS{Vk: srs.Vk}
		size := uint64(len(srs.Pk.G1)) - 3

		// instead of using ToLagrangeG1 we can directly do a fft on the powers of alpha
		// since we know the randomness in test.
		pAlpha := make([]fr_bw6761.Element, size)
		pAlpha[0].SetUint64(1)
		pAlpha[1].SetBigInt(tau)
		for i := 2; i < len(pAlpha); i++ {
			pAlpha[i].Mul(&pAlpha[i-1], &pAlpha[1])
		}

		// do a fft on this.
		d := fft_bw6761.NewDomain(size)
		d.FFTInverse(pAlpha, fft_bw6761.DIF)
		fft_bw6761.BitReverse(pAlpha)

		// bath scalar mul
		_, _, g1gen, _ := bw6761.Generators()
		newSRS.Pk.G1 = bw6761.BatchScalarMultiplicationG1(&g1gen, pAlpha)

		lagrangeSRS = newSRS
	case *kzg_bls24317.SRS:
		newSRS := &kzg_bls24317.SRS{Vk: srs.Vk}
		size := uint64(len(srs.Pk.G1)) - 3

		// instead of using ToLagrangeG1 we can directly do a fft on the powers of alpha
		// since we know the randomness in test.
		pAlpha := make([]fr_bls24317.Element, size)
		pAlpha[0].SetUint64(1)
		pAlpha[1].SetBigInt(tau)
		for i := 2; i < len(pAlpha); i++ {
			pAlpha[i].Mul(&pAlpha[i-1], &pAlpha[1])
		}

		// do a fft on this.
		d := fft_bls24317.NewDomain(size)
		d.FFTInverse(pAlpha, fft_bls24317.DIF)
		fft_bls24317.BitReverse(pAlpha)

		// bath scalar mul
		_, _, g1gen, _ := bls24317.Generators()
		newSRS.Pk.G1 = bls24317.BatchScalarMultiplicationG1(&g1gen, pAlpha)

		lagrangeSRS = newSRS
	case *kzg_bls24315.SRS:
		newSRS := &kzg_bls24315.SRS{Vk: srs.Vk}
		size := uint64(len(srs.Pk.G1)) - 3

		// instead of using ToLagrangeG1 we can directly do a fft on the powers of alpha
		// since we know the randomness in test.
		pAlpha := make([]fr_bls24315.Element, size)
		pAlpha[0].SetUint64(1)
		pAlpha[1].SetBigInt(tau)
		for i := 2; i < len(pAlpha); i++ {
			pAlpha[i].Mul(&pAlpha[i-1], &pAlpha[1])
		}

		// do a fft on this.
		d := fft_bls24315.NewDomain(size)
		d.FFTInverse(pAlpha, fft_bls24315.DIF)
		fft_bls24315.BitReverse(pAlpha)

		// bath scalar mul
		_, _, g1gen, _ := bls24315.Generators()
		newSRS.Pk.G1 = bls24315.BatchScalarMultiplicationG1(&g1gen, pAlpha)

		lagrangeSRS = newSRS
	case *kzg_bw6633.SRS:
		newSRS := &kzg_bw6633.SRS{Vk: srs.Vk}
		size := uint64(len(srs.Pk.G1)) - 3

		// instead of using ToLagrangeG1 we can directly do a fft on the powers of alpha
		// since we know the randomness in test.
		pAlpha := make([]fr_bw6633.Element, size)
		pAlpha[0].SetUint64(1)
		pAlpha[1].SetBigInt(tau)
		for i := 2; i < len(pAlpha); i++ {
			pAlpha[i].Mul(&pAlpha[i-1], &pAlpha[1])
		}

		// do a fft on this.
		d := fft_bw6633.NewDomain(size)
		d.FFTInverse(pAlpha, fft_bw6633.DIF)
		fft_bw6633.BitReverse(pAlpha)

		// bath scalar mul
		_, _, g1gen, _ := bw6633.Generators()
		newSRS.Pk.G1 = bw6633.BatchScalarMultiplicationG1(&g1gen, pAlpha)

		lagrangeSRS = newSRS
	default:
		panic("unrecognized curve")
	}

	return lagrangeSRS
}

func fsRead(key string, cacheDir string) (cacheEntry, error) {
	filePath := filepath.Join(cacheDir, key)

	// if file does not exist, return false
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		return cacheEntry{}, fmt.Errorf("file %s does not exist", filePath)
	}

	// else open file and read the srs.
	f, err := os.Open(filePath)
	if err != nil {
		return cacheEntry{}, err
	}
	defer f.Close()

	r := bufio.NewReaderSize(f, 1<<20)

	curveID, err := extractCurveID(key)
	if err != nil {
		return cacheEntry{}, err
	}
	cacheEntry := cacheEntry{
		canonical: kzg.NewSRS(curveID),
		lagrange:  kzg.NewSRS(curveID),
	}
	_, err = cacheEntry.canonical.UnsafeReadFrom(r)
	if err != nil {
		return cacheEntry, err
	}
	_, err = cacheEntry.lagrange.UnsafeReadFrom(r)
	if err != nil {
		return cacheEntry, err
	}

	return cacheEntry, nil
}

func fsWrite(key string, cacheDir string, canonical kzg.SRS, lagrange kzg.SRS) {
	// if file exist, return.
	filePath := filepath.Join(cacheDir, key)
	if _, err := os.Stat(filePath); err == nil {
		return
	}

	// else open file and write the srs.
	f, err := os.Create(filePath)
	if err != nil {
		return
	}
	defer f.Close()

	w := bufio.NewWriterSize(f, 1<<20)

	if _, err = canonical.WriteRawTo(w); err != nil {
		return
	}

	if _, err = lagrange.WriteRawTo(w); err != nil {
		return
	}

	w.Flush()
}
