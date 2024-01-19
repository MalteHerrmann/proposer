# Proposer

This repository contains a helper to aid in generating the contents of Software Upgrade Proposals. It does so by

- [x] Querying the release on GitHub
- [ ] Deriving the changed features from the release notes
- [x] Calculating the block height for the desired time of upgrade
- [x] Updating the description template
- [ ] Creating a Commonwealth thread
- [ ] Creating the CLI command to submit the proposal
- [ ] Creating a Notion page that contains the proposal text as well as the CLI command

## Installation

The tool can be built or installed using Rust's `cargo` utility.

- `cargo install`
- `cargo build`

## Usage

The following main commands can be used when calling the binary:

1. `./proposer generate-proposal`: Updates the description template with the information for the desired upgrade.
2. `./proposer generate-command`: Creates the CLI command to submit the software upgrade proposal.
