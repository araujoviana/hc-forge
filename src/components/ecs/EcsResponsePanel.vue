<script setup lang="ts">
import { computed, ref } from "vue";
import ReloadIconButton from "../ReloadIconButton.vue";
import SshTerminalPanel from "../SshTerminalPanel.vue";
import type {
  CreateEcsResult,
  DockerContainerSummary,
  DockerImageSummary,
  EcsServer,
  EipRecord,
  EvsVolume,
  NixPackageSummary,
  PlatformOpsTab,
  SshTerminalEntry,
} from "../../types/ecs";

type CreateSummary = {
  status: string;
  statusCode: number;
  serverId?: string | null;
  jobId?: string | null;
  message?: string | null;
};

const props = defineProps<{
  quickCopyFeedback: string | null;
  errorMsg: string;
  deleteMsg: string | null;
  createSummary: CreateSummary | null;
  createdServer: EcsServer | null;
  createdEip: EipRecord | null;
  pollingEcs: boolean;
  pollingAttempts: number;
  pollMaxAttempts: number;
  pollingStatus: string | null;
  pollingError: string | null;
  canWatch: boolean;
  result: CreateEcsResult | null;
  eips: EipRecord[];
  evss: EvsVolume[];
  ecses: EcsServer[];
  loadingResponse: boolean;
  loadingEips: boolean;
  loadingEvss: boolean;
  loadingEcses: boolean;
  cacheAgeEips: string;
  cacheAgeEvss: string;
  cacheAgeEcses: string;
  sshPanelOpen: boolean;
  sshPanelServer: EcsServer | null;
  sshPanelHost: string | null;
  sshConnectedToPanel: boolean;
  sshBusyServerId: string | null;
  stoppingServerId: string | null;
  deletingServerId: string | null;
  sshRunningCommand: boolean;
  sshUseFormPassword: boolean;
  sshManualPassword: string;
  sshCommandInput: string;
  sshTerminalEntries: SshTerminalEntry[];
  statusTone: (status: string | null | undefined) => string;
  copyEipAddress: (address: string | null | undefined) => void | Promise<void>;
  evsRole: (volume: EvsVolume) => "Boot" | "Data";
  evsAttachedServer: (volume: EvsVolume) => string;
  findSshHostForServer: (ecs: EcsServer) => string | null;
  autoUpdateStatusForServer: (
    serverId: string
  ) => "queued" | "running" | "done" | "failed" | "idle";
  autoUpdateStatusLabel: (ecs: EcsServer) => string;
  autoUpdateProgressHint: (serverId: string) => string | null;
  startupTaskRdpUserForServer: (serverId: string) => string | null;
  loginUsernameForServer: (serverId: string) => string;
  copyLoginUsernameForServer: (serverId: string) => void | Promise<void>;
  hasSavedPasswordForServer: (serverId: string) => boolean;
  copyPasswordForServer: (serverId: string) => void | Promise<void>;
  isSshOpenForEcs: (ecs: EcsServer) => boolean;
  canConnectSsh: (ecs: EcsServer) => boolean;
  toggleSshForEcs: (ecs: EcsServer) => void | Promise<void>;
  sshButtonLabel: (ecs: EcsServer) => string;
  canStopEcs: (ecs: EcsServer) => boolean;
  stopEcs: (ecs: EcsServer) => void | Promise<void>;
  deleteEcs: (ecs: EcsServer) => void | Promise<void>;
  startPolling: (serverId: string | null) => void;
  stopPolling: () => void;
  reloadResponseData: () => void | Promise<void>;
  reloadEips: () => void | Promise<void>;
  reloadEvss: () => void | Promise<void>;
  reloadEcses: () => void | Promise<void>;
  closeSshPanel: () => void | Promise<void>;
  clearSshTerminal: () => void;
  reconnectSshForPanel: () => void | Promise<void>;
  disconnectActiveSsh: () => void | Promise<void>;
  runSshCommand: () => void | Promise<void>;
  sendSshControlShortcut: (control: "ctrl+c" | "ctrl+d" | "ctrl+u") => void | Promise<void>;
  handleSshTerminalResize: (size: { cols: number; rows: number }) => void;
  handleSshCommandKeydown: (event: KeyboardEvent) => void;
  setSshUseFormPassword: (value: boolean) => void;
  setSshManualPassword: (value: string) => void;
  setSshCommandInput: (value: string) => void;
  platformPanelOpen: boolean;
  platformPanelServer: EcsServer | null;
  platformPanelHost: string | null;
  platformPanelBusy: boolean;
  platformBusyServerId: string | null;
  platformActionLabel: string | null;
  platformError: string | null;
  platformInfo: string | null;
  platformActiveTab: PlatformOpsTab;
  platformDockerInstallEnabled: boolean;
  platformDockerImages: DockerImageSummary[];
  platformDockerContainers: DockerContainerSummary[];
  platformDockerfileTargetPath: string;
  platformDockerfileContent: string;
  platformMinikubeInstallEnabled: boolean;
  platformMinikubeEnsureDocker: boolean;
  platformMinikubeAutoStart: boolean;
  platformMinikubeProfile: string;
  platformMinikubeDriver: "docker" | "none";
  platformMinikubeCpus: number;
  platformMinikubeMemoryMb: number;
  platformMinikubeK8sVersion: string;
  platformMinikubeStatus: string;
  platformMinikubeNodes: string;
  platformMinikubePods: string;
  platformNixInstallEnabled: boolean;
  platformNixEnableFlakes: boolean;
  platformNixRunGarbageCollect: boolean;
  platformNixPackagesInput: string;
  platformNixVersion: string;
  platformNixPackages: NixPackageSummary[];
  platformNixStoreUsage: string;
  isPlatformOpenForEcs: (ecs: EcsServer) => boolean;
  platformButtonLabel: (ecs: EcsServer) => string;
  togglePlatformForEcs: (ecs: EcsServer) => void | Promise<void>;
  closePlatformPanel: () => void;
  setPlatformActiveTab: (value: PlatformOpsTab) => void;
  setPlatformDockerInstallEnabled: (value: boolean) => void;
  setPlatformDockerfileContent: (value: string) => void;
  setPlatformMinikubeInstallEnabled: (value: boolean) => void;
  setPlatformMinikubeEnsureDocker: (value: boolean) => void;
  setPlatformMinikubeAutoStart: (value: boolean) => void;
  setPlatformMinikubeProfile: (value: string) => void;
  setPlatformMinikubeDriver: (value: "docker" | "none") => void;
  setPlatformMinikubeCpus: (value: number) => void;
  setPlatformMinikubeMemoryMb: (value: number) => void;
  setPlatformMinikubeK8sVersion: (value: string) => void;
  setPlatformNixInstallEnabled: (value: boolean) => void;
  setPlatformNixEnableFlakes: (value: boolean) => void;
  setPlatformNixRunGarbageCollect: (value: boolean) => void;
  setPlatformNixPackagesInput: (value: string) => void;
  runPlatformDockerSetup: () => void | Promise<void>;
  refreshPlatformDockerImages: () => void | Promise<void>;
  refreshPlatformDockerContainers: () => void | Promise<void>;
  importPlatformDockerfile: (file: File) => void | Promise<void>;
  runPlatformMinikubeSetup: () => void | Promise<void>;
  refreshPlatformMinikubeStatus: () => void | Promise<void>;
  refreshPlatformMinikubeNodes: () => void | Promise<void>;
  refreshPlatformMinikubePods: () => void | Promise<void>;
  runPlatformNixSetup: () => void | Promise<void>;
  refreshPlatformNixVersion: () => void | Promise<void>;
  refreshPlatformNixPackages: () => void | Promise<void>;
  refreshPlatformNixStoreUsage: () => void | Promise<void>;
}>();

