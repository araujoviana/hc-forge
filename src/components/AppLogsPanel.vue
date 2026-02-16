<script setup lang="ts">
type LogLevel = "trace" | "debug" | "info" | "warn" | "error";
type LogEntry = {
  id: number;
  at: string;
  source: string;
  level: LogLevel;
  message: string;
};

defineProps<{
  open: boolean;
  entries: LogEntry[];
  formatDateTime: (value: string | null | undefined) => string;
  hasUnreadError: boolean;
}>();

const emit = defineEmits<{
  (event: "toggle"): void;
  (event: "clear"): void;
}>();

function toggle() {
  emit("toggle");
}

function clear() {
  emit("clear");
}
</script>

<template>
  <div class="logs-floating">
    <button class="log-fab" :class="{ 'log-fab-alert': hasUnreadError }" type="button" @click="toggle">
      <span>{{ open ? "Hide Logs" : "Show Logs" }}</span>
      <span class="log-count">{{ entries.length }}</span>
      <span v-if="hasUnreadError" class="log-alert-dot" aria-hidden="true"></span>
    </button>

    <transition name="drawer">
      <aside v-if="open" class="log-drawer">
        <div class="log-drawer-head">
          <strong>Application Logs</strong>
          <div class="log-drawer-actions">
            <button class="ghost minor" type="button" @click="clear">Clear</button>
          </div>
        </div>
        <div class="log-list">
          <div
            v-for="entry in entries"
            :key="entry.id"
            class="log-row"
            :class="`lvl-${entry.level}`"
          >
            <div class="log-row-head">
              <span class="mono tiny">{{ formatDateTime(entry.at) }}</span>
              <span class="tiny">{{ entry.source }}</span>
              <span class="tiny">{{ entry.level.toUpperCase() }}</span>
            </div>
            <div>{{ entry.message }}</div>
          </div>
          <p v-if="!entries.length" class="muted tiny">No logs yet.</p>
        </div>
      </aside>
    </transition>
  </div>
</template>

<style scoped>
.logs-floating {
  position: fixed;
  inset: 0;
  z-index: 110;
  pointer-events: none;
}

.log-fab,
.log-drawer {
  pointer-events: auto;
}

.log-fab {
  position: fixed;
  right: 14px;
  bottom: 12px;
  min-height: 38px;
  padding: 0 14px;
  border-radius: 999px;
  border: 1px solid rgba(166, 31, 44, 0.45);
  background: linear-gradient(135deg, #a61f2c 0%, #cf3f4f 100%);
  color: #fff8f8;
  box-shadow:
    0 10px 24px rgba(123, 37, 44, 0.3),
    inset 0 -1px 0 rgba(0, 0, 0, 0.16);
  font-weight: 700;
  font-size: 0.82rem;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.log-fab-alert {
  animation: log-alert-pulse 1.15s ease-in-out infinite;
  border-color: rgba(254, 202, 202, 0.95);
}

.log-count {
  display: inline-flex;
  min-width: 22px;
  min-height: 22px;
  padding: 0 6px;
  border-radius: 999px;
  align-items: center;
  justify-content: center;
  font-size: 0.72rem;
  background: rgba(255, 255, 255, 0.22);
  border: 1px solid rgba(255, 255, 255, 0.34);
}

.log-alert-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: #fecaca;
  box-shadow: 0 0 0 4px rgba(254, 202, 202, 0.22);
}

.log-drawer {
  position: fixed;
  right: 14px;
  bottom: 58px;
  width: min(520px, calc(100vw - 20px));
  max-height: min(54vh, 430px);
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 9px;
  border-radius: 12px;
  border: 1px solid #f3b0b4;
  background: #fff7f7;
  box-shadow: 0 16px 32px rgba(123, 37, 44, 0.22);
}

.log-drawer-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  align-items: center;
  color: #7f1d1d;
}

.log-drawer-actions {
  display: flex;
  gap: 6px;
}

.log-list {
  display: grid;
  gap: 5px;
  overflow: auto;
  max-height: min(46vh, 360px);
}

.log-row {
  border-radius: 9px;
  border: 1px solid #f3ccd0;
  background: #ffffff;
  padding: 7px 8px;
  font-size: 0.76rem;
}

.log-row-head {
  display: flex;
  gap: 6px;
  margin-bottom: 3px;
  color: #7b4f56;
  align-items: center;
  flex-wrap: wrap;
}

.log-row.lvl-error {
  border-color: rgba(180, 35, 24, 0.35);
  background: #fff7f6;
}

.log-row.lvl-warn {
  border-color: rgba(217, 119, 6, 0.35);
  background: #fffaf2;
}

.ghost {
  background: #fef2f2;
  border: 1px solid #efb5ba;
  color: #7f1d1d;
}

.minor {
  min-height: 32px;
  padding: 0 10px;
  border-radius: 9px;
  font-size: 0.8rem;
  font-weight: 700;
}

.mono {
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
}

.muted {
  color: #7b4f56;
}

.tiny {
  font-size: 0.72rem;
}

.drawer-enter-active,
.drawer-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.drawer-enter-from,
.drawer-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

@keyframes log-alert-pulse {
  0%,
  100% {
    box-shadow:
      0 10px 24px rgba(123, 37, 44, 0.3),
      0 0 0 0 rgba(127, 29, 29, 0.25),
      inset 0 -1px 0 rgba(0, 0, 0, 0.16);
  }
  50% {
    box-shadow:
      0 12px 26px rgba(123, 37, 44, 0.34),
      0 0 0 8px rgba(127, 29, 29, 0.08),
      inset 0 -1px 0 rgba(0, 0, 0, 0.16);
  }
}

@media (max-width: 980px) {
  .log-fab {
    right: 10px;
    bottom: 10px;
  }

  .log-drawer {
    right: 10px;
    bottom: 56px;
    width: min(96vw, 520px);
    max-height: 58vh;
  }
}
</style>
