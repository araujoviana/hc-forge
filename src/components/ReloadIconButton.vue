<script setup lang="ts">
const props = defineProps<{
  title: string;
  disabled?: boolean;
  loading?: boolean;
  loadingTitle?: string;
}>();

const emit = defineEmits<{
  (event: "click"): void;
}>();

function handleClick() {
  if (props.disabled) {
    return;
  }
  emit("click");
}
</script>

<template>
  <button
    class="icon-reload"
    type="button"
    :disabled="disabled"
    :title="loading ? (loadingTitle ?? title) : title"
    @click.prevent="handleClick"
  >
    ↻
  </button>
</template>

<style scoped>
.icon-reload {
  width: 28px;
  height: 28px;
  min-width: 28px;
  padding: 0;
  border-radius: 999px;
  border: 1px solid #d8b5b5;
  background: #ffffff;
  color: #7f1d1d;
  font-size: 0.9rem;
  line-height: 1;
  font-weight: 700;
  transition: background-color 0.16s ease, border-color 0.16s ease, opacity 0.16s ease;
}

.icon-reload:not(:disabled):hover {
  background: #fef2f2;
  border-color: #e39fa4;
}

.icon-reload:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