const dockerfileImportInput = ref<HTMLInputElement | null>(null);

const platformTabModel = computed({
  get: () => props.platformActiveTab,
  set: (value: PlatformOpsTab) => props.setPlatformActiveTab(value),
});
const platformDockerInstallModel = computed({
  get: () => props.platformDockerInstallEnabled,
  set: (value: boolean) => props.setPlatformDockerInstallEnabled(value),
});
const platformDockerfileContentModel = computed({
  get: () => props.platformDockerfileContent,
  set: (value: string) => props.setPlatformDockerfileContent(value),
});
const platformMinikubeInstallModel = computed({
  get: () => props.platformMinikubeInstallEnabled,
  set: (value: boolean) => props.setPlatformMinikubeInstallEnabled(value),
});
const platformMinikubeEnsureDockerModel = computed({
  get: () => props.platformMinikubeEnsureDocker,
  set: (value: boolean) => props.setPlatformMinikubeEnsureDocker(value),
});
const platformMinikubeAutoStartModel = computed({
  get: () => props.platformMinikubeAutoStart,
  set: (value: boolean) => props.setPlatformMinikubeAutoStart(value),
});
const platformMinikubeProfileModel = computed({
  get: () => props.platformMinikubeProfile,
  set: (value: string) => props.setPlatformMinikubeProfile(value),
});
const platformMinikubeDriverModel = computed({
  get: () => props.platformMinikubeDriver,
  set: (value: "docker" | "none") => props.setPlatformMinikubeDriver(value),
});
const platformMinikubeCpusModel = computed({
  get: () => props.platformMinikubeCpus,
  set: (value: number) => props.setPlatformMinikubeCpus(value),
});
const platformMinikubeMemoryMbModel = computed({
  get: () => props.platformMinikubeMemoryMb,
  set: (value: number) => props.setPlatformMinikubeMemoryMb(value),
});
const platformMinikubeK8sVersionModel = computed({
  get: () => props.platformMinikubeK8sVersion,
  set: (value: string) => props.setPlatformMinikubeK8sVersion(value),
});
const platformNixInstallModel = computed({
  get: () => props.platformNixInstallEnabled,
  set: (value: boolean) => props.setPlatformNixInstallEnabled(value),
});
const platformNixEnableFlakesModel = computed({
  get: () => props.platformNixEnableFlakes,
  set: (value: boolean) => props.setPlatformNixEnableFlakes(value),
});
const platformNixRunGarbageCollectModel = computed({
  get: () => props.platformNixRunGarbageCollect,
  set: (value: boolean) => props.setPlatformNixRunGarbageCollect(value),
});
const platformNixPackagesInputModel = computed({
  get: () => props.platformNixPackagesInput,
  set: (value: string) => props.setPlatformNixPackagesInput(value),
});

