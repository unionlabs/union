package types

import (
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"strings"

	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

type ProposalType string

const (
	ProposalTypeStoreCode                           ProposalType = "StoreCode"
	ProposalTypeInstantiateContract                 ProposalType = "InstantiateContract"
	ProposalTypeInstantiateContract2                ProposalType = "InstantiateContract2"
	ProposalTypeMigrateContract                     ProposalType = "MigrateContract"
	ProposalTypeSudoContract                        ProposalType = "SudoContract"
	ProposalTypeExecuteContract                     ProposalType = "ExecuteContract"
	ProposalTypeUpdateAdmin                         ProposalType = "UpdateAdmin"
	ProposalTypeClearAdmin                          ProposalType = "ClearAdmin"
	ProposalTypePinCodes                            ProposalType = "PinCodes"
	ProposalTypeUnpinCodes                          ProposalType = "UnpinCodes"
	ProposalTypeUpdateInstantiateConfig             ProposalType = "UpdateInstantiateConfig"
	ProposalTypeStoreAndInstantiateContractProposal ProposalType = "StoreAndInstantiateContract"
)

// DisableAllProposals contains no wasm gov types.
var DisableAllProposals []ProposalType

// EnableAllProposals contains all wasm gov types as keys.
var EnableAllProposals = []ProposalType{
	ProposalTypeStoreCode,
	ProposalTypeInstantiateContract,
	ProposalTypeInstantiateContract2,
	ProposalTypeMigrateContract,
	ProposalTypeSudoContract,
	ProposalTypeExecuteContract,
	ProposalTypeUpdateAdmin,
	ProposalTypeClearAdmin,
	ProposalTypePinCodes,
	ProposalTypeUnpinCodes,
	ProposalTypeUpdateInstantiateConfig,
	ProposalTypeStoreAndInstantiateContractProposal,
}

// ConvertToProposals maps each key to a ProposalType and returns a typed list.
// If any string is not a valid type (in this file), then return an error
func ConvertToProposals(keys []string) ([]ProposalType, error) {
	valid := make(map[string]bool, len(EnableAllProposals))
	for _, key := range EnableAllProposals {
		valid[string(key)] = true
	}

	proposals := make([]ProposalType, len(keys))
	for i, key := range keys {
		if _, ok := valid[key]; !ok {
			return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "'%s' is not a valid ProposalType", key)
		}
		proposals[i] = ProposalType(key)
	}
	return proposals, nil
}

func init() { // register new content types with the sdk
	v1beta1.RegisterProposalType(string(ProposalTypeStoreCode))
	v1beta1.RegisterProposalType(string(ProposalTypeInstantiateContract))
	v1beta1.RegisterProposalType(string(ProposalTypeInstantiateContract2))
	v1beta1.RegisterProposalType(string(ProposalTypeMigrateContract))
	v1beta1.RegisterProposalType(string(ProposalTypeSudoContract))
	v1beta1.RegisterProposalType(string(ProposalTypeExecuteContract))
	v1beta1.RegisterProposalType(string(ProposalTypeUpdateAdmin))
	v1beta1.RegisterProposalType(string(ProposalTypeClearAdmin))
	v1beta1.RegisterProposalType(string(ProposalTypePinCodes))
	v1beta1.RegisterProposalType(string(ProposalTypeUnpinCodes))
	v1beta1.RegisterProposalType(string(ProposalTypeUpdateInstantiateConfig))
	v1beta1.RegisterProposalType(string(ProposalTypeStoreAndInstantiateContractProposal))
}

func NewStoreCodeProposal(
	title string,
	description string,
	runAs string,
	wasmBz []byte,
	permission *AccessConfig,
	unpinCode bool,
	source string,
	builder string,
	codeHash []byte,
) *StoreCodeProposal {
	return &StoreCodeProposal{title, description, runAs, wasmBz, permission, unpinCode, source, builder, codeHash}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p StoreCodeProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *StoreCodeProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p StoreCodeProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p StoreCodeProposal) ProposalType() string { return string(ProposalTypeStoreCode) }

