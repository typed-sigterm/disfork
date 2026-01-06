use crate::analyzer::ForkInfo;
use anyhow::{Context, Result};
use console::{Term, style};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct CliInterface {
    term: Term,
    theme: ColorfulTheme,
}

impl CliInterface {
    pub fn new() -> Self {
        Self {
            term: Term::stderr(),
            theme: ColorfulTheme::default(),
        }
    }

    pub fn show_welcome(&self) -> Result<()> {
        self.term.write_line(&format!(
            "\n{} {}\n",
            style("🧹").bold(),
            style(format!("DisFork - {}", clap::crate_description!()))
                .bold()
                .cyan()
        ))?;
        Ok(())
    }

    pub fn show_device_code(&self, user_code: &str, verification_uri: &str) -> Result<()> {
        self.term.write_line(&format!(
            "{} Please visit: {}",
            style("→").bold().cyan(),
            style(verification_uri).bold().green()
        ))?;
        self.term.write_line(&format!(
            "{} And enter code: {}",
            style("→").bold().cyan(),
            style(user_code).bold().yellow()
        ))?;
        self.term.write_line("")?;
        self.term
            .write_line(&style("Waiting for authorization...").dim().to_string())?;
        Ok(())
    }

    pub fn create_progress_bar(&self, len: u64, message: &str) -> Result<ProgressBar> {
        let pb = ProgressBar::new(len);
        let style = ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
            .context("invalid analysis progress template")?
            .progress_chars("=>-");
        pb.set_style(style);
        pb.set_message(message.to_string());
        Ok(pb)
    }

    pub fn create_spinner(&self, message: &str) -> Result<ProgressBar> {
        let pb = ProgressBar::new_spinner();
        let style = ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .context("invalid spinner template")?;
        pb.set_style(style);
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        Ok(pb)
    }

    pub fn select_repos_to_delete(&self, fork_infos: &[ForkInfo]) -> Result<Vec<usize>> {
        if fork_infos.is_empty() {
            self.term
                .write_line(&style("✓ No fork repositories found!").green().to_string())?;
            return Ok(vec![]);
        }

        self.term.write_line("")?;
        self.term.write_line(&format!(
            "{} Found {} fork repositories",
            style("ℹ").bold().cyan(),
            fork_infos.len()
        ))?;

        let useless_count = fork_infos.iter().filter(|f| f.is_useless).count();
        self.term.write_line(&format!(
            "{} {} are useless, selected by default",
            style("→").cyan(),
            style(useless_count).yellow()
        ))?;
        self.term.write_line("")?;

        let items: Vec<String> = fork_infos
            .iter()
            .map(|info| {
                let repo_name = info.full_name();
                if info.is_useless {
                    format!("{} - {}", repo_name, style("useless").red())
                } else {
                    repo_name.to_string()
                }
            })
            .collect();

        let defaults: Vec<bool> = fork_infos.iter().map(|f| f.is_useless).collect();

        let selections = MultiSelect::with_theme(&self.theme)
            .with_prompt("Select repositories to delete (Space to toggle, Enter to confirm)")
            .items(&items)
            .defaults(&defaults)
            .interact()?;

        Ok(selections)
    }

    pub async fn show_cooldown(&self, seconds: u64, is_batch: bool) -> Result<()> {
        let action = if is_batch {
            "batch deletion"
        } else {
            "deletion"
        };

        self.term.write_line("")?;
        self.term.write_line(&format!(
            "{} {} cooldown period...",
            style("⏳").bold().yellow(),
            style(action).bold()
        ))?;

        let pb = ProgressBar::new(seconds);
        let style = ProgressStyle::default_bar()
            .template("{msg} [{bar:40.yellow/dim}] {pos}s/{len}s")
            .context("invalid cooldown progress template")?
            .progress_chars("█▓░");
        pb.set_style(style);
        pb.set_message("Cooling down".to_string());

        for _ in 0..seconds {
            tokio::time::sleep(Duration::from_secs(1)).await;
            pb.inc(1);
        }

        pb.finish_with_message("Ready!");
        Ok(())
    }

    pub fn confirm_deletion(&self, count: usize, is_batch: bool) -> Result<bool> {
        self.term.write_line("")?;

        let message = if is_batch {
            format!("Are you sure you want to delete {} repositories?", count)
        } else {
            "Are you sure you want to delete this repository?".to_string()
        };

        let confirmed = Confirm::with_theme(&self.theme)
            .with_prompt(message)
            .default(false)
            .interact()?;

        Ok(confirmed)
    }

    pub fn show_success(&self, message: &str) -> Result<()> {
        self.term
            .write_line(&format!("{} {}", style("✓").green(), message))?;
        Ok(())
    }

    pub fn show_error(&self, message: &str) -> Result<()> {
        self.term
            .write_line(&format!("{} {}", style("✗").red(), message))?;
        Ok(())
    }

    pub fn show_info(&self, message: &str) -> Result<()> {
        self.term
            .write_line(&format!("{} {}", style("ℹ").cyan(), message))?;
        Ok(())
    }
}