function triggerDockerfileImport() {
  if (props.platformPanelBusy) {
    return;
  }
  dockerfileImportInput.value?.click();
}

async function handleDockerfileImportChange(event: Event) {
  const input = event.target as HTMLInputElement | null;
  const file = input?.files?.[0] ?? null;
  if (file) {
    await props.importPlatformDockerfile(file);
  }
  if (input) {
    input.value = "";
  }
}
</script>

<template>
  <section class="panel output">
    <div class="output-header">
      <h2>Response</h2>
      <ReloadIconButton
        :disabled="loadingResponse"
        :loading="loadingResponse"
        :title="loadingResponse ? 'Reloading response data...' : 'Reload response data'"
        @click="reloadResponseData()"
      />
    </div>
    <p v-if="quickCopyFeedback" class="muted tiny quick-copy-feedback">{{ quickCopyFeedback }}</p>

    <p v-if="errorMsg" class="error">{{ errorMsg }}</p>
    <p v-if="deleteMsg" class="muted tiny">{{ deleteMsg }}</p>

    <div class="output-grid">
      <div class="output-col output-col-summary">
        <div class="output-card wide output-card-last">
          <div class="card-title">Last Create</div>
          <div v-if="createSummary" class="card-content">
            <div class="status-row">
              <span class="badge">{{ createSummary.statusCode }}</span>
              <span>{{ createSummary.status }}</span>
            </div>
            <div class="meta">
              <div>
                <span class="muted tiny">Server ID</span>
                <div class="mono">
                  {{ createSummary.serverId ?? "—" }}
                </div>
              </div>
              <div>
                <span class="muted tiny">Job ID</span>
                <div class="mono">{{ createSummary.jobId ?? "—" }}</div>
              </div>
            </div>
            <div v-if="createSummary.message" class="warning">
              {{ createSummary.message }}
            </div>
            <div class="meta">
              <div>
                <span class="muted tiny">Instance</span>
                <div>
                  {{ createdServer?.name ?? "Waiting for ECS data..." }}
                </div>
              </div>
              <div>
                <span class="muted tiny">Status</span>
                <div>{{ createdServer?.status ?? "—" }}</div>
              </div>
            </div>
            <div class="meta">
              <div>
                <span class="muted tiny">Public IP</span>
                <div class="mono">
                  {{ createdEip?.public_ip_address ?? "Not associated yet" }}
                </div>
              </div>
              <div>
                <span class="muted tiny">Association</span>
                <div class="mono">
                  {{ createdEip?.associate_instance_id ?? "—" }}
                </div>
              </div>
            </div>
            <div class="polling-row">
              <div>
                <span class="muted tiny">Polling</span>
                <div>
                  <span v-if="pollingEcs"> Active ({{ pollingAttempts }}/{{ pollMaxAttempts }}) </span>
                  <span v-else>Idle</span>
                  <span v-if="pollingStatus"> • {{ pollingStatus }} </span>
                </div>
                <div class="muted tiny">
                  Target:
                  {{
                    createSummary?.serverId ??
                    createdServer?.id ??
                    (ecses.length ? "Newest instance" : "—")
                  }}
                </div>
                <div v-if="pollingError" class="muted tiny">
                  {{ pollingError }}
                </div>
              </div>
              <div class="polling-actions">
                <button
                  class="ghost minor"
                  :disabled="pollingEcs || !canWatch"
                  @click="startPolling(createSummary?.serverId ?? createdServer?.id ?? null)"
                >
                  Start Watch
                </button>
                <button class="ghost minor" :disabled="!pollingEcs" @click="stopPolling">Stop</button>
              </div>
            </div>
          </div>
          <p v-else class="muted">No create action yet.</p>
          <details v-if="result" class="raw">
            <summary>Raw create response</summary>
            <pre class="body">{{ result.body }}</pre>
          </details>
        </div>

        <div class="output-card output-card-eips">
          <div class="card-head-inline">
            <div class="card-title">Elastic IPs</div>
            <ReloadIconButton
              :disabled="loadingEips"
              :loading="loadingEips"
              :title="loadingEips ? 'Reloading EIPs...' : 'Reload EIPs'"
              @click="reloadEips()"
            />
          </div>
          <div class="card-subtitle">{{ eips.length }} total • Updated {{ cacheAgeEips }}</div>
          <div v-if="eips.length" class="entity-list eip-list">
            <article
              v-for="(eip, index) in eips"
              :key="eip.id ?? eip.public_ip_address ?? `eip-${index}`"
              class="entity-item eip-item"
            >
              <div class="entity-item-head">
                <div class="entity-head-main">
                  <div class="entity-title-row">
                    <div class="entity-title mono">{{ eip.public_ip_address ?? "—" }}</div>
                    <button
                      class="ghost minor action-chip"
                      type="button"
                      :disabled="!eip.public_ip_address"
                      title="Copy EIP address"
                      @click="copyEipAddress(eip.public_ip_address)"
                    >
                      Copy IP
                    </button>
                  </div>
                </div>
                <span class="status-pill" :class="statusTone(eip.status)">
                  {{ eip.status ?? "UNKNOWN" }}
                </span>
              </div>
              <div class="entity-meta-grid">
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Association</span>
                  <span class="entity-meta-value mono">{{ eip.associate_instance_id ?? "—" }}</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">VPC</span>
                  <span class="entity-meta-value mono">{{ eip.vnic?.vpc_id ?? "—" }}</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Port</span>
                  <span class="entity-meta-value mono">{{ eip.vnic?.port_id ?? "—" }}</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Pool</span>
                  <span class="entity-meta-value">{{ eip.publicip_pool_name ?? "—" }}</span>
                </div>
              </div>
            </article>
          </div>
          <p v-else class="muted tiny">No elastic IPs found in this region.</p>
        </div>

        <div class="output-card output-card-evs">
          <div class="card-head-inline">
            <div class="card-title">EVS Disks</div>
            <ReloadIconButton
              :disabled="loadingEvss"
              :loading="loadingEvss"
              :title="loadingEvss ? 'Reloading EVS disks...' : 'Reload EVS disks'"
              @click="reloadEvss()"
            />
          </div>
          <div class="card-subtitle">{{ evss.length }} total • Updated {{ cacheAgeEvss }}</div>
          <div v-if="evss.length" class="entity-list evs-list">
            <article
              v-for="(volume, index) in evss"
              :key="volume.id ?? volume.name ?? `evs-${index}`"
              class="entity-item evs-item"
            >
              <div class="entity-item-head">
                <div class="entity-head-main">
                  <div class="entity-title mono">{{ volume.name ?? volume.id ?? "—" }}</div>
                  <div class="muted tiny">
                    ID: <span class="mono">{{ volume.id ?? "—" }}</span>
                  </div>
                </div>
                <span class="status-pill" :class="statusTone(volume.status)">
                  {{ volume.status ?? "UNKNOWN" }}
                </span>
              </div>
              <div class="entity-meta-grid">
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Role</span>
                  <span class="entity-meta-value">{{ evsRole(volume) }}</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Size</span>
                  <span class="entity-meta-value">{{ volume.size ?? "—" }} GB</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Type</span>
                  <span class="entity-meta-value">{{ volume.volume_type ?? "—" }}</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Attached ECS</span>
                  <span class="entity-meta-value mono">{{ evsAttachedServer(volume) }}</span>
                </div>
              </div>
            </article>
          </div>
          <p v-else class="muted tiny">No EVS disks found in this region.</p>
        </div>
      </div>

      <div class="output-col output-col-entities">
        <div class="output-card output-card-ecs">
          <div class="card-head-inline">
            <div class="card-title">ECS Instances</div>
            <ReloadIconButton
              :disabled="loadingEcses"
              :loading="loadingEcses"
              :title="loadingEcses ? 'Reloading ECS instances...' : 'Reload ECS instances'"
              @click="reloadEcses()"
            />
          </div>
          <div class="card-subtitle">{{ ecses.length }} total • Updated {{ cacheAgeEcses }}</div>
          <div v-if="ecses.length" class="entity-list ecs-list">
            <article
              v-for="(ecs, index) in ecses"
              :key="ecs.id ?? ecs.name ?? `ecs-${index}`"
              class="entity-item ecs-item"
            >
              <div class="entity-item-head">
                <div class="entity-head-main">
                  <div class="entity-title mono">{{ ecs.name ?? ecs.id ?? "—" }}</div>
                  <div class="muted tiny">ID: <span class="mono">{{ ecs.id ?? "—" }}</span></div>
                </div>
                <span class="status-pill" :class="statusTone(ecs.status)">
                  {{ ecs.status ?? "UNKNOWN" }}
                </span>
              </div>
              <div class="entity-meta-grid">
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Flavor</span>
                  <span class="entity-meta-value mono">{{ ecs.flavor?.name ?? ecs.flavor?.id ?? "—" }}</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">AZ</span>
                  <span class="entity-meta-value">{{ ecs.availability_zone ?? "—" }}</span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Public EIP</span>
                  <span class="entity-meta-value entity-inline-actions">
                    <span class="mono">{{ findSshHostForServer(ecs) ?? "Not assigned" }}</span>
                    <button
                      class="ghost minor action-chip"
                      type="button"
                      :disabled="!findSshHostForServer(ecs)"
                      title="Copy ECS public IP"
                      @click="copyEipAddress(findSshHostForServer(ecs))"
                    >
                      Copy IP
                    </button>
                  </span>
                </div>
                <div class="entity-meta-item">
                  <span class="entity-meta-key">Startup Tasks</span>
                  <span
                    class="entity-meta-value"
                    :class="{
                      'update-state-failed': autoUpdateStatusForServer(ecs.id ?? '') === 'failed',
                      'update-state-running': autoUpdateStatusForServer(ecs.id ?? '') === 'running',
                    }"
                  >
                    {{ autoUpdateStatusLabel(ecs) }}
                  </span>
                  <span v-if="autoUpdateProgressHint(ecs.id ?? '')" class="muted tiny update-progress-hint">
                    {{ autoUpdateProgressHint(ecs.id ?? "") }}
                  </span>
                  <span
                    v-if="startupTaskRdpUserForServer(ecs.id ?? '')"
                    class="muted tiny update-progress-hint"
                  >
                    RDP login: <span class="mono">{{ startupTaskRdpUserForServer(ecs.id ?? "") }}</span>
                    <span v-if="findSshHostForServer(ecs)">
                      @ <span class="mono">{{ findSshHostForServer(ecs) }}:3389</span>
                    </span>
                  </span>
                </div>
              </div>
              <div class="ecs-copy-row">
                <span class="muted tiny ecs-login-hint">
                  Login user: <span class="mono">{{ loginUsernameForServer(ecs.id ?? "") }}</span>
                </span>
                <button
                  class="ghost minor action-chip"
                  type="button"
                  title="Copy VM username"
                  @click="copyLoginUsernameForServer(ecs.id ?? '')"
                >
                  Copy User
                </button>
                <button
                  class="ghost minor action-chip"
                  type="button"
                  :disabled="!hasSavedPasswordForServer(ecs.id ?? '')"
                  title="Copy saved VM admin password"
                  @click="copyPasswordForServer(ecs.id ?? '')"
                >
                  Copy Password
                </button>
              </div>
              <div class="ecs-item-actions">
                <button
                  class="ghost minor ssh-action"
                  :class="{ active: isSshOpenForEcs(ecs) }"
                  :disabled="
                    !canConnectSsh(ecs) ||
                    sshBusyServerId === ecs.id ||
                    stoppingServerId === ecs.id ||
                    deletingServerId === ecs.id
                  "
                  @click="toggleSshForEcs(ecs)"
                >
                  {{ sshButtonLabel(ecs) }}
                </button>
                <button
                  class="ghost minor"
                  :class="{ active: isPlatformOpenForEcs(ecs) }"
                  :disabled="
                    !canConnectSsh(ecs) ||
                    !!platformBusyServerId ||
                    sshBusyServerId === ecs.id ||
                    stoppingServerId === ecs.id ||
                    deletingServerId === ecs.id
                  "
                  @click="togglePlatformForEcs(ecs)"
                >
                  {{ platformButtonLabel(ecs) }}
                </button>
                <button
                  class="ghost minor"
                  :disabled="!canStopEcs(ecs) || stoppingServerId === ecs.id || deletingServerId === ecs.id"
                  @click="stopEcs(ecs)"
                >
                  {{ stoppingServerId === ecs.id ? "Stopping..." : "Stop" }}
                </button>
                <button
                  class="ghost minor danger"
                  :disabled="!ecs.id || deletingServerId === ecs.id || stoppingServerId === ecs.id"
                  @click="deleteEcs(ecs)"
                >
                  {{ deletingServerId === ecs.id ? "Deleting..." : "Delete" }}
                </button>
              </div>
            </article>
          </div>
          <p v-else class="muted tiny">No ECS instances found in this region.</p>
        </div>

        <SshTerminalPanel
          :open="sshPanelOpen && !!sshPanelServer"
          :server-label="sshPanelServer?.name ?? sshPanelServer?.id ?? 'selected ECS'"
          :host="sshPanelHost"
          :connected="sshConnectedToPanel"
          :busy="!!sshPanelServer && sshBusyServerId === sshPanelServer.id"
          :running="sshRunningCommand"
          :use-form-password="sshUseFormPassword"
          :manual-password="sshManualPassword"
          :terminal-entries="sshTerminalEntries"
          :command-input="sshCommandInput"
          :can-reconnect="
            !!sshPanelServer &&
            canConnectSsh(sshPanelServer) &&
            sshBusyServerId !== sshPanelServer.id
          "
          :can-disconnect="
            sshConnectedToPanel &&
            (!sshPanelServer || sshBusyServerId !== sshPanelServer.id)
          "
          :can-run="sshConnectedToPanel && !sshRunningCommand && !!sshCommandInput.trim()"
          @close="closeSshPanel"
          @clear="clearSshTerminal"
          @reconnect="reconnectSshForPanel"
          @disconnect="disconnectActiveSsh()"
          @run="runSshCommand"
          @send-control="sendSshControlShortcut"
          @terminal-resize="handleSshTerminalResize"
          @command-keydown="handleSshCommandKeydown"
          @update:use-form-password="setSshUseFormPassword"
          @update:manual-password="setSshManualPassword"
          @update:command-input="setSshCommandInput"
        />

        <section v-if="platformPanelOpen && platformPanelServer" class="output-card platform-card">
          <div class="platform-card-head">
            <div>
              <div class="card-title">Platform Ops</div>
              <div class="card-subtitle">
                {{ platformPanelServer?.name ?? platformPanelServer?.id ?? "selected ECS" }}
              </div>
            </div>
            <button class="ghost minor close-button" type="button" :disabled="platformPanelBusy" @click="closePlatformPanel">
              Close
            </button>
          </div>

          <div class="platform-meta-row">
            <span class="muted tiny">Host: <span class="mono">{{ platformPanelHost ?? "No public EIP" }}</span></span>
            <span class="muted tiny">User: <span class="mono">root</span></span>
            <span class="muted tiny" v-if="platformPanelBusy">
              Running:
              <span class="mono">{{ platformActionLabel ?? "operation" }}</span>
            </span>
          </div>

          <p v-if="platformError" class="error">{{ platformError }}</p>
          <p v-else-if="platformInfo" class="muted tiny">{{ platformInfo }}</p>

          <div class="platform-tab-row">
            <button
              class="ghost minor"
              :class="{ active: platformTabModel === 'docker' }"
              :disabled="platformPanelBusy"
              @click="platformTabModel = 'docker'"
            >
              Docker
            </button>
            <button
              class="ghost minor"
              :class="{ active: platformTabModel === 'minikube' }"
              :disabled="platformPanelBusy"
              @click="platformTabModel = 'minikube'"
            >
              Minikube
            </button>
            <button
              class="ghost minor"
              :class="{ active: platformTabModel === 'nix' }"
              :disabled="platformPanelBusy"
              @click="platformTabModel = 'nix'"
            >
              Nix
            </button>
          </div>

          <div v-if="platformTabModel === 'docker'" class="platform-pane">
            <label class="toggle-inline">
              <input v-model="platformDockerInstallModel" type="checkbox" />
              <span>Install or repair Docker engine if missing</span>
            </label>

            <div class="platform-input-grid">
              <label class="field span-2">
                <span>Dockerfile (optional)</span>
                <textarea
                  v-model="platformDockerfileContentModel"
                  class="platform-dockerfile-editor"
                  rows="10"
                  spellcheck="false"
                  placeholder="Paste Dockerfile content here or import a Dockerfile."
                ></textarea>
                <span class="muted tiny">
                  Apply Docker Setup uploads this Dockerfile to
                  <span class="mono">{{ platformDockerfileTargetPath }}</span>.
                </span>
              </label>
            </div>

            <div class="platform-action-row">
              <button class="primary" :disabled="platformPanelBusy" @click="runPlatformDockerSetup">
                {{ platformPanelBusy ? "Running..." : "Apply Docker Setup" }}
              </button>
              <button class="ghost minor" :disabled="platformPanelBusy" @click="triggerDockerfileImport">
                Import Dockerfile
              </button>
              <input
                ref="dockerfileImportInput"
                class="platform-file-input"
                type="file"
                accept=".dockerfile,Dockerfile,text/plain"
                @change="handleDockerfileImportChange"
              />
            </div>

            <div class="platform-list-grid">
              <section class="platform-list-card">
                <div class="card-head-inline">
                  <strong>Docker Images</strong>
                  <div class="platform-list-head-actions">
                    <span class="muted tiny">{{ platformDockerImages.length }} total</span>
                    <ReloadIconButton
                      :disabled="platformPanelBusy"
                      :loading="platformPanelBusy && platformActionLabel === 'Docker image listing'"
                      :title="
                        platformPanelBusy && platformActionLabel === 'Docker image listing'
                          ? 'Refreshing Docker images...'
                          : 'Refresh Docker images'
                      "
                      @click="refreshPlatformDockerImages"
                    />
                  </div>
                </div>
                <div v-if="platformDockerImages.length" class="platform-record-list">
                  <article
                    v-for="(image, index) in platformDockerImages"
                    :key="`${image.id}-${index}`"
                    class="platform-record"
                  >
                    <div class="mono">
                      {{ image.repository }}:{{ image.tag }}
                    </div>
                    <div class="muted tiny">
                      {{ image.id }} • {{ image.size }} • {{ image.createdSince }}
                    </div>
                  </article>
                </div>
                <p v-else class="muted tiny">No Docker images listed yet.</p>
              </section>

              <section class="platform-list-card">
                <div class="card-head-inline">
                  <strong>Container Instances</strong>
                  <div class="platform-list-head-actions">
                    <span class="muted tiny">{{ platformDockerContainers.length }} total</span>
                    <ReloadIconButton
                      :disabled="platformPanelBusy"
                      :loading="platformPanelBusy && platformActionLabel === 'Docker container listing'"
                      :title="
                        platformPanelBusy && platformActionLabel === 'Docker container listing'
                          ? 'Refreshing Docker containers...'
                          : 'Refresh Docker containers'
                      "
                      @click="refreshPlatformDockerContainers"
                    />
                  </div>
                </div>
                <div v-if="platformDockerContainers.length" class="platform-record-list">
                  <article
                    v-for="(container, index) in platformDockerContainers"
                    :key="`${container.id}-${index}`"
                    class="platform-record"
                  >
                    <div class="mono">{{ container.name }} ({{ container.image }})</div>
                    <div class="muted tiny">{{ container.status }}</div>
                    <div class="muted tiny">{{ container.ports }}</div>
                  </article>
                </div>
                <p v-else class="muted tiny">No Docker containers listed yet.</p>
              </section>
            </div>
          </div>

          <div v-else-if="platformTabModel === 'minikube'" class="platform-pane">
            <div class="platform-toggle-grid">
              <label class="toggle-inline">
                <input v-model="platformMinikubeInstallModel" type="checkbox" />
                <span>Install Minikube and kubectl if missing</span>
              </label>
              <label class="toggle-inline">
                <input v-model="platformMinikubeEnsureDockerModel" type="checkbox" />
                <span>Ensure Docker is available for Docker driver</span>
              </label>
              <label class="toggle-inline">
                <input v-model="platformMinikubeAutoStartModel" type="checkbox" />
                <span>Auto-start cluster after setup</span>
              </label>
            </div>

            <div class="platform-input-grid">
              <label class="field">
                <span>Profile</span>
                <input v-model="platformMinikubeProfileModel" placeholder="hcforge" />
              </label>
              <label class="field">
                <span>Driver</span>
                <select v-model="platformMinikubeDriverModel">
                  <option value="docker">docker</option>
                  <option value="none">none</option>
                </select>
              </label>
              <label class="field">
                <span>CPUs</span>
                <input v-model.number="platformMinikubeCpusModel" type="number" min="1" max="64" step="1" />
              </label>
              <label class="field">
                <span>Memory (MB)</span>
                <input
                  v-model.number="platformMinikubeMemoryMbModel"
                  type="number"
                  min="1024"
                  max="262144"
                  step="256"
                />
              </label>
              <label class="field span-2">
                <span>Kubernetes Version (optional)</span>
                <input v-model="platformMinikubeK8sVersionModel" placeholder="v1.31.0" />
              </label>
            </div>

            <div class="platform-action-row">
              <button class="primary" :disabled="platformPanelBusy" @click="runPlatformMinikubeSetup">
                {{ platformPanelBusy ? "Running..." : "Apply Minikube Setup" }}
              </button>
            </div>

            <div class="platform-list-grid">
              <section class="platform-list-card">
                <div class="card-head-inline">
                  <strong>Status</strong>
                  <ReloadIconButton
                    :disabled="platformPanelBusy"
                    :loading="platformPanelBusy && platformActionLabel === 'Minikube status'"
                    :title="
                      platformPanelBusy && platformActionLabel === 'Minikube status'
                        ? 'Refreshing Minikube status...'
                        : 'Refresh Minikube status'
                    "
                    @click="refreshPlatformMinikubeStatus"
                  />
                </div>
                <pre class="platform-raw">{{ platformMinikubeStatus || "Run Cluster Status to view output." }}</pre>
              </section>
              <section class="platform-list-card">
                <div class="card-head-inline">
                  <strong>Nodes</strong>
                  <ReloadIconButton
                    :disabled="platformPanelBusy"
                    :loading="platformPanelBusy && platformActionLabel === 'Minikube nodes listing'"
                    :title="
                      platformPanelBusy && platformActionLabel === 'Minikube nodes listing'
                        ? 'Refreshing Minikube nodes...'
                        : 'Refresh Minikube nodes'
                    "
                    @click="refreshPlatformMinikubeNodes"
                  />
                </div>
                <pre class="platform-raw">{{ platformMinikubeNodes || "Run Nodes to view cluster nodes." }}</pre>
              </section>
              <section class="platform-list-card platform-list-card-wide">
                <div class="card-head-inline">
                  <strong>Pods (All Namespaces)</strong>
                  <ReloadIconButton
                    :disabled="platformPanelBusy"
                    :loading="platformPanelBusy && platformActionLabel === 'Minikube pods listing'"
                    :title="
                      platformPanelBusy && platformActionLabel === 'Minikube pods listing'
                        ? 'Refreshing Minikube pods...'
                        : 'Refresh Minikube pods'
                    "
                    @click="refreshPlatformMinikubePods"
                  />
                </div>
                <pre class="platform-raw">{{ platformMinikubePods || "Run Pods to view cluster pods." }}</pre>
              </section>
            </div>
          </div>

          <div v-else class="platform-pane">
            <div class="platform-toggle-grid">
              <label class="toggle-inline">
                <input v-model="platformNixInstallModel" type="checkbox" />
                <span>Install Nix package manager if missing</span>
              </label>
              <label class="toggle-inline">
                <input v-model="platformNixEnableFlakesModel" type="checkbox" />
                <span>Enable flakes + nix-command in nix.conf</span>
              </label>
              <label class="toggle-inline">
                <input v-model="platformNixRunGarbageCollectModel" type="checkbox" />
                <span>Run garbage collection after setup</span>
              </label>
            </div>

            <div class="platform-input-grid">
              <label class="field span-2">
                <span>Bootstrap Packages (optional)</span>
                <textarea
                  v-model="platformNixPackagesInputModel"
                  class="platform-dockerfile-editor"
                  rows="5"
                  spellcheck="false"
                  placeholder="Examples: git ripgrep fd htop"
                ></textarea>
                <span class="muted tiny">
                  Packages can be separated by spaces, commas, or new lines.
                </span>
              </label>
            </div>

            <div class="platform-action-row">
              <button class="primary" :disabled="platformPanelBusy" @click="runPlatformNixSetup">
                {{ platformPanelBusy ? "Running..." : "Apply Nix Setup" }}
              </button>
            </div>

            <div class="platform-list-grid">
              <section class="platform-list-card">
                <div class="card-head-inline">
                  <strong>Nix Version</strong>
                  <ReloadIconButton
                    :disabled="platformPanelBusy"
                    :loading="platformPanelBusy && platformActionLabel === 'Nix version inspection'"
                    :title="
                      platformPanelBusy && platformActionLabel === 'Nix version inspection'
                        ? 'Refreshing Nix version info...'
                        : 'Refresh Nix version info'
                    "
                    @click="refreshPlatformNixVersion"
                  />
                </div>
                <pre class="platform-raw">{{ platformNixVersion || "Run refresh to load Nix version details." }}</pre>
              </section>

              <section class="platform-list-card">
                <div class="card-head-inline">
                  <strong>Installed Packages</strong>
                  <div class="platform-list-head-actions">
                    <span class="muted tiny">{{ platformNixPackages.length }} total</span>
                    <ReloadIconButton
                      :disabled="platformPanelBusy"
                      :loading="platformPanelBusy && platformActionLabel === 'Nix package listing'"
                      :title="
                        platformPanelBusy && platformActionLabel === 'Nix package listing'
                          ? 'Refreshing installed Nix packages...'
                          : 'Refresh installed Nix packages'
                      "
                      @click="refreshPlatformNixPackages"
                    />
                  </div>
                </div>
                <div v-if="platformNixPackages.length" class="platform-record-list">
                  <article
                    v-for="(pkg, index) in platformNixPackages"
                    :key="`${pkg.name}-${pkg.version}-${index}`"
                    class="platform-record"
                  >
                    <div class="mono">{{ pkg.name }}</div>
                    <div class="muted tiny">version: {{ pkg.version }}</div>
                    <div class="muted tiny">source: {{ pkg.source }}</div>
                  </article>
                </div>
                <p v-else class="muted tiny">No Nix packages listed yet.</p>
              </section>

              <section class="platform-list-card platform-list-card-wide">
                <div class="card-head-inline">
                  <strong>Store Usage</strong>
                  <ReloadIconButton
                    :disabled="platformPanelBusy"
                    :loading="platformPanelBusy && platformActionLabel === 'Nix store usage'"
                    :title="
                      platformPanelBusy && platformActionLabel === 'Nix store usage'
                        ? 'Refreshing Nix store usage...'
                        : 'Refresh Nix store usage'
                    "
                    @click="refreshPlatformNixStoreUsage"
                  />
                </div>
                <pre class="platform-raw">{{ platformNixStoreUsage || "Run refresh to inspect the Nix store." }}</pre>
              </section>
            </div>
          </div>
        </section>
      </div>
    </div>
  </section>
</template>
