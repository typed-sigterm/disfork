export class GitHubClient {
  private token: string;

  constructor(token: string) {
    this.token = token;
  }

  private async fetchGit<T>(path: string, options: any = {}): Promise<T> {
    return await $fetch<T>(path, {
      baseURL: 'https://api.github.com',
      ...options,
      headers: {
        Accept: 'application/vnd.github.v3+json',
        Authorization: `Bearer ${this.token}`,
        ...options.headers,
      },
    });
  }

  async currentUser(): Promise<string> {
    const user = await this.fetchGit<{ login: string }>('/user');
    return user.login;
  }

  async listRepos(owner: string): Promise<any[]> {
    const profile = await this.fetchGit<{ type: string }>(`/users/${owner}`);
    const repos: any[] = [];
    let page = 1;

    // Using simple loop to fetch paginated repos
    while (true) {
      const perPage = 100;
      let url = '';
      if (profile.type.toLowerCase() === 'organization') {
        url = `/orgs/${owner}/repos?per_page=${perPage}&page=${page}`;
      } else {
        url = `/users/${owner}/repos?per_page=${perPage}&page=${page}`;
      }

      const pageData = await this.fetchGit<any[]>(url);
      repos.push(...pageData);
      if (pageData.length < perPage)
        break;
      page++;
    }
    return repos;
  }

  async getRepo(owner: string, repo: string): Promise<any> {
    return await this.fetchGit<any>(`/repos/${owner}/${repo}`);
  }

  async listBranches(owner: string, repo: string): Promise<any[]> {
    const branches: any[] = [];
    let page = 1;
    while (true) {
      const perPage = 100;
      const pageData = await this.fetchGit<any[]>(`/repos/${owner}/${repo}/branches?per_page=${perPage}&page=${page}`);
      branches.push(...pageData);
      if (pageData.length < perPage)
        break;
      page++;
    }
    return branches;
  }

  async compareCommits(owner: string, repo: string, base: string, head: string): Promise<number> {
    const response = await this.fetchGit<{ ahead_by: number }>(`/repos/${owner}/${repo}/compare/${base}...${head}`);
    return response.ahead_by;
  }

  async deleteRepo(owner: string, repo: string): Promise<void> {
    await this.fetchGit(`/repos/${owner}/${repo}`, { method: 'DELETE' });
  }
}
