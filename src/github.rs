use anyhow::Result;
use octocrab::models::{Repository, repos::Branch};
use octocrab::{Octocrab, Page};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub struct GitHubClient {
    pub octocrab: Octocrab,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

impl GitHubClient {
    pub async fn start_device_flow(client_id: &str) -> Result<DeviceCode> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&[("client_id", client_id)])
            .send()
            .await?;

        let device_code: DeviceCode = response.json().await?;
        Ok(device_code)
    }

    pub async fn poll_for_token(
        client_id: &str,
        device_code: &str,
        interval: u64,
        expires_in: u64,
    ) -> Result<String> {
        let client = reqwest::Client::new();
        let start = tokio::time::Instant::now();
        let expires_after = std::time::Duration::from_secs(expires_in);
        let mut poll_interval = interval;

        loop {
            if start.elapsed() >= expires_after {
                anyhow::bail!("Authorization timed out after {} seconds", expires_in);
            }

            tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;

            if start.elapsed() >= expires_after {
                anyhow::bail!("Authorization timed out after {} seconds", expires_in);
            }

            let response = client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&[
                    ("client_id", client_id),
                    ("device_code", device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await?;

            #[derive(Deserialize)]
            struct TokenResponse {
                access_token: Option<String>,
                error: Option<String>,
                error_description: Option<String>,
            }

            let result: TokenResponse = response.json().await?;

            if let Some(token) = result.access_token {
                return Ok(token);
            }

            if let Some(error) = result.error {
                match error.as_str() {
                    "authorization_pending" => continue,
                    "slow_down" => {
                        poll_interval += 5;
                        continue;
                    }
                    "expired_token" => {
                        anyhow::bail!("Device code expired. Please restart authorization.")
                    }
                    "access_denied" => anyhow::bail!("Authorization denied on GitHub device flow."),
                    _ => anyhow::bail!("Authorization failed: {}", error),
                }
            }

            if let Some(description) = result.error_description {
                anyhow::bail!("Authorization failed: {}", description);
            }
        }
    }

    pub fn new(token: String, parallel: usize) -> Result<Self> {
        let octocrab = Octocrab::builder().personal_token(token).build()?;
        let semaphore = Arc::new(Semaphore::new(parallel));

        Ok(Self { octocrab, semaphore })
    }

    pub async fn current_user(&self) -> Result<String> {
        let user = self.octocrab.current().user().await?;
        Ok(user.login)
    }

    pub async fn list_repos(&self, owner: &str) -> Result<Vec<Repository>> {
        let profile = self.octocrab.users(owner).profile().await?;
        let account_type = profile.r#type.to_ascii_lowercase();

        if account_type == "organization" || account_type == "enterprise" {
            self.list_org_repos(owner).await
        } else {
            self.list_user_repos(owner).await
        }
    }

    async fn list_user_repos(&self, owner: &str) -> Result<Vec<Repository>> {
        let mut repos = Vec::new();
        let mut page = 1u32;

        loop {
            let page_data: Page<Repository> = self
                .octocrab
                .users(owner)
                .repos()
                .per_page(100)
                .page(page)
                .send()
                .await?;

            repos.extend(page_data.items);

            if page_data.next.is_none() {
                break;
            }
            page += 1;
        }

        Ok(repos)
    }

    async fn list_org_repos(&self, owner: &str) -> Result<Vec<Repository>> {
        let mut repos = Vec::new();
        let mut page = 1u32;

        loop {
            let page_data: Page<Repository> = self
                .octocrab
                .orgs(owner)
                .list_repos()
                .per_page(100)
                .page(page)
                .send()
                .await?;

            repos.extend(page_data.items);

            if page_data.next.is_none() {
                break;
            }
            page += 1;
        }

        Ok(repos)
    }

    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repository> {
        let _permit = self.semaphore.acquire().await?;
        let repo = self.octocrab.repos(owner, repo).get().await?;
        Ok(repo)
    }

    pub async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>> {
        let mut branches = Vec::new();
        let mut page = 1u32;

        loop {
            // Acquire permit per page to ensure fair distribution of HTTP requests
            let _permit = self.semaphore.acquire().await?;
            let page_data: Page<Branch> = self
                .octocrab
                .repos(owner, repo)
                .list_branches()
                .per_page(100)
                .page(page)
                .send()
                .await?;

            let has_next = page_data.next.is_some();
            branches.extend(page_data.items);

            if !has_next {
                break;
            }
            page += 1;
        }

        Ok(branches)
    }

    pub async fn compare_commits(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<i64> {
        let _permit = self.semaphore.acquire().await?;
        let url = format!("/repos/{}/{}/compare/{}...{}", owner, repo, base, head);

        #[derive(Deserialize)]
        struct CompareResult {
            ahead_by: i64,
        }

        let response: CompareResult = self.octocrab.get(&url, None::<&()>).await?;

        Ok(response.ahead_by)
    }

    pub async fn delete_repo(&self, owner: &str, repo: &str) -> Result<()> {
        self.octocrab.repos(owner, repo).delete().await?;

        Ok(())
    }
}
