<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";

type SshTerminalEntry = {
  id: number;
  kind: "meta" | "command" | "stdout" | "stderr";
  text: string;
};

const props = defineProps<{
  open: boolean;
  serverLabel: string;
  host: string | null;
  connected: boolean;
  busy: boolean;
  running: boolean;
  useFormPassword: boolean;
  manualPassword: string;
  terminalEntries: SshTerminalEntry[];
  commandInput: string;
  canReconnect: boolean;
  canDisconnect: boolean;
  canRun: boolean;
}>();

const emit = defineEmits<{
  (event: "close"): void;
  (event: "clear"): void;
  (event: "reconnect"): void;
  (event: "disconnect"): void;
  (event: "run"): void;
  (event: "send-control", value: "ctrl+c" | "ctrl+d" | "ctrl+u"): void;
  (event: "terminal-resize", value: { cols: number; rows: number }): void;
  (event: "command-keydown", value: KeyboardEvent): void;
  (event: "update:useFormPassword", value: boolean): void;
  (event: "update:manualPassword", value: string): void;
  (event: "update:commandInput", value: string): void;
}>();

const authOpen = ref(false);
const connectionOpen = ref(false);
const terminalRef = ref<HTMLElement | null>(null);
let resizeObserver: ResizeObserver | null = null;
let lastCols = 0;
let lastRows = 0;

const connectLabel = computed(() => {
  if (props.busy) {
    return props.connected ? "Reconnecting..." : "Connecting...";
  }
  return props.connected ? "Reconnect" : "Connect";
});

watch(
  () => props.open,
  async (open) => {
    teardownResizeObserver();
    if (!open) {
      return;
    }
    authOpen.value = false;
    connectionOpen.value = false;
    await nextTick();
    setupResizeObserver();
    emitTerminalResize();
  },
  { immediate: true }
);

watch(
  () => props.terminalEntries.length,
  async () => {
    await nextTick();
    const terminal = terminalRef.value;
    if (!terminal) {
      return;
    }
    terminal.scrollTop = terminal.scrollHeight;
  }
);

onBeforeUnmount(() => {
  teardownResizeObserver();
});

function updateUseFormPassword(event: Event) {
  emit("update:useFormPassword", (event.target as HTMLInputElement).checked);
}

function updateManualPassword(event: Event) {
  emit("update:manualPassword", (event.target as HTMLInputElement).value);
}

function updateCommandInput(event: Event) {
  emit("update:commandInput", (event.target as HTMLInputElement).value);
}

function onCommandKeydown(event: KeyboardEvent) {
  emit("command-keydown", event);
}

function emitControlShortcut(control: "ctrl+c" | "ctrl+d" | "ctrl+u") {
  emit("send-control", control);
}

function setupResizeObserver() {
  const terminal = terminalRef.value;
  if (!terminal) {
    return;
  }

  resizeObserver = new ResizeObserver(() => {
    emitTerminalResize();
  });
  resizeObserver.observe(terminal);
}

function teardownResizeObserver() {
  if (!resizeObserver) {
    return;
  }
  resizeObserver.disconnect();
  resizeObserver = null;
}

function emitTerminalResize() {
  const terminal = terminalRef.value;
  if (!terminal) {
    return;
  }

  const cols = Math.max(40, Math.floor((terminal.clientWidth - 20) / 8.6));
  const rows = Math.max(12, Math.floor((terminal.clientHeight - 16) / 18));
  if (cols === lastCols && rows === lastRows) {
    return;
  }
  lastCols = cols;
  lastRows = rows;
  emit("terminal-resize", { cols, rows });
}
</script>

