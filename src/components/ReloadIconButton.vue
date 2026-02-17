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
    :class="{ loading }"
    type="button"
    :disabled="disabled"
    :title="loading ? (loadingTitle ?? title) : title"
    @click.prevent="handleClick"
  >
    <span class="glyph">↻</span>
  </button>
</template>

<style scoped>
.icon-reload {
  width: 34px;
  height: 34px;
  min-width: 34px;
  padding: 0;
  border-radius: 999px;
  border: 1px solid rgba(166, 31, 44, 0.42);
  background: linear-gradient(135deg, #a61f2c 0%, #cf3f4f 100%);
  color: #fff8f8;
  font-size: 1rem;
  line-height: 1;
  font-weight: 700;
  box-shadow:
    0 6px 14px rgba(123, 37, 44, 0.28),
    inset 0 -1px 0 rgba(0, 0, 0, 0.16);
  transition:
    background-color 0.16s ease,
    border-color 0.16s ease,
    opacity 0.16s ease,
    box-shadow 0.16s ease,
    transform 0.16s ease;
}

.icon-reload:not(:disabled):hover {
  background: linear-gradient(135deg, #951c28 0%, #b92738 100%);
  border-color: rgba(166, 31, 44, 0.52);
  box-shadow:
    0 8px 16px rgba(123, 37, 44, 0.32),
    inset 0 -1px 0 rgba(0, 0, 0, 0.2);
  transform: translateY(-1px);
}

.icon-reload:disabled {
  opacity: 0.55;
  cursor: not-allowed;
  box-shadow: none;
}

.glyph {
  display: inline-block;
  transform-origin: 50% 50%;
}

.icon-reload.loading .glyph {
  animation: reload-spin 0.85s linear infinite;
}

@keyframes reload-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
