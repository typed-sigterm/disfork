<script setup lang="ts">
const token = useCookie<string>('disfork-token', {
  default: () => '',
  maxAge: 60 * 60 * 24 * 30,
});

const oauthStatus = useState<OAuthStatus>('oauth-status', () =>
  token.value ? 'done' : 'idle');
const oauthUserCode = useState('oauth-user-code', () => '');
const oauthVerificationUri = useState('oauth-verification-uri', () => '');
const oauthError = useState('oauth-error', () => '');
const authenticatedUser = useState('authenticated-user', () => '');

// Non-reactive; only needed during the polling window
let deviceCode = '';
let interval = 5;

const targetAccount = useState('targetAccount', () => '');
const isAnalyzing = useState('isAnalyzing', () => false);
const isDeleting = useState('isDeleting', () => false);
const progress = useState<AnalysisProgress>('progress', () => ({
  current: 0,
  total: 0,
  repoName: '',
}));
const forks = useState<ForkInfo[]>('forks', () => []);
const selectedForks = useState<Record<string, boolean>>('selectedForks', () => ({}));

onMounted(async () => {
  if (token.value && oauthStatus.value === 'done') {
    try {
      authenticatedUser.value = await new GitHubClient(token.value).currentUser();
    } catch {
      token.value = '';
      oauthStatus.value = 'idle';
      authenticatedUser.value = '';
    }
  }
});

async function startFlow() {
  oauthStatus.value = 'starting';
  oauthError.value = '';
  try {
    const result = await fetchDeviceCode();
    deviceCode = result.deviceCode;
    interval = result.interval;
    oauthUserCode.value = result.userCode;
    oauthVerificationUri.value = result.verificationUri;
    oauthStatus.value = 'awaiting';
  } catch (err: any) {
    oauthStatus.value = 'error';
    oauthError.value = err.message ?? 'Unknown error';
  }
}

async function continuePolling() {
  if (oauthStatus.value !== 'awaiting')
    return;
  oauthStatus.value = 'polling';
  try {
    token.value = await pollForToken(deviceCode, interval);
    authenticatedUser.value = await new GitHubClient(token.value).currentUser();
    oauthStatus.value = 'done';
  } catch (err: any) {
    oauthStatus.value = 'error';
    oauthError.value = err.message ?? 'Unknown error';
  }
}

function resetOAuth() {
  oauthStatus.value = 'idle';
  token.value = '';
  oauthUserCode.value = '';
  oauthVerificationUri.value = '';
  oauthError.value = '';
  authenticatedUser.value = '';
  deviceCode = '';
  forks.value = [];
  selectedForks.value = {};
}

async function startAnalysis() {
  isAnalyzing.value = true;
  forks.value = [];
  selectedForks.value = {};
  progress.value = { current: 0, total: 0, repoName: '' };
  try {
    const result = await analyzeForks(
      token.value,
      targetAccount.value,
      (p) => {
        progress.value = p;
      },
    );
    forks.value = result.forks;
    selectedForks.value = Object.fromEntries(
      result.forks.map(f => [f.repo.full_name, result.uselessNames.has(f.repo.full_name)]),
    );
  } catch (err) {
    console.error('Analysis failed:', err);
  } finally {
    isAnalyzing.value = false;
  }
}

async function executeDeletion() {
  const toDelete = Object.keys(selectedForks.value).filter(key => selectedForks.value[key]);

  isDeleting.value = true;
  try {
    await deleteForks(token.value, toDelete, (_current, _total, deleted) => {
      forks.value = forks.value.filter(f => f.repo.full_name !== deleted);
    });
    selectedForks.value = {};
  } catch (err) {
    console.error('Deletion failed:', err);
  } finally {
    isDeleting.value = false;
  }
}
</script>

<template>
  <UApp>
    <UMain class="p-8">
      <div class="max-w-3xl mx-auto">
        <AnalyzerForm
          v-model:token="token"
          v-model:target-account="targetAccount"
          v-model:selected-forks="selectedForks"
          :is-analyzing
          :is-deleting
          :progress
          :oauth-status
          :oauth-user-code
          :oauth-verification-uri
          :oauth-error
          :authenticated-user
          :forks
          @oauth-start="startFlow"
          @oauth-continue="continuePolling"
          @oauth-reset="resetOAuth"
          @analyze="startAnalysis"
          @delete="executeDeletion"
        />

        <div class="text-muted mt-2 flex justify-between">
          <p class="text-sm">
            &copy; 2026-present
            <ULink active href="https://typed-sigterm.me/?utm_source=disfork.by-ts.top&utm_medium=footer" target="_blank">
              Typed SIGTERM
            </ULink>
          </p>
          <ULink class="text-lg" href="http://github.com/typed-sigterm/disfork" target="_blank">
            <UIcon name="i-logos-github-icon" />
          </ULink>
        </div>
      </div>
    </UMain>
  </UApp>
</template>
