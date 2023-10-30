use super::{
    rejection::{FailedToResolveHost, HostRejection},
    FromRequest, RequestParts,
};
use async_trait::async_trait;
use http::header::{HeaderMap, FORWARDED};

const X_FORWARDED_HOST_HEADER_KEY: &str = "X-Forwarded-Host";

/// Extractor that resolves the hostname of the request.
///
/// Hostname is resolved through the following, in order:
/// - `Forwarded` header
/// - `X-Forwarded-Host` header
/// - `Host` header
/// - request target / URI
///
/// Note that user agents can set `X-Forwarded-Host` and `Host` headers to arbitrary values so make
/// sure to validate them to avoid security issues.
#[derive(Debug, Clone)]
pub struct Host(pub String);

#[async_trait]
impl<B> FromRequest<B> for Host
where
    B: Send,
{
    type Rejection = HostRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if let Some(host) = parse_forwarded(req.headers()) {
            return Ok(Host(host.to_owned()));
        }

        if let Some(host) = req
            .headers()
            .get(X_FORWARDED_HOST_HEADER_KEY)
            .and_then(|host| host.to_str().ok())
        {
            return Ok(Host(host.to_owned()));
        }

        if let Some(host) = req
            .headers()
            .get(http::header::HOST)
            .and_then(|host| host.to_str().ok())
        {
            return Ok(Host(host.to_owned()));
        }

        if let Some(host) = req.uri().host() {
            return Ok(Host(host.to_owned()));
        }

        Err(HostRejection::FailedToResolveHost(FailedToResolveHost))
    }
}

#[allow(warnings)]
fn parse_forwarded(headers: &HeaderMap) -> Option<&str> {
    // if there are multiple `Forwarded` `HeaderMap::get` will return the first one
    let forwarded_values = headers.get(FORWARDED)?.to_str().ok()?;

    // get the first set of values
    let first_value = forwarded_values.split(',').nth(0)?;

    // find the value of the `host` field
    first_value.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("host")
            .then(|| value.trim().trim_matches('"'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{routing::get, test_helpers::TestClient, Router};
    use http::header::HeaderName;

    fn test_client() -> TestClient {
        async fn host_as_body(Host(host): Host) -> String {
            host
        }

        TestClient::new(Router::new().route("/", get(host_as_body)))
    }

    #[tokio::test]
    async fn host_header() {
        let original_host = "some-domain:123";
        let host = test_client()
            .get("/")
            .header(http::header::HOST, original_host)
            .send()
            .await
            .text()
            .await;
        assert_eq!(host, original_host);
    }

    #[tokio::test]
    async fn x_forwarded_host_header() {
        let original_host = "some-domain:456";
        let host = test_client()
            .get("/")
            .header(X_FORWARDED_HOST_HEADER_KEY, original_host)
            .send()
            .await
            .text()
            .await;
        assert_eq!(host, original_host);
    }

    #[tokio::test]
    async fn x_forwarded_host_precedence_over_host_header() {
        let x_forwarded_host_header = "some-domain:456";
        let host_header = "some-domain:123";
        let host = test_client()
            .get("/")
            .header(X_FORWARDED_HOST_HEADER_KEY, x_forwarded_host_header)
            .header(http::header::HOST, host_header)
            .send()
            .await
            .text()
            .await;
        assert_eq!(host, x_forwarded_host_header);
    }

    #[tokio::test]
    async fn uri_host() {
        let host = test_client().get("/").send().await.text().await;
        assert!(host.contains("127.0.0.1"));
    }

    #[test]
    fn forwarded_parsing() {
        // the basic case
        let headers = header_map(&[(FORWARDED, "host=192.0.2.60;proto=http;by=203.0.113.43")]);
        let value = parse_forwarded(&headers).unwrap();
        assert_eq!(value, "192.0.2.60");

        // is case insensitive
        let headers = header_map(&[(FORWARDED, "host=192.0.2.60;proto=http;by=203.0.113.43")]);
        let value = parse_forwarded(&headers).unwrap();
        assert_eq!(value, "192.0.2.60");

        // ipv6
        let headers = header_map(&[(FORWARDED, "host=\"[2001:db8:cafe::17]:4711\"")]);
        let value = parse_forwarded(&headers).unwrap();
        assert_eq!(value, "[2001:db8:cafe::17]:4711");

        // multiple values in one header
        let headers = header_map(&[(FORWARDED, "host=192.0.2.60, host=127.0.0.1")]);
        let value = parse_forwarded(&headers).unwrap();
        assert_eq!(value, "192.0.2.60");

        // multiple header values
        let headers = header_map(&[
            (FORWARDED, "host=192.0.2.60"),
            (FORWARDED, "host=127.0.0.1"),
        ]);
        let value = parse_forwarded(&headers).unwrap();
        assert_eq!(value, "192.0.2.60");
    }

    fn header_map(values: &[(HeaderName, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (key, value) in values {
            headers.append(key, value.parse().unwrap());
        }
        headers
    }
}