// ValidateBasic validates the proposal
func (p StoreCodeProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if _, err := sdk.AccAddressFromBech32(p.RunAs); err != nil {
		return errorsmod.Wrap(err, "run as")
	}

	if err := validateWasmCode(p.WASMByteCode, MaxProposalWasmSize); err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "code bytes %s", err.Error())
	}

	if p.InstantiatePermission != nil {
		if err := p.InstantiatePermission.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "instantiate permission")
		}
	}

	if err := ValidateVerificationInfo(p.Source, p.Builder, p.CodeHash); err != nil {
		return errorsmod.Wrapf(err, "code verification info")
	}
	return nil
}

// String implements the Stringer interface.
func (p StoreCodeProposal) String() string {
	return fmt.Sprintf(`Store Code Proposal:
  Title:       %s
  Description: %s
  Run as:      %s
  WasmCode:    %X
  Source:      %s
  Builder:     %s
  Code Hash:   %X
`, p.Title, p.Description, p.RunAs, p.WASMByteCode, p.Source, p.Builder, p.CodeHash)
}

// MarshalYAML pretty prints the wasm byte code
func (p StoreCodeProposal) MarshalYAML() (interface{}, error) {
	return struct {
		Title                 string        `yaml:"title"`
		Description           string        `yaml:"description"`
		RunAs                 string        `yaml:"run_as"`
		WASMByteCode          string        `yaml:"wasm_byte_code"`
		InstantiatePermission *AccessConfig `yaml:"instantiate_permission"`
		Source                string        `yaml:"source"`
		Builder               string        `yaml:"builder"`
		CodeHash              string        `yaml:"code_hash"`
	}{
		Title:                 p.Title,
		Description:           p.Description,
		RunAs:                 p.RunAs,
		WASMByteCode:          base64.StdEncoding.EncodeToString(p.WASMByteCode),
		InstantiatePermission: p.InstantiatePermission,
		Source:                p.Source,
		Builder:               p.Builder,
		CodeHash:              hex.EncodeToString(p.CodeHash),
	}, nil
}

func NewInstantiateContractProposal(
	title string,
	description string,
	runAs string,
	admin string,
	codeID uint64,
	label string,
	msg RawContractMessage,
	funds sdk.Coins,
) *InstantiateContractProposal {
	return &InstantiateContractProposal{title, description, runAs, admin, codeID, label, msg, funds}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p InstantiateContractProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *InstantiateContractProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p InstantiateContractProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p InstantiateContractProposal) ProposalType() string {
	return string(ProposalTypeInstantiateContract)
}

// ValidateBasic validates the proposal
func (p InstantiateContractProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if _, err := sdk.AccAddressFromBech32(p.RunAs); err != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "run as")
	}

	if p.CodeID == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "code id is required")
	}

	if err := ValidateLabel(p.Label); err != nil {
		return err
	}

	if !p.Funds.IsValid() {
		return sdkerrors.ErrInvalidCoins
	}

	if len(p.Admin) != 0 {
		if _, err := sdk.AccAddressFromBech32(p.Admin); err != nil {
			return err
		}
	}
	if err := p.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

// String implements the Stringer interface.
func (p InstantiateContractProposal) String() string {
	return fmt.Sprintf(`Instantiate Code Proposal:
  Title:       %s
  Description: %s
  Run as:      %s
  Admin:       %s
  Code id:     %d
  Label:       %s
  Msg:         %q
  Funds:       %s
`, p.Title, p.Description, p.RunAs, p.Admin, p.CodeID, p.Label, p.Msg, p.Funds)
}

