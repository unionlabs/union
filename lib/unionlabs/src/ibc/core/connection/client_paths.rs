use macros::model;

#[model(proto(raw(protos::ibc::core::connection::v1::ClientPaths), into, from))]
pub struct ClientPaths {
    pub paths: Vec<String>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::ibc::core::connection::client_paths::ClientPaths;

    impl From<protos::ibc::core::connection::v1::ClientPaths> for ClientPaths {
        fn from(value: protos::ibc::core::connection::v1::ClientPaths) -> Self {
            Self { paths: value.paths }
        }
    }

    impl From<ClientPaths> for protos::ibc::core::connection::v1::ClientPaths {
        fn from(value: ClientPaths) -> Self {
            protos::ibc::core::connection::v1::ClientPaths { paths: value.paths }
        }
    }
}
