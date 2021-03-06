# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.


### v0.8.0 (2018-04-23)

- [added] Add `ApiBuilder::with_custom_endpoint` method
- [changed] Require Rust 1.21+
- [changed] Constructors of `SimpleApi` and `E2eApi` are now private, use the
  `ApiBuilder` instead
- [changed] Upgrade sodiumoxide dependency to 0.0.16
- [changed] Upgrade reqwest dependency to 0.8
- [changed] Upgrade log dependency to 0.4
- [changed] Upgrade data-encoding dependency to 2.1

### v0.7.1 (2017-08-28)

- [changed] Upgrade data-encoding dependency to 2.0.0-rc.2
- [changed] Upgrade mime_guess dependency to 2.0.0-alpha.2

### v0.7.0 (2017-07-17)

- [changed] Update reqwest dependency to 0.7
- [changed] Update mime dependency to 0.3
- [changed] Require Rust 1.18+

### v0.6.0 (2017-05-29)

- [changed] Upgrade serde to 1.0.0

### v0.5.0 (2017-04-20)

- [changed] Require Rust 1.15+
- [added] Implement `lookup_credits` (#8)
- [added] Implement `lookup_capabilities` (#9)
- [added] Implement blob uploading (#11)
- [added] Implement encrypting of e2e text, image and file messages (#11)

### v0.4.1 (2017-04-12)

- [added] Add `as_bytes` and `Into<String>` to `RecipientKey` (#7)

### v0.4.0 (2017-04-10)

- [changed] Revamped and simplified entire API, it now uses a more
  object-oriented approach (#6)

### v0.3.2 (2017-04-06)

- [fixed] Fix bad API URL
- [changed] Update reqwest dependency to 0.5

### v0.3.1 (2017-04-04)

- [added] Add debug logging to lookups

### v0.3.0 (2017-03-22)

- [changed] Upgrade some dependencies
- [changed] Move from hyper to reqwest
- [changed] Require Rust 1.13+

### v0.2.0 (2016-12-15)

- [added] Implement Threema ID lookups

### v0.1.0 (2016-12-15)

- Initial release
