use macros::model;

#[model(proto(raw(protos::cosmos::staking::v1beta1::Description), into, from))]
pub struct Description {
    /// moniker defines a human-readable name for the validator.
    pub moniker: String,
    /// identity defines an optional identity signature (ex. `UPort` or `Keybase`).
    pub identity: String,
    /// website defines an optional website link.
    pub website: String,
    /// `security_contact` defines an optional email for security contact.
    pub security_contact: String,
    /// details define other optional details.
    pub details: String,
}

impl From<protos::cosmos::staking::v1beta1::Description> for Description {
    fn from(value: protos::cosmos::staking::v1beta1::Description) -> Self {
        Self {
            moniker: value.moniker,
            identity: value.identity,
            website: value.website,
            security_contact: value.security_contact,
            details: value.details,
        }
    }
}

impl From<Description> for protos::cosmos::staking::v1beta1::Description {
    fn from(value: Description) -> Self {
        Self {
            moniker: value.moniker,
            identity: value.identity,
            website: value.website,
            security_contact: value.security_contact,
            details: value.details,
        }
    }
}
