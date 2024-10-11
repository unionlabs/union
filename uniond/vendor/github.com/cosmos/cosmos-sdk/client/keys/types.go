package keys

// used for outputting keyring.LegacyInfo over REST

// AddNewKey request a new key
type AddNewKey struct {
	Name     string `json:"name"`
	Password string `json:"password"`
	Mnemonic string `json:"mnemonic"`
	Account  int    `json:"account,string,omitempty"`
	Index    int    `json:"index,string,omitempty"`
}

// NewAddNewKey constructs a new AddNewKey request structure.
func NewAddNewKey(name, password, mnemonic string, account, index int) AddNewKey {
	return AddNewKey{
		Name:     name,
		Password: password,
		Mnemonic: mnemonic,
		Account:  account,
		Index:    index,
	}
}

// RecoverKey recovers a key
type RecoverKey struct {
	Password string `json:"password"`
	Mnemonic string `json:"mnemonic"`
	Account  int    `json:"account,string,omitempty"`
	Index    int    `json:"index,string,omitempty"`
}

// NewRecoverKey constructs a new RecoverKey request structure.
func NewRecoverKey(password, mnemonic string, account, index int) RecoverKey {
	return RecoverKey{Password: password, Mnemonic: mnemonic, Account: account, Index: index}
}

// UpdateKeyReq requests updating a key
type UpdateKeyReq struct {
	OldPassword string `json:"old_password"`
	NewPassword string `json:"new_password"`
}

// NewUpdateKeyReq constructs a new UpdateKeyReq structure.
func NewUpdateKeyReq(old, new string) UpdateKeyReq {
	return UpdateKeyReq{OldPassword: old, NewPassword: new}
}

// DeleteKeyReq requests deleting a key
type DeleteKeyReq struct {
	Password string `json:"password"`
}

// NewDeleteKeyReq constructs a new DeleteKeyReq structure.
func NewDeleteKeyReq(password string) DeleteKeyReq { return DeleteKeyReq{Password: password} }