// MarshalYAML pretty prints the init message
func (p InstantiateContractProposal) MarshalYAML() (interface{}, error) {
	return struct {
		Title       string    `yaml:"title"`
		Description string    `yaml:"description"`
		RunAs       string    `yaml:"run_as"`
		Admin       string    `yaml:"admin"`
		CodeID      uint64    `yaml:"code_id"`
		Label       string    `yaml:"label"`
		Msg         string    `yaml:"msg"`
		Funds       sdk.Coins `yaml:"funds"`
	}{
		Title:       p.Title,
		Description: p.Description,
		RunAs:       p.RunAs,
		Admin:       p.Admin,
		CodeID:      p.CodeID,
		Label:       p.Label,
		Msg:         string(p.Msg),
		Funds:       p.Funds,
	}, nil
}

func NewInstantiateContract2Proposal(
	title string,
	description string,
	runAs string,
	admin string,
	codeID uint64,
	label string,
	msg RawContractMessage,
	funds sdk.Coins,
	salt []byte,
	fixMsg bool,
) *InstantiateContract2Proposal {
	return &InstantiateContract2Proposal{title, description, runAs, admin, codeID, label, msg, funds, salt, fixMsg}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p InstantiateContract2Proposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *InstantiateContract2Proposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p InstantiateContract2Proposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p InstantiateContract2Proposal) ProposalType() string {
	return string(ProposalTypeInstantiateContract2)
}

// ValidateBasic validates the proposal
func (p InstantiateContract2Proposal) ValidateBasic() error {
	// Validate title and description
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	// Validate run as
	if _, err := sdk.AccAddressFromBech32(p.RunAs); err != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "run as")
	}
	// Validate admin
	if len(p.Admin) != 0 {
		if _, err := sdk.AccAddressFromBech32(p.Admin); err != nil {
			return err
		}
	}
	// Validate codeid
	if p.CodeID == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "code id is required")
	}
	// Validate label
	if err := ValidateLabel(p.Label); err != nil {
		return err
	}
	// Validate msg
	if err := p.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	// Validate funds
	if !p.Funds.IsValid() {
		return sdkerrors.ErrInvalidCoins
	}
	// Validate salt
	if len(p.Salt) == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "salt is required")
	}
	return nil
}

// String implements the Stringer interface.
func (p InstantiateContract2Proposal) String() string {
	return fmt.Sprintf(`Instantiate Code Proposal:
  Title:       %s
  Description: %s
  Run as:      %s
  Admin:       %s
  Code id:     %d
  Label:       %s
  Msg:         %q
  Funds:       %s
  Salt:        %X
`, p.Title, p.Description, p.RunAs, p.Admin, p.CodeID, p.Label, p.Msg, p.Funds, p.Salt)
}

// MarshalYAML pretty prints the init message
func (p InstantiateContract2Proposal) MarshalYAML() (interface{}, error) {
	return struct {
		Title       string    `yaml:"title"`
		Description string    `yaml:"description"`
		RunAs       string    `yaml:"run_as"`
		Admin       string    `yaml:"admin"`
		CodeID      uint64    `yaml:"code_id"`
		Label       string    `yaml:"label"`
		Msg         string    `yaml:"msg"`
		Funds       sdk.Coins `yaml:"funds"`
		Salt        string    `yaml:"salt"`
	}{
		Title:       p.Title,
		Description: p.Description,
		RunAs:       p.RunAs,
		Admin:       p.Admin,
		CodeID:      p.CodeID,
		Label:       p.Label,
		Msg:         string(p.Msg),
		Funds:       p.Funds,
		Salt:        base64.StdEncoding.EncodeToString(p.Salt),
	}, nil
}

