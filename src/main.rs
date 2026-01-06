mod analyzer;
mod cli;
mod github;

use analyzer::ForkAnalyzer;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use cli::CliInterface;
use github::GitHubClient;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Parser, Debug)]
#[command(name = "DisFork")]
#[command(author = clap::crate_authors!())]
#[command(version = clap::crate_version!())]
#[command(about = clap::crate_description!(), long_about = None)]
struct Args {
    /// GitHub access token (overrides GitHub App authorization)
    #[arg(long, env = "GITHUB_TOKEN")]
    github_token: Option<String>,

    /// GitHub App slug (to get it: https://github.com/apps/<SLUG_HERE>)
    #[arg(long, default_value = "disfork")]
    app_slug: String,

    /// GitHub App client ID
    #[arg(long, default_value = "Iv23licpLWlZABwjnLK7")]
    app_client_id: String,

    /// GitHub user or organization to scan (defaults to authenticated user)
    #[arg(long)]
    account: Option<String>,

    /// Skip interactive selection and delete all useless forks
    #[arg(long)]
    auto: bool,

    /// Number of parallel fetching tasks
    #[arg(long, default_value_t = 6)]
    parallel: usize,

    /// Don't actually delete anything
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let cli = CliInterface::new();

    cli.show_welcome()?;

    let token = if let Some(token) = args.github_token {
        cli.show_info("Using GITHUB_TOKEN from environment")?;
        token
    } else {
        if let Some(account) = &args.account {
            cli.show_info(&format!(
                "Please install the GitHub App on user/org {}:",
                account
            ))?;
        } else {
            cli.show_info("Please install the GitHub App on your personal account:")?;
        }
        cli.show_info(&format!(
            "Visit: https://github.com/apps/{}/installations/select_target",
            args.app_slug
        ))?;
        cli.show_info("After installation, press Enter to continue...")?;
        tokio::task::spawn_blocking(|| {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)
        })
        .await
        .context("Failed to wait for Enter input")??;

        let device_code = GitHubClient::start_device_flow(&args.app_client_id)
            .await
            .context("Failed to start device flow")?;

        cli.show_device_code(&device_code.user_code, &device_code.verification_uri)?;

        let token = GitHubClient::poll_for_token(
            &args.app_client_id,
            &device_code.device_code,
            device_code.interval,
            device_code.expires_in,
        )
        .await
        .context("Failed to get access token")?;

        cli.show_success("Authorization successful!")?;
        token
    };

    let client = GitHubClient::new(token).context("Failed to create GitHub client")?;
    let target_account = if let Some(account) = args.account {
        account
    } else {
        client.current_user().await?
    };

    let spinner = cli.create_spinner("Fetching repositories...")?;
    let repos = client
        .list_repos(&target_account)
        .await
        .context("Failed to list repositories")?;
    let forks: Vec<_> = repos
        .into_iter()
        .filter(|r| r.fork.unwrap_or(false))
        .collect();

    if forks.is_empty() {
        cli.show_success("No fork repositories found!")?;
        return Ok(());
    }

    spinner.finish_with_message(format!("Found {} fork repositories", forks.len()));

    let analyzer = ForkAnalyzer::new(client.clone());
    let pb = cli.create_progress_bar(forks.len() as u64, "Analyzing")?;

    let semaphore = Arc::new(Semaphore::new(args.parallel));
    let mut tasks = tokio::task::JoinSet::new();

    for fork in forks {
        let analyzer = analyzer.clone();
        let pb = pb.clone();
        let sem = semaphore.clone();

        tasks.spawn(async move {
            let _permit = sem
                .acquire()
                .await
                .map_err(|_| anyhow!("Semaphore closed while analyzing repositories"))?;
            let result = analyzer.analyze_fork(fork).await;
            pb.inc(1);
            result
        });
    }

    let mut fork_infos = Vec::new();
    while let Some(result) = tasks.join_next().await {
        fork_infos.push(result??);
    }
    pb.finish_with_message("Analysis complete");

    // 选择要删除的仓库
    let selections = if args.auto {
        fork_infos
            .iter()
            .enumerate()
            .filter(|(_, info)| info.is_useless)
            .map(|(i, _)| i)
            .collect()
    } else {
        cli.select_repos_to_delete(&fork_infos)?
    };

    if selections.is_empty() {
        cli.show_info("No repositories selected for deletion")?;
        return Ok(());
    }

    let selected_repos: Vec<_> = selections.iter().map(|&i| &fork_infos[i]).collect();

    // 显示将要删除的仓库
    cli.show_info(&format!(
        "Selected {} repositories for deletion:",
        selected_repos.len()
    ))?;
    for info in &selected_repos {
        println!("  - {}", info.full_name());
    }

    if args.dry_run {
        cli.show_info("Dry run mode - no repositories will be deleted")?;
        return Ok(());
    }

    // 确认删除
    let is_batch = selected_repos.len() > 1;
    if !cli.confirm_deletion(selected_repos.len(), is_batch)? {
        cli.show_info("Deletion cancelled")?;
        return Ok(());
    }

    // 冷静期
    let cooldown = if is_batch { 20 } else { 5 };
    cli.show_cooldown(cooldown, is_batch).await?;

    // 删除仓库
    let pb = cli.create_progress_bar(selected_repos.len() as u64, "Deleting")?;
    for info in selected_repos {
        let owner = info
            .owner_login()
            .with_context(|| format!("{} is missing owner information", info.full_name()))?;
        let repo_name = info.repo.name.as_str();

        match client.delete_repo(owner, repo_name).await {
            Ok(_) => {
                cli.show_success(&format!("Deleted {}", info.full_name()))?;
            }
            Err(e) => {
                cli.show_error(&format!("Failed to delete {}: {}", info.full_name(), e))?;
            }
        }

        pb.inc(1);
    }
    pb.finish_with_message("Deletion complete");

    cli.show_success("All done!")?;

    Ok(())
}
