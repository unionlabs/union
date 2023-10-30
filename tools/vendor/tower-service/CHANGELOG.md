# Unreleased

- None

# 0.3.2 (June 17, 2022)

## Added

- **docs**: Clarify subtlety around cloning and readiness in the `Service` docs
  ([#548])
- **docs**: Clarify details around shared resource consumption in `poll_ready()`
  ([#662])


[#548]: https://github.com/tower-rs/tower/pull/548
[#662]: https://github.com/tower-rs/tower/pull/662


# 0.3.1 (November 29, 2019)

- Improve example in `Service` docs. ([#510])

[#510]: https://github.com/tower-rs/tower/pull/510

# 0.3.0 (November 29, 2019)

- Update to `futures 0.3`.
- Update documentation for `std::future::Future`.

# 0.3.0-alpha.2 (September 30, 2019)

- Documentation fixes.

# 0.3.0-alpha.1 (Aug 20, 2019)

* Switch to `std::future::Future`

# 0.2.0 (Dec 12, 2018)

* Change `Service`'s `Request` associated type to be a generic instead.
  * Before:

    ```rust
    impl Service for Client {
        type Request = HttpRequest;
        type Response = HttpResponse;
        // ...
    }
    ```
  * After:

    ```rust
    impl Service<HttpRequest> for Client {
        type Response = HttpResponse;
        // ...
    }
    ```
* Remove `NewService`, use `tower_util::MakeService` instead.
* Remove `Service::ready` and `Ready`, use `tower_util::ServiceExt` instead.

# 0.1.0 (Aug 9, 2018)

* Initial release
