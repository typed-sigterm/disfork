export interface ForkInfo {
  repo: any
  isUseless: boolean
}

export class ForkAnalyzer {
  constructor(private client: GitHubClient) {}

  async analyzeFork(repo: any): Promise<ForkInfo> {
    const owner = repo.owner?.login;
    if (!owner)
      throw new Error('Fork repository missing owner information');
    const repoName = repo.name;

    const detailedRepo = await this.client.getRepo(owner, repoName);
    const branches = await this.client.listBranches(owner, repoName);

    if (branches.length === 0) {
      return { repo: detailedRepo, isUseless: true };
    }

    const parent = detailedRepo.parent;
    if (!parent) {
      return { repo: detailedRepo, isUseless: true };
    }

    const parentOwner = parent.owner?.login;
    const parentName = parent.name;

    if (!parentOwner)
      throw new Error('Parent repository missing owner information');

    let hasCommitsAhead = false;

    for (const branch of branches) {
      try {
        const aheadBy = await this.client.compareCommits(
          parentOwner,
          parentName,
          branch.name,
          `${owner}:${branch.name}`,
        );
        if (aheadBy > 0) {
          hasCommitsAhead = true;
          break;
        }
      } catch {
        // Branch doesn't exist in upstream, consider it as having independent commits
        hasCommitsAhead = true;
        break;
      }
    }

    return {
      repo: detailedRepo,
      isUseless: !hasCommitsAhead,
    };
  }
}