func NewStoreAndInstantiateContractProposal(
	title string,
	description string,
	runAs string,
	wasmBz []byte,
	source string,
	builder string,
	codeHash []byte,
	permission *AccessConfig,
	unpinCode bool,
	admin string,
	label string,
	msg RawContractMessage,
	funds sdk.Coins,
) *StoreAndInstantiateContractProposal {
	return &StoreAndInstantiateContractProposal{
		Title:                 title,
		Description:           description,
		RunAs:                 runAs,
		WASMByteCode:          wasmBz,
		Source:                source,
		Builder:               builder,
		CodeHash:              codeHash,
		InstantiatePermission: permission,
		UnpinCode:             unpinCode,
		Admin:                 admin,
		Label:                 label,
		Msg:                   msg,
		Funds:                 funds,
	}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p StoreAndInstantiateContractProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *StoreAndInstantiateContractProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p StoreAndInstantiateContractProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p StoreAndInstantiateContractProposal) ProposalType() string {
	return string(ProposalTypeStoreAndInstantiateContractProposal)
}

// ValidateBasic validates the proposal
func (p StoreAndInstantiateContractProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if _, err := sdk.AccAddressFromBech32(p.RunAs); err != nil {
		return errorsmod.Wrap(err, "run as")
	}

	if err := validateWasmCode(p.WASMByteCode, MaxProposalWasmSize); err != nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "code bytes %s", err.Error())
	}

	if err := ValidateVerificationInfo(p.Source, p.Builder, p.CodeHash); err != nil {
		return errorsmod.Wrap(err, "code info")
	}

	if p.InstantiatePermission != nil {
		if err := p.InstantiatePermission.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "instantiate permission")
		}
	}

	if err := ValidateLabel(p.Label); err != nil {
		return err
	}

	if !p.Funds.IsValid() {
		return sdkerrors.ErrInvalidCoins
	}

	if len(p.Admin) != 0 {
		if _, err := sdk.AccAddressFromBech32(p.Admin); err != nil {
			return err
		}
	}
	if err := p.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

// String implements the Stringer interface.
func (p StoreAndInstantiateContractProposal) String() string {
	return fmt.Sprintf(`Store And Instantiate Coontract Proposal:
  Title:       %s
  Description: %s
  Run as:      %s
  WasmCode:    %X
  Source:      %s
  Builder:     %s
  Code Hash:   %X
  Instantiate permission: %s
  Unpin code:  %t  
  Admin:       %s
  Label:       %s
  Msg:         %q
  Funds:       %s
`, p.Title, p.Description, p.RunAs, p.WASMByteCode, p.Source, p.Builder, p.CodeHash, p.InstantiatePermission, p.UnpinCode, p.Admin, p.Label, p.Msg, p.Funds)
}

// MarshalYAML pretty prints the wasm byte code and the init message
func (p StoreAndInstantiateContractProposal) MarshalYAML() (interface{}, error) {
	return struct {
		Title                 string        `yaml:"title"`
		Description           string        `yaml:"description"`
		RunAs                 string        `yaml:"run_as"`
		WASMByteCode          string        `yaml:"wasm_byte_code"`
		Source                string        `yaml:"source"`
		Builder               string        `yaml:"builder"`
		CodeHash              string        `yaml:"code_hash"`
		InstantiatePermission *AccessConfig `yaml:"instantiate_permission"`
		UnpinCode             bool          `yaml:"unpin_code"`
		Admin                 string        `yaml:"admin"`
		Label                 string        `yaml:"label"`
		Msg                   string        `yaml:"msg"`
		Funds                 sdk.Coins     `yaml:"funds"`
	}{
		Title:                 p.Title,
		Description:           p.Description,
		RunAs:                 p.RunAs,
		WASMByteCode:          base64.StdEncoding.EncodeToString(p.WASMByteCode),
		InstantiatePermission: p.InstantiatePermission,
		UnpinCode:             p.UnpinCode,
		Admin:                 p.Admin,
		Label:                 p.Label,
		Source:                p.Source,
		Builder:               p.Builder,
		CodeHash:              hex.EncodeToString(p.CodeHash),
		Msg:                   string(p.Msg),
		Funds:                 p.Funds,
	}, nil
}

