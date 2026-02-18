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
    class="icon-trash"
    :class="{ loading }"
    type="button"
    :disabled="disabled"
    :title="loading ? (loadingTitle ?? title) : title"
    :aria-label="loading ? (loadingTitle ?? title) : title"
    @click.prevent="handleClick"
  >
    <svg class="glyph" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M9 3h6l1 2h4v2H4V5h4l1-2zm1 6h2v8h-2V9zm4 0h2v8h-2V9zM7 9h2v8H7V9zm-1 12h12l1-14H5l1 14z"
      />
    </svg>
  </button>
</template>

<style scoped>
.icon-trash {
  width: 34px;
  height: 34px;
  min-width: 34px;
  padding: 0;
  border-radius: 999px;
  border: 1px solid rgba(180, 35, 24, 0.4);
  background: linear-gradient(135deg, #b42318 0%, #d7483a 100%);
  color: #fff8f8;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-shadow:
    0 6px 14px rgba(123, 37, 44, 0.24),
    inset 0 -1px 0 rgba(0, 0, 0, 0.16);
  transition:
    background-color 0.16s ease,
    border-color 0.16s ease,
    opacity 0.16s ease,
    box-shadow 0.16s ease,
    transform 0.16s ease;
}

.icon-trash:not(:disabled):hover {
  background: linear-gradient(135deg, #9f1f16 0%, #c53e31 100%);
  border-color: rgba(180, 35, 24, 0.5);
  box-shadow:
    0 8px 16px rgba(123, 37, 44, 0.28),
    inset 0 -1px 0 rgba(0, 0, 0, 0.2);
  transform: translateY(-1px);
}

.icon-trash:disabled {
  opacity: 0.55;
  cursor: not-allowed;
  box-shadow: none;
}

.glyph {
  width: 16px;
  height: 16px;
  fill: currentColor;
}

.icon-trash.loading .glyph {
  animation: trash-spin 0.85s linear infinite;
}

@keyframes trash-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
