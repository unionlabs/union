package validator

import (
	"encoding/json"
	"errors"
	"io"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prysmaticlabs/prysm/v4/api/client"
	"github.com/prysmaticlabs/prysm/v4/api/client/validator"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	validatorType "github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
	"go.opencensus.io/trace"
)

func getProposerSettings(c *cli.Context, r io.Reader) error {
	ctx, span := trace.StartSpan(c.Context, "prysmctl.getProposerSettings")
	defer span.End()
	if !c.IsSet(HostFlag.Name) {
		return errNoFlag(HostFlag.Name)
	}
	if !c.IsSet(TokenFlag.Name) {
		return errNoFlag(TokenFlag.Name)
	}
	defaultFeeRecipient := params.BeaconConfig().DefaultFeeRecipient.Hex()
	if c.IsSet(ProposerSettingsOutputFlag.Name) {
		if c.IsSet(DefaultFeeRecipientFlag.Name) {
			recipient := c.String(DefaultFeeRecipientFlag.Name)
			if err := validateIsExecutionAddress(recipient); err != nil {
				return err
			}
			defaultFeeRecipient = recipient
		} else {
			promptText := "Please enter a default fee recipient address (an ethereum address in hex format)"
			resp, err := prompt.ValidatePrompt(r, promptText, validateIsExecutionAddress)
			if err != nil {
				return err
			}
			defaultFeeRecipient = resp
		}
	}

	cl, err := validator.NewClient(c.String(HostFlag.Name), client.WithAuthenticationToken(c.String(TokenFlag.Name)))
	if err != nil {
		return err
	}
	validators, err := cl.GetValidatorPubKeys(ctx)
	if err != nil {
		return err
	}
	feeRecipients, err := cl.GetFeeRecipientAddresses(ctx, validators)
	if err != nil {
		return err
	}

	log.Infoln("===============DISPLAYING CURRENT PROPOSER SETTINGS===============")

	for index := range validators {
		log.Infof("Validator: %s. Fee-recipient: %s", validators[index], feeRecipients[index])
	}

	if c.IsSet(ProposerSettingsOutputFlag.Name) {
		log.Infof("The default fee recipient is set to %s", defaultFeeRecipient)
		var builderSettings *validatorpb.BuilderConfig
		if c.Bool(WithBuilderFlag.Name) {
			builderSettings = &validatorpb.BuilderConfig{
				Enabled:  true,
				GasLimit: validatorType.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
			}
		} else {
			log.Infof("Default builder settings can be included with the `--%s` flag", WithBuilderFlag.Name)
		}
		proposerConfig := make(map[string]*validatorpb.ProposerOptionPayload)
		for index, val := range validators {
			proposerConfig[val] = &validatorpb.ProposerOptionPayload{
				FeeRecipient: feeRecipients[index],
				Builder:      builderSettings,
			}
		}
		fileConfig := &validatorpb.ProposerSettingsPayload{
			ProposerConfig: proposerConfig,
			DefaultConfig: &validatorpb.ProposerOptionPayload{
				FeeRecipient: defaultFeeRecipient,
				Builder:      builderSettings,
			},
		}
		b, err := json.Marshal(fileConfig)
		if err != nil {
			return err
		}
		if err := file.WriteFile(c.String(ProposerSettingsOutputFlag.Name), b); err != nil {
			return err
		}
		log.Infof("Successfully created `%s`. Settings can be imported into validator client using --%s flag.", c.String(ProposerSettingsOutputFlag.Name), flags.ProposerSettingsFlag.Name)
	}

	return nil
}

func validateIsExecutionAddress(input string) error {
	if !bytesutil.IsHex([]byte(input)) || !(len(input) == common.AddressLength*2+2) {
		return errors.New("no default address entered")
	}
	return nil
}