<template>
  <section v-if="open" class="output-card ssh-card">
    <div class="ssh-card-head">
      <div class="ssh-heading">
        <div class="card-title">SSH Terminal</div>
        <div class="card-subtitle">{{ serverLabel }}</div>
      </div>
      <div class="ssh-head-actions">
        <button class="ghost minor" type="button" @click="$emit('clear')">Clear</button>
        <button class="ghost minor close-button" type="button" aria-label="Close SSH panel" @click="$emit('close')">
          ✕
        </button>
      </div>
    </div>

    <div class="ssh-panel-meta">
      <span class="muted tiny">Host: <span class="mono">{{ host ?? "No public EIP" }}</span></span>
      <span class="muted tiny">User: <span class="mono">root</span></span>
      <span class="muted tiny">Session: <span class="mono">{{ connected ? "Connected" : "Disconnected" }}</span></span>
    </div>

    <div class="ssh-fold-grid">
      <div class="ssh-fold">
        <button class="ssh-fold-toggle" type="button" @click="authOpen = !authOpen">
          <span>Auth</span>
          <span class="fold-state">{{ authOpen ? "Hide" : "Show" }}</span>
        </button>
        <transition name="fold">
          <div v-show="authOpen" class="ssh-fold-body">
            <label class="toggle-inline">
              <input :checked="useFormPassword" type="checkbox" @change="updateUseFormPassword" />
              <span>Use admin password from form</span>
            </label>
            <input
              :value="manualPassword"
              type="password"
              spellcheck="false"
              placeholder="Enter SSH root password"
              :disabled="useFormPassword"
              @input="updateManualPassword"
            />
          </div>
        </transition>
      </div>

      <div class="ssh-fold">
        <button class="ssh-fold-toggle" type="button" @click="connectionOpen = !connectionOpen">
          <span>Connection</span>
          <span class="fold-state">{{ connectionOpen ? "Hide" : "Show" }}</span>
        </button>
        <transition name="fold">
          <div v-show="connectionOpen" class="ssh-fold-body">
            <div class="ssh-connect-actions">
              <button
                class="ghost minor"
                type="button"
                :disabled="!canReconnect"
                @click="$emit('reconnect')"
              >
                {{ connectLabel }}
              </button>
              <button
                class="ghost minor danger"
                type="button"
                :disabled="!canDisconnect"
                @click="$emit('disconnect')"
              >
                {{ busy && connected ? "Disconnecting..." : "Disconnect" }}
              </button>
            </div>
          </div>
        </transition>
      </div>
    </div>

    <div ref="terminalRef" class="ssh-terminal" role="log" aria-live="polite">
      <div
        v-for="entry in terminalEntries"
        :key="entry.id"
        class="ssh-line"
        :class="`ssh-${entry.kind}`"
      >
        <pre class="ssh-text">{{ entry.text }}</pre>
      </div>
      <p v-if="!terminalEntries.length" class="muted tiny">No SSH output yet.</p>
    </div>

    <div class="ssh-command-row">
      <div class="ssh-command-input-wrap">
        <input
          :value="commandInput"
          class="mono"
          placeholder="Type a command, then press Enter..."
          :disabled="!connected || running"
          @input="updateCommandInput"
          @keydown="onCommandKeydown"
        />
        <div class="ssh-ctrl-actions">
          <button
            class="ghost minor ctrl-chip"
            type="button"
            :disabled="!connected || busy"
            title="Send Ctrl+C"
            @click="emitControlShortcut('ctrl+c')"
          >
            Ctrl+C
          </button>
          <button
            class="ghost minor ctrl-chip"
            type="button"
            :disabled="!connected || busy"
            title="Send Ctrl+D"
            @click="emitControlShortcut('ctrl+d')"
          >
            Ctrl+D
          </button>
          <button
            class="ghost minor ctrl-chip"
            type="button"
            :disabled="!connected || busy"
            title="Send Ctrl+U (clear line)"
            @click="emitControlShortcut('ctrl+u')"
          >
            Clear Line
          </button>
        </div>
      </div>
      <button class="primary" type="button" :disabled="!canRun" @click="$emit('run')">Run</button>
    </div>
  </section>
</template>

<style scoped>
.ssh-card {
  border-radius: 14px;
  border: 1px solid #efc1c5;
  background: linear-gradient(180deg, rgba(255, 250, 250, 0.97), rgba(255, 245, 246, 0.98));
  box-shadow: 0 10px 20px rgba(123, 37, 44, 0.08);
  display: grid;
  gap: 10px;
  padding: 12px 14px;
}

