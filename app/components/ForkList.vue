<script setup lang="ts">
const props = defineProps<{
  forks: ForkInfo[]
  isDeleting: boolean
}>();

const emit = defineEmits<{
  (e: 'delete'): void
}>();

const selectedForks = defineModel<Record<string, boolean>>('selectedForks', { required: true });

const isConfirmOpen = ref(false);

const selectedCount = computed(() =>
  props.forks.filter(f => selectedForks.value[f.repo.full_name]).length,
);

function requestDelete() {
  if (selectedCount.value > 0)
    isConfirmOpen.value = true;
}

function onConfirm() {
  isConfirmOpen.value = false;
  emit('delete');
}
</script>

<template>
  <div class="space-y-4">
    <div class="grid grid-cols-3 gap-3 w-full">
      <UTooltip
        v-for="fork in forks"
        :key="fork.repo.full_name"
      >
        <div class="flex items-center gap-3 p-3 border rounded border-gray-200 dark:border-gray-800">
          <UCheckbox v-model="selectedForks[fork.repo.full_name]" class="cursor-pointer" />
          <div class="flex flex-1 min-w-0">
            <UBadge v-if="fork.isUseless" icon="i-lucide-archive-x" color="error" variant="soft" size="sm" />
            <ULink :href="fork.repo.html_url" target="_blank" class="block text-sm ml-1 truncate hover:underline">
              {{ fork.repo.name }}
            </ULink>
          </div>
        </div>
        <template v-if="fork.isUseless" #content>
          Useless
        </template>
      </UTooltip>
    </div>

    <div class="flex justify-between items-center">
      <p class="text-sm text-muted">
        When you finishing cleaning up, just close this page.
      </p>
      <UButton
        color="error"
        icon="i-lucide-trash-2"
        :disabled="isDeleting || selectedCount === 0"
        :loading="isDeleting"
        @click="requestDelete"
      >
        Delete Selected
        <template v-if="selectedCount > 0">
          ({{ selectedCount }})
        </template>
      </UButton>
    </div>
  </div>

  <UModal v-model:open="isConfirmOpen" :ui="{ header: 'font-bold', footer: 'flex gap-2 justify-end' }">
    <template #header>
      Confirm Deletion
    </template>
    <template #body>
      <p>
        Are you sure you want to permanently delete
        <strong>{{ selectedCount }} {{ selectedCount === 1 ? 'repository' : 'repositories' }}</strong>?
      </p>
      <p>
        This action cannot be undone.
      </p>
    </template>

    <template #footer>
      <UButton color="error" icon="i-lucide-trash-2" @click="onConfirm">
        Delete
      </UButton>
      <UButton variant="outline" color="neutral" @click="isConfirmOpen = false">
        Cancel
      </UButton>
    </template>
  </UModal>
</template>
