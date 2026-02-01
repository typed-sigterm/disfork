# DisFork ![Latest version](https://img.shields.io/github/v/release/typed-sigterm/disfork) ![License](https://img.shields.io/github/license/typed-sigterm/disfork) ![OSS Lifecycle](https://img.shields.io/osslifecycle?file_url=https%3A%2F%2Fraw.githubusercontent.com%2Ftyped-sigterm%2Fdisfork%2Fmain%2FOSSMETADATA) [![GitHub Stars](https://img.shields.io/github/stars/typed-sigterm/disfork)](https://github.com/typed-sigterm/disfork)

Clean up your useless GitHub forks.

## Installation

See [release page](https://github.com/typed-sigterm/disfork/releases/latest).

## Usage

Just run `disfork` and follow the interactive prompts ✨

```
❯ disfork

🧹 DisFork - Clean up your useless GitHub forks

ℹ Please install the GitHub App on your personal account:
ℹ Visit: https://github.com/apps/disfork/installations/select_target
ℹ After installation, press Enter to continue...

→ Please visit: https://github.com/login/device
→ And enter code: 38C3-1452

Waiting for authorization...
✓ Authorization successful!
ℹ Found 14 fork repositories
→ 2 are useless, selected by default

✔ Select repositories to delete (Space to toggle, Enter to confirm)
ℹ Selected 3 repositories for deletion:
  - typed-sigterm/dokploy
  - typed-sigterm/better-auth
  - typed-sigterm/nitro

✔ Are you sure you want to delete 3 repositories? · yes

⏳ batch deletion cooldown period...
Ready! [████████████████████████████████████████] 20s/20s
✓ All done!
```

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
      --skip-cooldown                  Skip the cooldown period before deletion
  -h, --help                           Print help
  -V, --version                        Print version
```
