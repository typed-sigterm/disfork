# DisFork ![Latest version](https://img.shields.io/github/v/release/typed-sigterm/disfork) ![License](https://img.shields.io/github/license/typed-sigterm/disfork) ![OSS Lifecycle](https://img.shields.io/osslifecycle?file_url=https%3A%2F%2Fraw.githubusercontent.com%2Ftyped-sigterm%2Fdisfork%2Fmain%2FOSSMETADATA) [![GitHub Stars](https://img.shields.io/github/stars/typed-sigterm/disfork)](https://github.com/typed-sigterm/disfork)

Clean up your useless GitHub forks.

## Usage

Just run `disfork` and follow the interactive prompts ✨

```
❯ disfork --help
Clean up your useless GitHub forks

Usage: disfork.exe [OPTIONS]

Options:
      --github-token <GITHUB_TOKEN>    GitHub access token (overrides GitHub App authorization) [env: GITHUB_TOKEN=]
      --app-slug <APP_SLUG>            GitHub App slug (to get it: https://github.com/apps/<SLUG_HERE>) [default: disfork]
      --app-client-id <APP_CLIENT_ID>  GitHub App client ID [default: Iv23licpLWlZABwjnLK7]        
      --account <ACCOUNT>              GitHub user or organization to scan (defaults to authenticated user)
      --auto                           Skip interactive selection and delete all useless forks     
      --parallel <PARALLEL>            Number of parallel fetching tasks [default: 6]
      --dry-run                        Don't actually delete anything
  -h, --help                           Print help
  -V, --version                        Print version
```