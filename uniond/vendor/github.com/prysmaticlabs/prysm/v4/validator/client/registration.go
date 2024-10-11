package client

import (
	"context"
	"strings"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
	"go.opencensus.io/trace"
)

// SubmitValidatorRegistrations signs validator registration objects and submits it to the beacon node by batch of validatorRegsBatchSize size maximum.
// If at least one error occurs during a registration call to the beacon node, the last error is returned.
func SubmitValidatorRegistrations(
	ctx context.Context,
	validatorClient iface.ValidatorClient,
	signedRegs []*ethpb.SignedValidatorRegistrationV1,
	validatorRegsBatchSize int,
) error {
	ctx, span := trace.StartSpan(ctx, "validator.SubmitValidatorRegistrations")
	defer span.End()

	if len(signedRegs) == 0 {
		return nil
	}

	chunks := chunkSignedValidatorRegistrationV1(signedRegs, validatorRegsBatchSize)
	var lastErr error

	for _, chunk := range chunks {
		innerSignerRegs := ethpb.SignedValidatorRegistrationsV1{
			Messages: chunk,
		}

		if _, err := validatorClient.SubmitValidatorRegistrations(ctx, &innerSignerRegs); err != nil {
			lastErr = errors.Wrap(err, "could not submit signed registrations to beacon node")

			if strings.Contains(err.Error(), builder.ErrNoBuilder.Error()) {
				log.Warnln("Beacon node does not utilize a custom builder via the --http-mev-relay flag. Validator registration skipped.")

				// We stop early the loop here, since if the builder endpoint is not configured for this chunk, it is useless to check the following chunks
				break
			}
		}
	}

	if lastErr == nil {
		log.Infoln("Submitted builder validator registration settings for custom builders")
	} else {
		log.WithError(lastErr).Warn("Could not submit all signed registrations to beacon node")
	}

	return lastErr
}

// Sings validator registration obj with the proposer domain and private key.
func signValidatorRegistration(ctx context.Context, signer iface.SigningFunc, reg *ethpb.ValidatorRegistrationV1) ([]byte, error) {
	// Per spec, we want the fork version and genesis validator to be nil.
	// Which is genesis value and zero by default.
	d, err := signing.ComputeDomain(
		params.BeaconConfig().DomainApplicationBuilder,
		nil, /* fork version */
		nil /* genesis val root */)
	if err != nil {
		return nil, err
	}

	r, err := signing.ComputeSigningRoot(reg, d)
	if err != nil {
		return nil, errors.Wrap(err, signingRootErr)
	}

	sig, err := signer(ctx, &validatorpb.SignRequest{
		PublicKey:       reg.Pubkey,
		SigningRoot:     r[:],
		SignatureDomain: d,
		Object:          &validatorpb.SignRequest_Registration{Registration: reg},
	})
	if err != nil {
		return nil, errors.Wrap(err, "could not sign validator registration")
	}
	return sig.Marshal(), nil
}

// SignValidatorRegistrationRequest compares and returns either the cached validator registration request or signs a new one.
func (v *validator) SignValidatorRegistrationRequest(ctx context.Context, signer iface.SigningFunc, newValidatorRegistration *ethpb.ValidatorRegistrationV1) (*ethpb.SignedValidatorRegistrationV1, error) {
	signedReg, ok := v.signedValidatorRegistrations[bytesutil.ToBytes48(newValidatorRegistration.Pubkey)]
	if ok && isValidatorRegistrationSame(signedReg.Message, newValidatorRegistration) {
		return signedReg, nil
	} else {
		sig, err := signValidatorRegistration(ctx, signer, newValidatorRegistration)
		if err != nil {
			return nil, err
		}
		newRequest := &ethpb.SignedValidatorRegistrationV1{
			Message:   newValidatorRegistration,
			Signature: sig,
		}
		v.signedValidatorRegistrations[bytesutil.ToBytes48(newValidatorRegistration.Pubkey)] = newRequest
		return newRequest, nil
	}
}

func isValidatorRegistrationSame(cachedVR *ethpb.ValidatorRegistrationV1, newVR *ethpb.ValidatorRegistrationV1) bool {
	isSame := true
	if cachedVR.GasLimit != newVR.GasLimit {
		isSame = false
	}
	if hexutil.Encode(cachedVR.FeeRecipient) != hexutil.Encode(newVR.FeeRecipient) {
		isSame = false
	}
	return isSame
}

// chunkSignedValidatorRegistrationV1 chunks regs into chunks of size chunkSize (the last chunk may be smaller). If chunkSize is non-positive, returns only one chunk.
func chunkSignedValidatorRegistrationV1(regs []*ethpb.SignedValidatorRegistrationV1, chunkSize int) [][]*ethpb.SignedValidatorRegistrationV1 {
	if chunkSize <= 0 {
		chunkSize = len(regs)
	}

	regsCount := len(regs)

	chunksCount := (regsCount + chunkSize - 1) / chunkSize
	lastChunkSize := regsCount % chunkSize

	if lastChunkSize == 0 {
		lastChunkSize = chunkSize
	}

	chunks := make([][]*ethpb.SignedValidatorRegistrationV1, chunksCount)

	for i := 0; i < chunksCount-1; i++ {
		chunks[i] = make([]*ethpb.SignedValidatorRegistrationV1, chunkSize)
	}

	chunks[chunksCount-1] = make([]*ethpb.SignedValidatorRegistrationV1, lastChunkSize)

	for i, reg := range regs {
		chunkIndex := i / chunkSize
		chunkOffset := i % chunkSize
		chunks[chunkIndex][chunkOffset] = reg
	}

	return chunks
}