.ssh-card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.ssh-heading {
  display: grid;
  gap: 2px;
}

.card-title {
  font-weight: 700;
  font-size: 1rem;
  color: #7f1d1d;
}

.card-subtitle {
  font-size: 0.8rem;
  color: #8b4c54;
}

.ssh-head-actions {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
}

.ghost {
  background: #fff8f8;
  border: 1px solid #ebb8be;
  color: #7f1d1d;
}

.minor {
  min-height: 34px;
  padding: 0 10px;
  border-radius: 9px;
  font-size: 0.82rem;
  font-weight: 700;
}

.danger {
  border-color: rgba(180, 35, 24, 0.35);
  color: #b42318;
  background: #fff5f5;
}

.close-button {
  width: 34px;
  min-width: 34px;
  padding: 0;
}

.ssh-panel-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 10px;
  background: #fff0f1;
  border: 1px solid #efc1c5;
}

.ssh-fold-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.ssh-fold {
  border: 1px solid #efc1c5;
  border-radius: 10px;
  background: #fffafa;
  overflow: hidden;
}

.ssh-fold-toggle {
  width: 100%;
  border: 0;
  background: #fff3f4;
  color: #7f1d1d;
  min-height: 34px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.74rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.fold-state {
  font-size: 0.68rem;
  color: #8d5a62;
  letter-spacing: 0.12em;
}

.ssh-fold-body {
  display: grid;
  gap: 8px;
  padding: 9px;
}

.toggle-inline {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.82rem;
  color: #6f3f46;
}

.ssh-connect-actions {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
}

.ssh-terminal {
  height: 400px;
  min-height: 340px;
  max-height: 460px;
  overflow: auto;
  border-radius: 10px;
  border: 1px solid #2b1821;
  background: radial-gradient(circle at top right, rgba(127, 29, 29, 0.18), transparent 45%),
    #070a15;
  color: #f8d7da;
  padding: 12px;
  display: grid;
  gap: 5px;
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.04),
    inset 0 10px 24px rgba(2, 6, 23, 0.5);
}

.ssh-line {
  display: block;
}

.ssh-text {
  margin: 0;
  white-space: pre;
  word-break: normal;
  overflow-wrap: normal;
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
  font-size: 0.86rem;
  line-height: 1.3;
}

.ssh-command .ssh-text {
  color: #fecaca;
}

.ssh-stderr .ssh-text {
  color: #fda4af;
}

.ssh-meta .ssh-text {
  color: #f5b3b9;
}

.ssh-command-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 110px;
  gap: 8px;
  align-items: end;
}

.ssh-command-input-wrap {
  display: grid;
  gap: 6px;
}

.ssh-command-row input {
  min-height: 40px;
  border: 1px solid #513547;
  border-radius: 10px;
  background: #120b1b;
  color: #fee2e2;
  padding: 0 12px;
}

.ssh-command-row input::placeholder {
  color: #c08f9b;
}

.ssh-ctrl-actions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.ctrl-chip {
  min-height: 28px;
  padding: 0 8px;
  border-radius: 8px;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.primary {
  min-height: 40px;
  border: 1px solid #bb2d37;
  border-radius: 10px;
  background: linear-gradient(135deg, #b4232f 0%, #cb3645 100%);
  color: #fff;
  font-size: 0.84rem;
  font-weight: 700;
}

.mono {
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
}

.muted {
  color: #9b7282;
}

.tiny {
  font-size: 0.74rem;
}

.fold-enter-active,
.fold-leave-active {
  transition: max-height 0.2s ease, opacity 0.16s ease;
  overflow: hidden;
}

.fold-enter-from,
.fold-leave-to {
  max-height: 0;
  opacity: 0;
}

.fold-enter-to,
.fold-leave-from {
  max-height: 260px;
  opacity: 1;
}

@media (max-width: 1100px) {
  .ssh-fold-grid {
    grid-template-columns: 1fr;
  }

  .ssh-command-row {
    grid-template-columns: 1fr;
  }

  .ssh-terminal {
    height: 320px;
    min-height: 320px;
    max-height: 320px;
  }
}
</style>
