# Proposer

This repository contains a helper to aid in generating the contents of Software Upgrade Proposals. It does so by

- [x] Querying the release on GitHub
- [x] Deriving the changed features from the release notes
- [x] Calculating the block height for the desired time of upgrade
- [x] Updating the description template
- [ ] Creating a Commonwealth thread
- [x] Creating the CLI command to submit the proposal
- [ ] Creating a Notion page that contains the proposal text as well as the CLI command

## Usage

**NOTE:** Because the Commonwealth integration is not yet implemented (an API key is already requested)
it is not possible to run all of this in one go.

It is rather required, to first _generate the proposal_, which creates a Markdown file with the proposal description,
that need to be posted on Commonwealth for mainnet upgrades.

```yaml
 $ ./proposer generate-proposal -h

Usage: proposer generate-proposal [OPTIONS]

Options:
  -m, --model <MODEL>  The LLM model to use for summarizing the release notes [default: gpt4o] [possible values: gpt4o]
  -h, --help           Print help
```

As a second step, it is required to _generate the shell command_ to submit the proposal.
It is written to a `.sh` file in your current working directory.

```yaml
 $ ./proposer generate-command -h

Usage: proposer generate-command [OPTIONS]

Options:
  -c, --config <CONFIG>  The path to the configuration file
  -h, --help             Print help
```


## Requirements

- **Rust** and **Cargo**

    You will need to have a somewhat recent version of the Rust toolchain installed.

- **OpenAI API Key**

    The tool is using OpenAI's LLMs to generate a summary of the changes in the release(s).
    To use this feature, ensure that you run the binary in an environment where `OPENAI_API_KEY` is set.

- **Configured `.evmosd` Home**

    To generate a shell command that can be instantly used,
    the tool is checking `$HOME/.evmosd` for the configured keyring.
    This keyring is then used to get the list of available keys.
    It is checked, which of those keys hold a balance on the selected network (mainnet/testnet)
    and lets the user select the one to execute the command with if there are multiple.
    To use this feature, ensure that you have your `$HOME/.evmosd` configuration set
    so that the configured keyring holds your mainnet or testnet keys.

## Installation

The tool can be built or installed locally using Rust's `cargo` utility.
Clone the desired version of the repository locally:

```
git clone https://github.com/MalteHerrmann/proposer.git --depth latest
cd proposer
```

To install the binary, run:

```
make install
```

If you do not wish for the binary to be installed in your `$PATH`,
run the build command which creates the binary inside of the `target` directory:

```
make build
```
