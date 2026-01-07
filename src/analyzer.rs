use crate::github::GitHubClient;
use anyhow::{Result, anyhow};
use octocrab::models::Repository;
use std::sync::Arc;
use tokio::sync::Semaphore;

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
    semaphore: Arc<Semaphore>,
    max_branches: usize,
}

impl ForkAnalyzer {
    pub fn new(client: GitHubClient, semaphore: Arc<Semaphore>, max_branches: usize) -> Self {
        Self { 
            client,
            semaphore,
            max_branches,
        }
    }

    pub async fn analyze_fork(&self, repo: Repository) -> Result<ForkInfo> {
        let owner = repo
            .owner
            .as_ref()
            .map(|o| o.login.as_str())
            .ok_or_else(|| anyhow!("Fork repository missing owner information"))?;
        let repo_name = &repo.name;
        
        let repo = self.client.get_repo(owner, repo_name, &self.semaphore).await?;
        let branches = self.client.list_branches(owner, repo_name, &self.semaphore).await?;

        if branches.is_empty() {
            return Ok(ForkInfo {
                repo,
                is_useless: true,
            });
        }

        // Skip analyzing repos with too many branches
        if branches.len() > self.max_branches {
            return Ok(ForkInfo {
                repo,
                is_useless: false,
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

        // Check if any branch has commits ahead of upstream - compare in parallel
        let mut tasks = tokio::task::JoinSet::new();

        for branch in branches {
            let client = self.client.clone();
            let semaphore = self.semaphore.clone();
            let parent_owner = parent_owner.to_string();
            let parent_name = parent_name.to_string();
            let owner = owner.to_string();
            let branch_name = branch.name.clone();

            tasks.spawn(async move {
                // Try to compare branches
                client
                    .compare_commits(
                        &parent_owner,
                        &parent_name,
                        &branch_name,
                        &format!("{}:{}", owner, branch_name),
                        &semaphore,
                    )
                    .await
            });
        }

        let mut has_commits_ahead = false;

        while let Some(result) = tasks.join_next().await {
            match result? {
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

        // Abort any remaining tasks to avoid unnecessary API calls
        tasks.abort_all();

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
