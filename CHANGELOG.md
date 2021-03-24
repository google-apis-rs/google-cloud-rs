Changelog
=========

All notable changes to **`google-cloud-rs`** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

Unreleased
----------

### Added

### Removed

### Fixed

### Changed

v0.2.0 - 2021-03-24
-------------------

### Added

- [pubsub] Added `Subscription::receive_with_options` (#33)

### Removed

### Fixed

- [storage] Fixed error when bucket listing turns out to be empty (#42)
- [pubsub] Changed default `max_messages` when pulling messages from `5` to `1` (#44)

### Changed

- Upgraded all dependencies (#35 and #41)

v0.1.0 - 2020-08-23
-------------------

This is the initial release.