func NewMigrateContractProposal(
	title string,
	description string,
	contract string,
	codeID uint64,
	msg RawContractMessage,
) *MigrateContractProposal {
	return &MigrateContractProposal{
		Title:       title,
		Description: description,
		Contract:    contract,
		CodeID:      codeID,
		Msg:         msg,
	}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p MigrateContractProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *MigrateContractProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p MigrateContractProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p MigrateContractProposal) ProposalType() string { return string(ProposalTypeMigrateContract) }

// ValidateBasic validates the proposal
func (p MigrateContractProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if p.CodeID == 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "code_id is required")
	}
	if _, err := sdk.AccAddressFromBech32(p.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	if err := p.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

// String implements the Stringer interface.
func (p MigrateContractProposal) String() string {
	return fmt.Sprintf(`Migrate Contract Proposal:
  Title:       %s
  Description: %s
  Contract:    %s
  Code id:     %d
  Msg:         %q
`, p.Title, p.Description, p.Contract, p.CodeID, p.Msg)
}

// MarshalYAML pretty prints the migrate message
func (p MigrateContractProposal) MarshalYAML() (interface{}, error) {
	return struct {
		Title       string `yaml:"title"`
		Description string `yaml:"description"`
		Contract    string `yaml:"contract"`
		CodeID      uint64 `yaml:"code_id"`
		Msg         string `yaml:"msg"`
	}{
		Title:       p.Title,
		Description: p.Description,
		Contract:    p.Contract,
		CodeID:      p.CodeID,
		Msg:         string(p.Msg),
	}, nil
}

func NewSudoContractProposal(
	title string,
	description string,
	contract string,
	msg RawContractMessage,
) *SudoContractProposal {
	return &SudoContractProposal{
		Title:       title,
		Description: description,
		Contract:    contract,
		Msg:         msg,
	}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p SudoContractProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *SudoContractProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p SudoContractProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p SudoContractProposal) ProposalType() string { return string(ProposalTypeSudoContract) }

// ValidateBasic validates the proposal
func (p SudoContractProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if _, err := sdk.AccAddressFromBech32(p.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	if err := p.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

// String implements the Stringer interface.
func (p SudoContractProposal) String() string {
	return fmt.Sprintf(`Migrate Contract Proposal:
  Title:       %s
  Description: %s
  Contract:    %s
  Msg:         %q
`, p.Title, p.Description, p.Contract, p.Msg)
}

// MarshalYAML pretty prints the migrate message
func (p SudoContractProposal) MarshalYAML() (interface{}, error) {
	return struct {
		Title       string `yaml:"title"`
		Description string `yaml:"description"`
		Contract    string `yaml:"contract"`
		Msg         string `yaml:"msg"`
	}{
		Title:       p.Title,
		Description: p.Description,
		Contract:    p.Contract,
		Msg:         string(p.Msg),
	}, nil
}

func NewExecuteContractProposal(
	title string,
	description string,
	runAs string,
	contract string,
	msg RawContractMessage,
	funds sdk.Coins,
) *ExecuteContractProposal {
	return &ExecuteContractProposal{title, description, runAs, contract, msg, funds}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p ExecuteContractProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *ExecuteContractProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p ExecuteContractProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p ExecuteContractProposal) ProposalType() string { return string(ProposalTypeExecuteContract) }

// ValidateBasic validates the proposal
func (p ExecuteContractProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if _, err := sdk.AccAddressFromBech32(p.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	if _, err := sdk.AccAddressFromBech32(p.RunAs); err != nil {
		return errorsmod.Wrap(err, "run as")
	}
	if !p.Funds.IsValid() {
		return sdkerrors.ErrInvalidCoins
	}
	if err := p.Msg.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "payload msg")
	}
	return nil
}

// String implements the Stringer interface.
func (p ExecuteContractProposal) String() string {
	return fmt.Sprintf(`Migrate Contract Proposal:
  Title:       %s
  Description: %s
  Contract:    %s
  Run as:      %s
  Msg:         %q
  Funds:       %s
`, p.Title, p.Description, p.Contract, p.RunAs, p.Msg, p.Funds)
}

// MarshalYAML pretty prints the migrate message
func (p ExecuteContractProposal) MarshalYAML() (interface{}, error) {
	return struct {
		Title       string    `yaml:"title"`
		Description string    `yaml:"description"`
		Contract    string    `yaml:"contract"`
		Msg         string    `yaml:"msg"`
		RunAs       string    `yaml:"run_as"`
		Funds       sdk.Coins `yaml:"funds"`
	}{
		Title:       p.Title,
		Description: p.Description,
		Contract:    p.Contract,
		Msg:         string(p.Msg),
		RunAs:       p.RunAs,
		Funds:       p.Funds,
	}, nil
}

func NewUpdateAdminProposal(
	title string,
	description string,
	newAdmin string,
	contract string,
) *UpdateAdminProposal {
	return &UpdateAdminProposal{title, description, newAdmin, contract}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p UpdateAdminProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *UpdateAdminProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p UpdateAdminProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p UpdateAdminProposal) ProposalType() string { return string(ProposalTypeUpdateAdmin) }

// ValidateBasic validates the proposal
func (p UpdateAdminProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if _, err := sdk.AccAddressFromBech32(p.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	if _, err := sdk.AccAddressFromBech32(p.NewAdmin); err != nil {
		return errorsmod.Wrap(err, "new admin")
	}
	return nil
}

// String implements the Stringer interface.
func (p UpdateAdminProposal) String() string {
	return fmt.Sprintf(`Update Contract Admin Proposal:
  Title:       %s
  Description: %s
  Contract:    %s
  New Admin:   %s
`, p.Title, p.Description, p.Contract, p.NewAdmin)
}

func NewClearAdminProposal(
	title string,
	description string,
	contract string,
) *ClearAdminProposal {
	return &ClearAdminProposal{title, description, contract}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p ClearAdminProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *ClearAdminProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p ClearAdminProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p ClearAdminProposal) ProposalType() string { return string(ProposalTypeClearAdmin) }

// ValidateBasic validates the proposal
func (p ClearAdminProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if _, err := sdk.AccAddressFromBech32(p.Contract); err != nil {
		return errorsmod.Wrap(err, "contract")
	}
	return nil
}

// String implements the Stringer interface.
func (p ClearAdminProposal) String() string {
	return fmt.Sprintf(`Clear Contract Admin Proposal:
  Title:       %s
  Description: %s
  Contract:    %s
`, p.Title, p.Description, p.Contract)
}

func NewPinCodesProposal(
	title string,
	description string,
	codeIDs []uint64,
) *PinCodesProposal {
	return &PinCodesProposal{
		Title:       title,
		Description: description,
		CodeIDs:     codeIDs,
	}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p PinCodesProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *PinCodesProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p PinCodesProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p PinCodesProposal) ProposalType() string { return string(ProposalTypePinCodes) }

// ValidateBasic validates the proposal
func (p PinCodesProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if len(p.CodeIDs) == 0 {
		return errorsmod.Wrap(ErrEmpty, "code ids")
	}
	return nil
}

// String implements the Stringer interface.
func (p PinCodesProposal) String() string {
	return fmt.Sprintf(`Pin Wasm Codes Proposal:
  Title:       %s
  Description: %s
  Codes:       %v
`, p.Title, p.Description, p.CodeIDs)
}

func NewUnpinCodesProposal(
	title string,
	description string,
	codeIDs []uint64,
) *UnpinCodesProposal {
	return &UnpinCodesProposal{
		Title:       title,
		Description: description,
		CodeIDs:     codeIDs,
	}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p UnpinCodesProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *UnpinCodesProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p UnpinCodesProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p UnpinCodesProposal) ProposalType() string { return string(ProposalTypeUnpinCodes) }

// ValidateBasic validates the proposal
func (p UnpinCodesProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if len(p.CodeIDs) == 0 {
		return errorsmod.Wrap(ErrEmpty, "code ids")
	}
	return nil
}

// String implements the Stringer interface.
func (p UnpinCodesProposal) String() string {
	return fmt.Sprintf(`Unpin Wasm Codes Proposal:
  Title:       %s
  Description: %s
  Codes:       %v
`, p.Title, p.Description, p.CodeIDs)
}

func validateProposalCommons(title, description string) error {
	if strings.TrimSpace(title) != title {
		return errorsmod.Wrap(govtypes.ErrInvalidProposalContent, "proposal title must not start/end with white spaces")
	}
	if len(title) == 0 {
		return errorsmod.Wrap(govtypes.ErrInvalidProposalContent, "proposal title cannot be blank")
	}
	if len(title) > v1beta1.MaxTitleLength {
		return errorsmod.Wrapf(govtypes.ErrInvalidProposalContent, "proposal title is longer than max length of %d", v1beta1.MaxTitleLength)
	}
	if strings.TrimSpace(description) != description {
		return errorsmod.Wrap(govtypes.ErrInvalidProposalContent, "proposal description must not start/end with white spaces")
	}
	if len(description) == 0 {
		return errorsmod.Wrap(govtypes.ErrInvalidProposalContent, "proposal description cannot be blank")
	}
	if len(description) > v1beta1.MaxDescriptionLength {
		return errorsmod.Wrapf(govtypes.ErrInvalidProposalContent, "proposal description is longer than max length of %d", v1beta1.MaxDescriptionLength)
	}
	return nil
}

func NewUpdateInstantiateConfigProposal(
	title string,
	description string,
	accessConfigUpdates ...AccessConfigUpdate,
) *UpdateInstantiateConfigProposal {
	return &UpdateInstantiateConfigProposal{
		Title:               title,
		Description:         description,
		AccessConfigUpdates: accessConfigUpdates,
	}
}

// ProposalRoute returns the routing key of a parameter change proposal.
func (p UpdateInstantiateConfigProposal) ProposalRoute() string { return RouterKey }

// GetTitle returns the title of the proposal
func (p *UpdateInstantiateConfigProposal) GetTitle() string { return p.Title }

// GetDescription returns the human readable description of the proposal
func (p UpdateInstantiateConfigProposal) GetDescription() string { return p.Description }

// ProposalType returns the type
func (p UpdateInstantiateConfigProposal) ProposalType() string {
	return string(ProposalTypeUpdateInstantiateConfig)
}

// ValidateBasic validates the proposal
func (p UpdateInstantiateConfigProposal) ValidateBasic() error {
	if err := validateProposalCommons(p.Title, p.Description); err != nil {
		return err
	}
	if len(p.AccessConfigUpdates) == 0 {
		return errorsmod.Wrap(ErrEmpty, "code updates")
	}
	dedup := make(map[uint64]bool)
	for _, codeUpdate := range p.AccessConfigUpdates {
		_, found := dedup[codeUpdate.CodeID]
		if found {
			return errorsmod.Wrapf(ErrDuplicate, "duplicate code: %d", codeUpdate.CodeID)
		}
		if err := codeUpdate.InstantiatePermission.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "instantiate permission")
		}
		dedup[codeUpdate.CodeID] = true
	}
	return nil
}

// String implements the Stringer interface.
func (p UpdateInstantiateConfigProposal) String() string {
	return fmt.Sprintf(`Update Instantiate Config Proposal:
  Title:       %s
  Description: %s
  AccessConfigUpdates: %v
`, p.Title, p.Description, p.AccessConfigUpdates)
}

// String implements the Stringer interface.
func (c AccessConfigUpdate) String() string {
	return fmt.Sprintf(`AccessConfigUpdate:
  CodeID:       %d
  AccessConfig: %v
`, c.CodeID, c.InstantiatePermission)
}
