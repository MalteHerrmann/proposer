# Changelog

## Unreleased

### Features

- (command) [#25](https://github.com/evmos/proposer/pull/25) Add Commonwealth link to submitted proposal.
- (cli) [#21](https://github.com/evmos/proposer/pull/21) Select key to submit proposal.
- (tests) [#18](https://github.com/evmos/proposer/pull/18) Add mocking for standard `GET` requests.
- (prop) [#17](https://github.com/evmos/proposer/pull/17) Get summary of release notes using GPT-4.
- (cli) [#1](https://github.com/evmos/proposer/pull/1) Prepare CLI command to submit proposal.

### Improvements

- (command) [#40](https://github.com/evmos/proposer/pull/40) Improve handling of Commonwealth info.
- (block) [#39](https://github.com/evmos/proposer/pull/39) Round upgrade height to nearest 500.
- (llm) [#38](https://github.com/evmos/proposer/pull/38) Enable using GPT-4o.
- (prop) [#23](https://github.com/evmos/proposer/pull/23) Get keyring location from user input.
- (prop) [#22](https://github.com/evmos/proposer/pull/22) Choose GPT model via CLI flag.
- (crate) [#16](https://github.com/evmos/proposer/pull/16) Adjust package name to `proposer`.
- (block) [#15](https://github.com/evmos/proposer/pull/15) Add error handling for blocks data.
- (all) [#13](https://github.com/evmos/proposer/pull/13) Implement better error handling using [thiserror](https://github.com/dtolnay/thiserror).
- (cli) [#12](https://github.com/evmos/proposer/pull/12) Use [clap](https://github.com/clap-rs/clap) for CLI handling.
- (tests) [#11](https://github.com/evmos/proposer/pull/11) Mock API responses for tests.

### Bug Fixes

- (command) [#33](https://github.com/evmos/proposer/pull/33) Use correct network denom for transaction fees.
- (rpc) [#32](https://github.com/evmos/proposer/pull/32) Use port 443 on Lava RPC endpoints.

## [v0.1.0](https://github.com/evmos/proposer/releases/tag/v0.1.0) - 2024-08-05

This section contains the legacy changelog from the [original repo](https://github.com/MalteHerrmann/upgrade-helper)

### Features

- (prop) [#5](https://github.com/MalteHerrmann/upgrade-helper/pull/5) Prepare proposal contents depending on input.
- (cli) [#3](https://github.com/MalteHerrmann/upgrade-helper/pull/3) Add basic CLI structure.

### Improvements

- (prop) [#12](https://github.com/MalteHerrmann/upgrade-helper/pull/12) Add version and height links to proposal.
- (github) [#11](https://github.com/MalteHerrmann/upgrade-helper/pull/11) Use octocrab to access GitHub data.
- (chain) [#10](https://github.com/MalteHerrmann/upgrade-helper/pull/10) Estimate the upgrade height based on last 50.000 blocks.
- (cli) [#9](https://github.com/MalteHerrmann/upgrade-helper/pull/9) Add logic for selection and calculation of upgrade date and time.
- (github) [#4](https://github.com/MalteHerrmann/upgrade-helper/pull/4) Check if release already exists on GitHub.
