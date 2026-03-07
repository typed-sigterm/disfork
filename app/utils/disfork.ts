export interface AnalysisProgress {
  current: number
  total: number
  /** Name of the repository currently being analyzed */
  repoName: string
}

export interface AnalysisResult {
  forks: ForkInfo[]
  /** full_names of forks with no upstream contribution */
  uselessNames: Set<string>
}

/**
 * Fetch and analyze all forks for the given account.
 * Reports progress via the onProgress callback (called per repo).
 * Returns the full result; throws on fatal error.
 */
export async function analyzeForks(
  token: string,
  targetAccount: string,
  onProgress: (p: AnalysisProgress) => void,
): Promise<AnalysisResult> {
  const client = new GitHubClient(token);
  const account = targetAccount || await client.currentUser();
  const repos = await client.listRepos(account);
  const forkRepos = repos.filter((r: any) => r.fork);

  if (forkRepos.length === 0)
    return { forks: [], uselessNames: new Set() };

  const analyzer = new ForkAnalyzer(client);
  const total = forkRepos.length;
  const results: ForkInfo[] = [];
  const uselessNames = new Set<string>();

  let index = 0;
  let current = 0;

  const processNext = async (): Promise<void> => {
    if (index >= total)
      return;
    const repo = forkRepos[index++];
    onProgress({ current, total, repoName: repo.name });
    try {
      const info = await analyzer.analyzeFork(repo);
      results.push(info);
      if (info.isUseless)
        uselessNames.add(info.repo.full_name);
    } catch (err) {
      console.error('Failed to analyze repository', err);
    } finally {
      current++;
      await processNext();
    }
  };

  await Promise.all(Array.from({ length: Math.min(6, total) }, processNext));

  return { forks: results, uselessNames };
}

/**
 * Delete the listed repositories one by one.
 * Reports progress via onProgress (called after each deletion).
 * The caller decides which repos remain in the list.
 */
export async function deleteForks(
  token: string,
  fullNames: string[],
  onProgress: (current: number, total: number, deleted: string) => void,
): Promise<void> {
  const client = new GitHubClient(token);
  for (let i = 0; i < fullNames.length; i++) {
    const [owner, repo] = fullNames[i]!.split('/');
    await client.deleteRepo(owner!, repo!);
    onProgress(i + 1, fullNames.length, fullNames[i]!);
  }
}
