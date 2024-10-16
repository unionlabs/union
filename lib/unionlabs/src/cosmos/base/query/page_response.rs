use macros::model;

#[model(proto(raw(protos::cosmos::base::query::v1beta1::PageResponse), into))]
pub struct PageResponse {
    /// `next_key` is the key to be passed to PageRequest.key to
    /// query the next page most efficiently. It will be empty if
    /// there are no more results.
    pub next_key: Vec<u8>,
    /// total is total number of results available if `PageRequest.count_total`
    /// was set, its value is undefined otherwise
    pub total: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::cosmos::base::query::page_response::PageResponse;

    impl From<protos::cosmos::base::query::v1beta1::PageResponse> for PageResponse {
        fn from(value: protos::cosmos::base::query::v1beta1::PageResponse) -> Self {
            Self {
                next_key: value.next_key,
                total: value.total,
            }
        }
    }

    impl From<PageResponse> for protos::cosmos::base::query::v1beta1::PageResponse {
        fn from(value: PageResponse) -> Self {
            Self {
                next_key: value.next_key,
                total: value.total,
            }
        }
    }
}
