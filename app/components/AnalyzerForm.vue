<script setup lang="ts">
const props = defineProps<{
  isAnalyzing: boolean
  isDeleting: boolean
  progress: AnalysisProgress
  oauthStatus: OAuthStatus
  oauthUserCode: string
  oauthVerificationUri: string
  oauthError: string
  authenticatedUser: string
  forks: ForkInfo[]
}>();

defineEmits<{
  (e: 'oauthStart'): void
  (e: 'oauthContinue'): void
  (e: 'oauthReset'): void
  (e: 'analyze'): void
  (e: 'delete'): void
}>();

const token = defineModel<string>('token', { required: true });
const targetAccount = defineModel<string>('targetAccount', { required: true });
const selectedForks = defineModel<Record<string, boolean>>('selectedForks', { required: true });

const patOpen = ref(!!token.value);
const copyCode = () => navigator.clipboard.writeText(props.oauthUserCode);

const step = computed(() => {
  if (props.isAnalyzing)
    return 'analyzing';
  if (props.forks.length > 0)
    return 'results';
  if (props.oauthStatus !== 'done')
    return props.oauthStatus;
  return 'ready';
});

const uselessCount = computed(() => props.forks.filter(f => f.isUseless).length);
</script>

<template>
  <UCard>
    <template #header>
      <template v-if="step === 'results'">
        <h1 class="text-2xl font-bold mb-1">
          Analysis Results
        </h1>
        <p class="text-sm text-gray-500">
          {{ forks.length }} forks found &middot; {{ uselessCount }} useless
        </p>
      </template>
      <template v-else-if="step === 'analyzing'">
        <h1 class="text-2xl font-bold mb-1">
          Analyzing Repositories
        </h1>
      </template>
      <template v-else>
        <h1 class="text-2xl font-bold mb-1">
          DisFork
        </h1>
        <p class="text-sm text-gray-500">
          Clean up useless GitHub forks effortlessly
        </p>
      </template>
    </template>

    <div v-if="step === 'idle'" class="space-y-4">
      <UButton block color="primary" icon="i-simple-icons-github" @click="$emit('oauthStart')">
        Authorize with GitHub
      </UButton>

      <UCollapsible v-model:open="patOpen">
        <UButton variant="ghost" color="neutral" icon="i-lucide-settings-2" trailing-icon="i-lucide-chevron-down" size="sm">
          Advanced Configuration
        </UButton>
        <template #content>
          <div class="space-y-4 pt-3">
            <UFormField label="Personal Access Token">
              <UInput v-model="token" type="password" placeholder="ghp_xxxxxxxxxxxx" class="w-full" />
              <template #description>
                Needs
                <code class="px-1.5 py-0.5 text-sm font-mono rounded-md inline-block border border-muted bg-muted">
                  delete_repo
                </code>
                scope
              </template>
            </UFormField>
            <UFormField label="Target Account" description="Leave blank to use your own account">
              <UInput v-model="targetAccount" placeholder="username of a user or organization" class="w-full" />
            </UFormField>
            <UButton block color="neutral" variant="outline" :disabled="!token" @click="$emit('analyze')">
              Start Analysis using PAT
            </UButton>
          </div>
        </template>
      </UCollapsible>
    </div>

    <div v-else-if="step === 'error'" class="space-y-4">
      <UAlert color="error" variant="soft" icon="i-lucide-circle-x" :title="oauthError" />
      <UButton block color="primary" icon="i-simple-icons-github" @click="$emit('oauthStart')">
        Try Again
      </UButton>
    </div>

    <Spinner v-else-if="step === 'starting'">
      Connecting to GitHub...
    </Spinner>

    <div v-else-if="step === 'awaiting'" class="space-y-4">
      <UAlert
        color="info"
        variant="soft"
        icon="i-lucide-key-round"
        title="Authorize on GitHub"
        description="Enter this code on GitHub, then click Continue."
      />
      <div class="flex items-center gap-2">
        <span class="flex-1 font-mono text-2xl font-bold tracking-[0.35em] text-center py-3 bg-gray-100 dark:bg-gray-800 rounded-lg">
          {{ oauthUserCode }}
        </span>
        <UButton icon="i-lucide-copy" color="neutral" variant="outline" square @click="copyCode" />
      </div>
      <UButton block color="primary" icon="i-lucide-external-link" :to="oauthVerificationUri" target="_blank">
        Open GitHub to Authorize
      </UButton>
      <UButton block color="neutral" variant="outline" icon="i-lucide-check" @click="$emit('oauthContinue')">
        I've authorized — Continue
      </UButton>
    </div>

    <Spinner v-else-if="step === 'polling'">
      Verifying authorization...
    </Spinner>

    <div v-else-if="step === 'ready'" class="space-y-4">
      <UAlert
        color="success"
        variant="soft"
        icon="i-lucide-circle-check"
        :title="authenticatedUser ? `Authorized as @${authenticatedUser}` : 'Authorized'"
      />

      <div class="flex justify-between items-center">
        <UCollapsible>
          <UButton variant="ghost" color="neutral" icon="i-lucide-settings-2" trailing-icon="i-lucide-chevron-down" size="sm">
            Advanced Configuration
          </UButton>
          <template #content>
            <div class="pt-3">
              <UFormField label="Target Account" description="Leave blank to use the authorized account">
                <UInput v-model="targetAccount" placeholder="username of a user or an organization" class="w-full" />
              </UFormField>
            </div>
          </template>
        </UCollapsible>

        <UButton color="neutral" variant="ghost" size="sm" icon="i-lucide-log-out" @click="$emit('oauthReset')">
          Sign out
        </UButton>
      </div>

      <UButton block color="primary" @click="$emit('analyze')">
        Start Analysis
      </UButton>
    </div>

    <div v-else-if="step === 'analyzing'" class="space-y-3">
      <UProgress
        :model-value="progress.total ? progress.current : null"
        :max="progress.total"
      />
      <div class="flex justify-between text-sm text-gray-500">
        <span v-if="progress.total" class="tabular-nums">
          Analyzing: {{ progress.current }} / {{ progress.total }}
        </span>
        <span v-else>
          Gathering repositories...
        </span>
        <span class="truncate ml-4 text-right">{{ progress.repoName }}</span>
      </div>
    </div>

    <div v-else-if="step === 'results'" class="space-y-4">
      <ForkList
        v-model:selected-forks="selectedForks"
        :forks
        :is-deleting
        @delete="$emit('delete')"
      />
      <UButton
        block
        color="neutral"
        variant="outline"
        icon="i-lucide-refresh-cw"
        :disabled="isDeleting"
        @click="$emit('analyze')"
      >
        Re-analyze
      </UButton>
    </div>
  </UCard>
</template>
