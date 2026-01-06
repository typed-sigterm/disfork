use crate::github::GitHubClient;
use anyhow::{Result, anyhow};
use octocrab::models::Repository;

#[derive(Debug, Clone)]
pub struct ForkInfo {
    pub repo: Repository,
    pub is_useless: bool,
}

impl ForkInfo {
    pub fn full_name(&self) -> &str {
        self.repo
            .full_name
            .as_deref()
            .unwrap_or(self.repo.name.as_str())
    }

    pub fn owner_login(&self) -> Option<&str> {
        self.repo.owner.as_ref().map(|owner| owner.login.as_str())
    }
}

#[derive(Clone)]
pub struct ForkAnalyzer {
    client: GitHubClient,
}

impl ForkAnalyzer {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn analyze_fork(&self, repo: Repository) -> Result<ForkInfo> {
        let owner = repo
            .owner
            .as_ref()
            .map(|o| o.login.as_str())
            .ok_or_else(|| anyhow!("Fork repository missing owner information"))?;
        let repo_name = &repo.name;
        let repo = self.client.get_repo(owner, repo_name).await?;
        let branches = self.client.list_branches(owner, repo_name).await?;

        if branches.is_empty() {
            return Ok(ForkInfo {
                repo,
                is_useless: true,
            });
        }

        let parent = match &repo.parent {
            Some(parent) => parent,
            None => {
                return Ok(ForkInfo {
                    repo,
                    is_useless: true,
                });
            }
        };

        let parent_owner = parent
            .owner
            .as_ref()
            .map(|o| o.login.as_str())
            .ok_or_else(|| anyhow!("Parent repository missing owner information"))?;
        let parent_name = &parent.name;

        // Check if any branch has commits ahead of upstream
        let mut has_commits_ahead = false;

        for branch in branches {
            // Try to compare branches
            match self
                .client
                .compare_commits(
                    parent_owner,
                    parent_name,
                    &branch.name,
                    &format!("{}:{}", owner, branch.name),
                )
                .await
            {
                Ok(ahead_by) => {
                    if ahead_by > 0 {
                        has_commits_ahead = true;
                        break;
                    }
                }
                Err(_) => {
                    // Branch doesn't exist in upstream, consider it as having independent commits
                    has_commits_ahead = true;
                    break;
                }
            }
        }

        if has_commits_ahead {
            Ok(ForkInfo {
                repo,
                is_useless: false,
            })
        } else {
            Ok(ForkInfo {
                repo,
                is_useless: true,
            })
        }
    }
}
