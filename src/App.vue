<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { LogLevel, attachLogger } from "@tauri-apps/plugin-log";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { load } from "@tauri-apps/plugin-store";
import AppLogsPanel from "./components/AppLogsPanel.vue";
import ReloadIconButton from "./components/ReloadIconButton.vue";
import SshTerminalPanel from "./components/SshTerminalPanel.vue";

type VpcOption = { id: string; name: string };
type SubnetOption = { id: string; name: string; cidr: string };
type ImageOption = {
  id: string;
  name: string;
  min_disk?: number | null;
  min_ram?: number | null;
};
type FlavorOption = {
  id: string;
  name: string;
  vcpus?: number | null;
  ram?: number | null;
  disk?: number | null;
  os_extra_specs?: Record<string, string>;
};
type EipVnic = {
  private_ip_address?: string | null;
  device_id?: string | null;
  vpc_id?: string | null;
  port_id?: string | null;
  instance_id?: string | null;
};
type EipBandwidth = {
  size?: number | null;
  share_type?: string | null;
  charge_mode?: string | null;
};
type EipRecord = {
  id?: string | null;
  public_ip_address?: string | null;
  status?: string | null;
  associate_instance_id?: string | null;
  associate_instance_type?: string | null;
  publicip_pool_name?: string | null;
  vnic?: EipVnic | null;
  bandwidth?: EipBandwidth | null;
};
type EipListResponse = {
  publicips?: EipRecord[];
  total_count?: number | null;
};
type EvsAttachment = {
  id?: string | null;
  server_id?: string | null;
  device?: string | null;
  attached_at?: string | null;
};
type EvsVolume = {
  id?: string | null;
  name?: string | null;
  status?: string | null;
  size?: number | null;
  volume_type?: string | null;
  availability_zone?: string | null;
  bootable?: boolean | null;
  multiattach?: boolean | null;
  created_at?: string | null;
  updated_at?: string | null;
  attachments?: EvsAttachment[];
};
type EvsListResponse = {
  volumes?: EvsVolume[];
  count?: number | null;
};
type EcsFlavorInfo = {
  name?: string | null;
  id?: string | null;
  vcpus?: number | null;
  ram?: number | null;
};
type EcsServer = {
  id?: string | null;
  name?: string | null;
  status?: string | null;
  availability_zone?: string | null;
  flavor?: EcsFlavorInfo | null;
  created?: string | null;
};
type EcsListResponse = {
  servers?: EcsServer[];
};
type CreateEcsResult = { status: string; status_code: number; body: string };
type CredentialsPayload = { accessKey: string; secretKey: string };
type DeleteOperationResult = {
  status: string;
  status_code?: number | null;
  body: string;
};
type DeleteEcsResult = {
  ecs: DeleteOperationResult;
  eip?: DeleteOperationResult | null;
};
type StopEcsResult = {
  ecs: DeleteOperationResult;
};
type SshConnectResult = {
  sessionId: string;
  host: string;
  port: number;
  username: string;
  connectedAt: string;
};
type SshExecResult = {
  sessionId: string;
  command: string;
  stdout: string;
  stderr: string;
  exitStatus?: number | null;
};
type SshDisconnectResult = {
  sessionId: string;
  disconnected: boolean;
};
type SshResizeResult = {
  sessionId: string;
  cols: number;
  rows: number;
};
type SshSendControlResult = {
  sessionId: string;
  control: string;
  sent: boolean;
};
type SshExecOneShotResult = {
  sessionId: string;
  host: string;
  port: number;
  username: string;
  command: string;
  stdout: string;
  stderr: string;
  exitStatus?: number | null;
};
type SshStreamEventPayload = {
  sessionId: string;
  kind: "meta" | "stdout" | "stderr";
  text: string;
  at: string;
};
type SshSessionInfo = SshConnectResult & {
  serverId: string;
  serverName: string;
};
type SshTerminalEntry = {
  id: number;
  at: string;
  kind: "meta" | "command" | "stdout" | "stderr";
  text: string;
};
type CachedEntry<T> = {
  updatedAt: string;
  data: T;
};
type CachedResource =
  | "images"
  | "flavors"
  | "vpcs"
  | "subnets"
  | "eips"
  | "evss"
  | "ecses";
type FlavorGroup = {
  key: string;
  label: string;
  flavors: FlavorOption[];
};
type LogSource = "app" | "backend" | "runtime";
type LogLevelName = "trace" | "debug" | "info" | "warn" | "error";
type AppLogEntry = {
  id: number;
  at: string;
  source: LogSource;
  level: LogLevelName;
  message: string;
};
type ConfirmDialogKind = "info" | "warning" | "error";
type ConfirmDialogState = {
  open: boolean;
  title: string;
  message: string;
  kind: ConfirmDialogKind;
  okLabel: string;
  cancelLabel: string;
};
type AutoUpdateProgressInfo = {
  sessionId: string | null;
  startedAt: string | null;
  finishedAt: string | null;
  percent: number | null;
  lastLine: string | null;
};
type StartupTaskConfig = {
  region: string;
  autoUpdate: boolean;
  setupGuiRdp: boolean;
  lastStatus: "pending" | "done" | "failed";
  createdAt: string;
  updatedAt: string;
};
type PendingStartupTaskCreate = {
  config: StartupTaskConfig;
  password: string;
};
type StoredServerPassword = {
  version: 1;
  saltB64: string;
  ivB64: string;
  cipherB64: string;
  updatedAt: string;
};

const regions = [
  "sa-brazil-1",
  "af-north-1",
  "af-south-1",
  "ap-southeast-1",
  "ap-southeast-2",
  "ap-southeast-3",
  "cn-east-3",
  "cn-north-4",
  "cn-south-1",
  "cn-southwest-2",
  "la-south-2",
  "tr-west-1",
] as const;

const POLL_INTERVAL_MS = 8000;
const POLL_MAX_ATTEMPTS = 30;
const CACHE_PREFIX = "cache.v1";
const RELATIVE_TIME_TICK_MS = 30000;
const MAX_LOG_ENTRIES = 350;

const PASSWORD_MIN_LENGTH = 8;
const PASSWORD_MAX_LENGTH = 26;
const PASSWORD_UPPER = "ABCDEFGHJKLMNPQRSTUVWXYZ";
const PASSWORD_LOWER = "abcdefghijkmnopqrstuvwxyz";
const PASSWORD_DIGITS = "23456789";
const PASSWORD_SYMBOLS = "!@#$%^*_-+=?";
const DEFAULT_EIP_BANDWIDTH_MBIT = 100;
const EIP_BANDWIDTH_MIN_MBIT = 1;
const EIP_BANDWIDTH_MAX_MBIT = 300;
const DEFAULT_DATA_DISK_SIZE_GB = 100;
const DATA_DISK_MIN_GB = 10;
const DATA_DISK_MAX_GB = 32768;
const DATA_DISK_MIN_COUNT = 1;
const DATA_DISK_MAX_COUNT = 24;
const DEFAULT_IMAGE_QUERY = "ubuntu 24";
const DEFAULT_FLAVOR_QUERY = "x1.4u.8g";
const AUTO_UPDATE_SESSION_PREFIX = "auto-update:";
const STORE_KEY_PASSWORDS_BY_SERVER = "serverPasswords.v1";
const STORE_KEY_STARTUP_TASKS_BY_SERVER = "startupTasks.v1";
const PBKDF2_ITERATIONS = 200_000;
const PBKDF2_SALT_BYTES = 16;
const AES_GCM_IV_BYTES = 12;
const USER_DATE_FORMATTER = new Intl.DateTimeFormat(undefined, {
  year: "numeric",
  month: "short",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});
const AUTO_VM_UPDATE_COMMAND = `
echo "[hc-forge] [progress] 2 Startup package update started."
if command -v apt-get >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 8 Package manager: apt-get"
  export DEBIAN_FRONTEND=noninteractive
  apt-get update
  echo "[hc-forge] [progress] 26 apt metadata refreshed."
  apt-get -y -o Dpkg::Options::=--force-confnew dist-upgrade
  echo "[hc-forge] [progress] 78 apt dist-upgrade complete."
  apt-get -y autoremove --purge
  echo "[hc-forge] [progress] 92 apt autoremove complete."
elif command -v dnf >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 8 Package manager: dnf"
  dnf -y upgrade --refresh
  echo "[hc-forge] [progress] 92 dnf upgrade complete."
elif command -v yum >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 8 Package manager: yum"
  yum -y update
  echo "[hc-forge] [progress] 92 yum update complete."
elif command -v zypper >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 8 Package manager: zypper"
  zypper --non-interactive refresh
  echo "[hc-forge] [progress] 26 zypper refresh complete."
  zypper --non-interactive update
  echo "[hc-forge] [progress] 92 zypper update complete."
elif command -v pacman >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 8 Package manager: pacman"
  pacman -Syu --noconfirm
  echo "[hc-forge] [progress] 92 pacman upgrade complete."
elif command -v apk >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 8 Package manager: apk"
  apk update
  echo "[hc-forge] [progress] 28 apk metadata refreshed."
  apk upgrade
  echo "[hc-forge] [progress] 92 apk upgrade complete."
else
  echo "No supported package manager found for automatic updates."
  exit 2
fi
echo "[hc-forge] [progress] 100 Startup package update finished."
`.trim();
const SETUP_GUI_RDP_COMMAND = `
echo "[hc-forge] [progress] 5 Desktop+RDP setup started."
if command -v apt-get >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 12 Package manager: apt-get"
  export DEBIAN_FRONTEND=noninteractive
  apt-get update
  echo "[hc-forge] [progress] 26 apt metadata refreshed."
  apt-get install -y --no-install-recommends xorg xrdp xterm icewm dbus-x11
  echo "[hc-forge] [progress] 78 apt packages installed."
elif command -v dnf >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 12 Package manager: dnf"
  dnf -y install xrdp xorgxrdp xorg-x11-server-Xorg xterm icewm || dnf -y install xrdp xterm icewm
  echo "[hc-forge] [progress] 78 dnf packages installed."
elif command -v yum >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 12 Package manager: yum"
  yum -y install xrdp xorgxrdp xterm icewm || yum -y install xrdp xterm icewm
  echo "[hc-forge] [progress] 78 yum packages installed."
elif command -v zypper >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 12 Package manager: zypper"
  zypper --non-interactive refresh
  zypper --non-interactive install -y xrdp xorg-x11-server xterm icewm
  echo "[hc-forge] [progress] 78 zypper packages installed."
elif command -v pacman >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 12 Package manager: pacman"
  pacman -Syu --noconfirm xorg-server xorg-xinit xterm icewm xrdp
  echo "[hc-forge] [progress] 78 pacman packages installed."
elif command -v apk >/dev/null 2>&1; then
  echo "[hc-forge] [progress] 12 Package manager: apk"
  apk update
  apk add xrdp xorg-server xinit xterm icewm dbus
  echo "[hc-forge] [progress] 78 apk packages installed."
else
  echo "No supported package manager found for Desktop+RDP setup."
  exit 2
fi

if command -v systemctl >/dev/null 2>&1; then
  systemctl enable xrdp || true
  systemctl restart xrdp || true
elif command -v rc-update >/dev/null 2>&1; then
  rc-update add xrdp default || true
  rc-service xrdp restart || true
elif command -v service >/dev/null 2>&1; then
  service xrdp restart || true
fi
echo "[hc-forge] [progress] 100 Desktop+RDP setup finished."
`.trim();

const region = ref("sa-brazil-1");
const name = ref("");
const imageId = ref("");
const imageSearch = ref("");
const imageVisibility = ref("public");
const imageType = ref("gold");
const useCustomName = ref(false);
const flavorId = ref("");
const flavorSearch = ref("");
const flavorArchFilter = ref("all");
const flavorVcpuFilter = ref("all");
const rootVolumeType = ref("GPSSD");
const rootVolumeSize = ref(40);
const includeDataDisk = ref(false);
const dataDiskType = ref("GPSSD");
const dataDiskSize = ref(DEFAULT_DATA_DISK_SIZE_GB);
const dataDiskCount = ref(1);
const dataDiskMultiattach = ref(false);
const dataDiskHwPassthrough = ref(false);
const allocateEip = ref(true);
const eipBandwidthSize = ref(DEFAULT_EIP_BANDWIDTH_MBIT);
const accessKey = ref("");
const secretKey = ref("");
const passwordSectionOpen = ref(false);
const storageSectionOpen = ref(false);
const imageFilterSectionOpen = ref(false);
const networkSectionOpen = ref(false);

const useGeneratedPassword = ref(true);
const generatedPassword = ref(generatePassword());
const customPassword = ref("");
const passwordCopyFeedback = ref<string | null>(null);
const showAdminPassword = ref(false);

const vpcs = ref<VpcOption[]>([]);
const subnets = ref<SubnetOption[]>([]);
const images = ref<ImageOption[]>([]);
const flavors = ref<FlavorOption[]>([]);
const eips = ref<EipRecord[]>([]);
const evss = ref<EvsVolume[]>([]);
const ecses = ref<EcsServer[]>([]);
const selectedVpc = ref("");
const selectedSubnet = ref("");

const loadingVpcs = ref(false);
const loadingSubnets = ref(false);
const loadingImages = ref(false);
const loadingFlavors = ref(false);
const loadingEips = ref(false);
const loadingEvss = ref(false);
const loadingEcses = ref(false);
const savingCredentials = ref(false);
const loadingAll = ref(false);
const creating = ref(false);
const deletingServerId = ref<string | null>(null);
const stoppingServerId = ref<string | null>(null);

const errorMsg = ref("");
const deleteMsg = ref<string | null>(null);
const logPanelOpen = ref(false);
const logEntries = ref<AppLogEntry[]>([]);
const logsUnreadError = ref(false);
const autoUpdateVmOnStartup = ref(false);
const setupGuiRdpOnStartup = ref(false);
const autoUpdatePendingServerIds = ref<string[]>([]);
const autoUpdateRunningServerId = ref<string | null>(null);
const autoUpdateDoneServerIds = ref<string[]>([]);
const autoUpdateFailedServerIds = ref<string[]>([]);
const autoUpdateProgressByServer = ref<Record<string, AutoUpdateProgressInfo>>({});
const startupTaskConfigsByServer = ref<Record<string, StartupTaskConfig>>({});
const pendingStartupTaskCreate = ref<PendingStartupTaskCreate | null>(null);
const pendingCreatedServerPassword = ref<string | null>(null);
const confirmDialog = ref<ConfirmDialogState>({
  open: false,
  title: "Confirm Action",
  message: "",
  kind: "warning",
  okLabel: "Confirm",
  cancelLabel: "Cancel",
});
const result = ref<CreateEcsResult | null>(null);
const createdServer = ref<EcsServer | null>(null);
const createdEip = ref<EipRecord | null>(null);
const sshPanelOpen = ref(false);
const sshPanelServerId = ref<string | null>(null);
const sshSession = ref<SshSessionInfo | null>(null);
const sshBusyServerId = ref<string | null>(null);
const sshRunningCommand = ref(false);
const sshUseFormPassword = ref(true);
const sshManualPassword = ref("");
const sshCommandInput = ref("");
const sshCommandHistory = ref<string[]>([]);
const sshHistoryCursor = ref(-1);
const sshTerminalEntries = ref<SshTerminalEntry[]>([]);
const sshLastResize = ref<{ cols: number; rows: number } | null>(null);
const createSummary = ref<{
  status: string;
  statusCode: number;
  serverId?: string | null;
  jobId?: string | null;
  message?: string | null;
} | null>(null);
const pollingEcs = ref(false);
const pollingAttempts = ref(0);
const pollingStatus = ref<string | null>(null);
const pollingError = ref<string | null>(null);
const pollingActiveRefreshDone = ref(false);

const cacheUpdatedAt = ref<Record<CachedResource, string | null>>({
  images: null,
  flavors: null,
  vpcs: null,
  subnets: null,
  eips: null,
  evss: null,
  ecses: null,
});
const nowMs = ref(Date.now());

let pollingTimer: number | null = null;
let passwordFeedbackTimer: number | null = null;
let relativeClockTimer: number | null = null;
let sshResizeTimer: number | null = null;
let logSeq = 0;
let backendLogUnlisten: UnlistenFn | null = null;
let sshOutputUnlisten: UnlistenFn | null = null;
let errorListener: ((event: ErrorEvent) => void) | null = null;
let rejectionListener: ((event: PromiseRejectionEvent) => void) | null = null;
let sshTerminalSeq = 0;
let resolveConfirmDialog: ((value: boolean) => void) | null = null;
let autoUpdateDrainInFlight = false;
const autoUpdateSessionToServerId = new Map<string, string>();
const autoUpdateSessionLineBuffer = new Map<string, string>();
const serverPasswords = ref<Record<string, string>>({});
const encryptedServerPasswords = ref<Record<string, StoredServerPassword>>({});
let notificationPermissionChecked = false;

let store: Awaited<ReturnType<typeof load>> | null = null;
const storeReady = ref(false);

const canWatch = computed(
  () =>
    !!createSummary.value?.serverId ||
    !!createdServer.value?.id ||
    ecses.value.length > 0
);

const selectedPassword = computed(() =>
  useGeneratedPassword.value ? generatedPassword.value : customPassword.value.trim()
);

const passwordError = computed(() => validatePassword(selectedPassword.value));

const canLoadSubnets = computed(() => !!selectedVpc.value && !loadingSubnets.value);
const canCreate = computed(
  () =>
    !!imageId.value &&
    !!flavorId.value &&
    !!selectedVpc.value &&
    !!selectedSubnet.value &&
    (!useCustomName.value || !!name.value.trim()) &&
    !passwordError.value &&
    !creating.value
);

const canListImages = computed(() => !!region.value && !loadingImages.value);
const canListFlavors = computed(() => !!region.value && !loadingFlavors.value);

const imageMinDisk = computed(() => {
  const image = images.value.find((item) => item.id === imageId.value);
  const minDisk = image?.min_disk ?? 1;
  return Math.min(Math.max(minDisk, 1), 1024);
});

const imageMinRam = computed(() => {
  const image = images.value.find((item) => item.id === imageId.value);
  return image?.min_ram ?? 0;
});

function searchScore(text: string, query: string): number {
  const haystack = text.toLowerCase();
  if (!query) {
    return 0;
  }
  if (haystack === query) {
    return 0;
  }
  if (haystack.startsWith(query)) {
    return 1;
  }
  const tokenIndex = haystack.indexOf(` ${query}`);
  if (tokenIndex >= 0) {
    return 2 + tokenIndex / 1000;
  }
  const matchIndex = haystack.indexOf(query);
  if (matchIndex >= 0) {
    return 4 + matchIndex / 1000;
  }
  return Number.POSITIVE_INFINITY;
}

const filteredImages = computed(() => {
  const query = imageSearch.value.trim().toLowerCase();
  if (!query) {
    return images.value;
  }

  return images.value
    .filter((image) => {
      const imageName = image.name.toLowerCase();
      return imageName.includes(query) || image.id.toLowerCase().includes(query);
    })
    .sort((a, b) => {
      const scoreA = Math.min(searchScore(a.name, query), searchScore(a.id, query));
      const scoreB = Math.min(searchScore(b.name, query), searchScore(b.id, query));
      if (scoreA !== scoreB) {
        return scoreA - scoreB;
      }
      return a.name.localeCompare(b.name);
    });
});

const flavorArchitectureOptions = computed(() => {
  const set = new Set<string>();
  for (const flavor of flavors.value) {
    set.add(flavorArchitecture(flavor));
  }
  const sorted = Array.from(set).sort((a, b) => {
    const rankDiff = architectureSortRank(a) - architectureSortRank(b);
    if (rankDiff !== 0) {
      return rankDiff;
    }
    return a.localeCompare(b);
  });
  return ["all", ...sorted];
});

const filteredFlavors = computed(() => {
  const query = flavorSearch.value.trim().toLowerCase();
  const minRam = imageMinRam.value;

  const matched = flavors.value
    .filter((flavor) => {
      if (!minRam || flavor.ram == null) {
        return true;
      }
      return flavor.ram >= minRam;
    })
    .filter((flavor) => {
      if (flavorArchFilter.value === "all") {
        return true;
      }
      return flavorArchitecture(flavor) === flavorArchFilter.value;
    })
    .filter((flavor) => matchesVcpuBucket(flavor, flavorVcpuFilter.value))
    .filter((flavor) => {
      if (!query) {
        return true;
      }
      return (
        flavor.name.toLowerCase().includes(query) ||
        flavor.id.toLowerCase().includes(query)
      );
    });

  if (!query) {
    return matched;
  }

  return matched.sort((a, b) => {
    const aText = `${a.name} ${a.id}`;
    const bText = `${b.name} ${b.id}`;
    const scoreA = searchScore(aText, query);
    const scoreB = searchScore(bText, query);
    if (scoreA !== scoreB) {
      return scoreA - scoreB;
    }
    return a.name.localeCompare(b.name);
  });
});

const flavorGroups = computed<FlavorGroup[]>(() => {
  const grouped = new Map<string, FlavorOption[]>();

  for (const flavor of filteredFlavors.value) {
    const key = flavorArchitecture(flavor);
    const list = grouped.get(key) ?? [];
    list.push(flavor);
    grouped.set(key, list);
  }

  return Array.from(grouped.entries())
    .sort(([a], [b]) => {
      const rankDiff = architectureSortRank(a) - architectureSortRank(b);
      if (rankDiff !== 0) {
        return rankDiff;
      }
      return a.localeCompare(b);
    })
    .map(([key, list]) => {
      const sorted = [...list].sort((a, b) => {
        const vcpuDiff = (a.vcpus ?? 0) - (b.vcpus ?? 0);
        if (vcpuDiff !== 0) {
          return vcpuDiff;
        }
        const ramDiff = (a.ram ?? 0) - (b.ram ?? 0);
        if (ramDiff !== 0) {
          return ramDiff;
        }
        return a.name.localeCompare(b.name);
      });

      return {
        key,
        label: `${key} (${sorted.length})`,
        flavors: sorted,
      };
    });
});

const cacheAge = computed(() => {
  nowMs.value;
  return {
    images: formatRelativeTimestamp(cacheUpdatedAt.value.images),
    flavors: formatRelativeTimestamp(cacheUpdatedAt.value.flavors),
    vpcs: formatRelativeTimestamp(cacheUpdatedAt.value.vpcs),
    subnets: formatRelativeTimestamp(cacheUpdatedAt.value.subnets),
    eips: formatRelativeTimestamp(cacheUpdatedAt.value.eips),
    evss: formatRelativeTimestamp(cacheUpdatedAt.value.evss),
    ecses: formatRelativeTimestamp(cacheUpdatedAt.value.ecses),
  };
});

const sshPanelServer = computed(() => {
  const serverId = sshPanelServerId.value;
  if (!serverId) {
    return null;
  }
  return ecses.value.find((item) => item.id === serverId) ?? null;
});

const sshPanelHost = computed(() => {
  const server = sshPanelServer.value;
  if (!server) {
    return null;
  }
  return findSshHostForServer(server);
});

const sshConnectedToPanel = computed(() => {
  if (!sshPanelServerId.value || !sshSession.value) {
    return false;
  }
  return sshSession.value.serverId === sshPanelServerId.value;
});

const orderedLogEntries = computed(() =>
  [...logEntries.value].sort((a, b) => b.id - a.id)
);

watch(imageMinDisk, (minDisk) => {
  if (!rootVolumeSize.value || rootVolumeSize.value < minDisk) {
    rootVolumeSize.value = minDisk;
  }
  if (rootVolumeSize.value > 1024) {
    rootVolumeSize.value = 1024;
  }
});

watch(eipBandwidthSize, (value) => {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    eipBandwidthSize.value = EIP_BANDWIDTH_MIN_MBIT;
    return;
  }

  const sanitized = Math.min(
    EIP_BANDWIDTH_MAX_MBIT,
    Math.max(EIP_BANDWIDTH_MIN_MBIT, Math.trunc(parsed))
  );
  if (sanitized !== value) {
    eipBandwidthSize.value = sanitized;
  }
});

watch(dataDiskSize, (value) => {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    dataDiskSize.value = DATA_DISK_MIN_GB;
    return;
  }
  const sanitized = Math.min(DATA_DISK_MAX_GB, Math.max(DATA_DISK_MIN_GB, Math.trunc(parsed)));
  if (sanitized !== value) {
    dataDiskSize.value = sanitized;
  }
});

watch(dataDiskCount, (value) => {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    dataDiskCount.value = DATA_DISK_MIN_COUNT;
    return;
  }
  const sanitized = Math.min(
    DATA_DISK_MAX_COUNT,
    Math.max(DATA_DISK_MIN_COUNT, Math.trunc(parsed))
  );
  if (sanitized !== value) {
    dataDiskCount.value = sanitized;
  }
});

watch(region, async () => {
  stopPolling();
  deleteMsg.value = null;
  pendingStartupTaskCreate.value = null;
  pendingCreatedServerPassword.value = null;
  autoUpdatePendingServerIds.value = [];
  autoUpdateRunningServerId.value = null;
  autoUpdateDoneServerIds.value = [];
  autoUpdateFailedServerIds.value = [];
  autoUpdateProgressByServer.value = {};
  autoUpdateSessionToServerId.clear();
  autoUpdateSessionLineBuffer.clear();

  const hadCache = await hydrateRegionCache();
  if (!hadCache) {
    await loadAll();
  }
});

watch([imageVisibility, imageType], () => {
  loadImages();
});

watch(filteredImages, (list) => {
  if (!list.length) {
    imageId.value = "";
    return;
  }
  if (!list.some((image) => image.id === imageId.value)) {
    imageId.value = list[0].id;
  }
});

watch(filteredFlavors, (list) => {
  if (!list.length) {
    flavorId.value = "";
    return;
  }
  if (!list.some((flavor) => flavor.id === flavorId.value)) {
    flavorId.value = list[0].id;
  }
});

watch(selectedVpc, async (nextVpc, previousVpc) => {
  if (nextVpc === previousVpc) {
    return;
  }
  const hadSubnetCache = await hydrateSubnetsCache(nextVpc);
  if (!hadSubnetCache && nextVpc) {
    subnets.value = [];
    selectedSubnet.value = "";
  }
});

watch(ecses, (servers) => {
  cleanupStartupTaskTracking(servers);
  queueStartupTaskCandidates(servers);
  void drainAutoUpdateQueue();

  const activeServerId = sshPanelServerId.value;
  if (!activeServerId) {
    return;
  }
  const exists = servers.some((server) => server.id === activeServerId);
  if (exists) {
    return;
  }

  if (sshSession.value?.serverId === activeServerId) {
    void disconnectActiveSsh({ silent: true });
  }
  sshPanelServerId.value = null;
  sshPanelOpen.value = false;
});

watch(eips, () => {
  queueStartupTaskCandidates(ecses.value);
  void drainAutoUpdateQueue();
});

watch(autoUpdateVmOnStartup, (enabled) => {
  if (store) {
    void store.set("autoUpdateVmOnStartup", enabled);
  }
});

watch(setupGuiRdpOnStartup, (enabled) => {
  if (store) {
    void store.set("setupGuiRdpOnStartup", enabled);
  }
});

watch(logPanelOpen, (open) => {
  if (open) {
    logsUnreadError.value = false;
  }
});

function mapPluginLogLevel(level: LogLevel): LogLevelName {
  switch (level) {
    case LogLevel.Trace:
      return "trace";
    case LogLevel.Debug:
      return "debug";
    case LogLevel.Info:
      return "info";
    case LogLevel.Warn:
      return "warn";
    case LogLevel.Error:
      return "error";
    default:
      return "info";
  }
}

function addLog(source: LogSource, level: LogLevelName, message: string) {
  const next: AppLogEntry = {
    id: (logSeq += 1),
    at: new Date().toISOString(),
    source,
    level,
    message,
  };
  const list = [...logEntries.value, next];
  logEntries.value = list.slice(-MAX_LOG_ENTRIES);
  if (level === "error" && !logPanelOpen.value) {
    logsUnreadError.value = true;
  }
}

function setError(message: string) {
  errorMsg.value = message;
  addLog("app", "error", message);
}

function clearLogs() {
  logEntries.value = [];
  logsUnreadError.value = false;
}

async function showConfirmDialog(
  message: string,
  options: {
    title?: string;
    kind?: ConfirmDialogKind;
    okLabel?: string;
    cancelLabel?: string;
  }
) {
  const normalizedTitle = options.title?.trim() || "Confirm Action";
  const compactMessage = message.trim().replace(/\s+/g, " ");
  const titlePrefix = normalizedTitle.toLowerCase();
  const compactLower = compactMessage.toLowerCase();
  const dedupedMessage =
    compactLower.startsWith(titlePrefix)
      ? compactMessage.slice(normalizedTitle.length).trim().replace(/^[-:]/, "").trim()
      : compactMessage;
  const finalMessage = dedupedMessage || compactMessage || "Confirm this action?";
  if (resolveConfirmDialog) {
    resolveConfirmDialog(false);
    resolveConfirmDialog = null;
  }
  confirmDialog.value = {
    open: true,
    title: normalizedTitle,
    message: finalMessage,
    kind: options.kind ?? "warning",
    okLabel: options.okLabel?.trim() || "Confirm",
    cancelLabel: options.cancelLabel?.trim() || "Cancel",
  };
  return await new Promise<boolean>((resolve) => {
    resolveConfirmDialog = resolve;
  });
}

function closeConfirmDialog(confirmed: boolean) {
  const resolver = resolveConfirmDialog;
  resolveConfirmDialog = null;
  confirmDialog.value.open = false;
  if (resolver) {
    resolver(confirmed);
  }
}

async function initLogListeners() {
  try {
    backendLogUnlisten = await attachLogger((entry) => {
      addLog("backend", mapPluginLogLevel(entry.level), entry.message);
    });
    addLog("app", "info", "Attached backend log listener.");
  } catch (err) {
    addLog("app", "warn", `Failed to attach backend log listener: ${errorToString(err)}`);
  }

  try {
    sshOutputUnlisten = await listen<SshStreamEventPayload>("ssh-output", (event) => {
      const payload = event.payload;
      if (isAutoUpdateSessionId(payload.sessionId)) {
        trackAutoUpdateLineForSession(payload.sessionId, payload.kind, payload.text);
      }
      if (!sshSession.value || payload.sessionId !== sshSession.value.sessionId) {
        return;
      }

      const kind =
        payload.kind === "stderr"
          ? "stderr"
          : payload.kind === "stdout"
            ? "stdout"
            : "meta";
      addSshTerminalEntry(kind, payload.text, payload.at);
    });
    addLog("app", "info", "Attached SSH output listener.");
  } catch (err) {
    addLog("app", "warn", `Failed to attach SSH output listener: ${errorToString(err)}`);
  }

  errorListener = (event: ErrorEvent) => {
    addLog("runtime", "error", event.message || "Unhandled runtime error");
  };
  rejectionListener = (event: PromiseRejectionEvent) => {
    addLog("runtime", "error", `Unhandled promise rejection: ${String(event.reason)}`);
  };
  window.addEventListener("error", errorListener);
  window.addEventListener("unhandledrejection", rejectionListener);
}

onMounted(() => {
  relativeClockTimer = window.setInterval(() => {
    nowMs.value = Date.now();
  }, RELATIVE_TIME_TICK_MS);

  void initLogListeners();
  initStore();
});

onBeforeUnmount(() => {
  stopPolling();
  void disconnectActiveSsh({ silent: true });
  autoUpdateSessionToServerId.clear();
  autoUpdateSessionLineBuffer.clear();
  if (resolveConfirmDialog) {
    resolveConfirmDialog(false);
    resolveConfirmDialog = null;
  }

  if (passwordFeedbackTimer !== null) {
    window.clearTimeout(passwordFeedbackTimer);
    passwordFeedbackTimer = null;
  }

  if (relativeClockTimer !== null) {
    window.clearInterval(relativeClockTimer);
    relativeClockTimer = null;
  }
  if (sshResizeTimer !== null) {
    window.clearTimeout(sshResizeTimer);
    sshResizeTimer = null;
  }

  if (backendLogUnlisten) {
    backendLogUnlisten();
    backendLogUnlisten = null;
  }
  if (sshOutputUnlisten) {
    sshOutputUnlisten();
    sshOutputUnlisten = null;
  }
  if (errorListener) {
    window.removeEventListener("error", errorListener);
    errorListener = null;
  }
  if (rejectionListener) {
    window.removeEventListener("unhandledrejection", rejectionListener);
    rejectionListener = null;
  }
});

function randomInt(max: number): number {
  const bytes = new Uint32Array(1);
  window.crypto.getRandomValues(bytes);
  return bytes[0] % max;
}

function pickRandom(chars: string): string {
  return chars[randomInt(chars.length)];
}

function generatePassword(length = 18): string {
  const targetLength = Math.min(
    Math.max(length, PASSWORD_MIN_LENGTH),
    PASSWORD_MAX_LENGTH
  );
  const all = PASSWORD_UPPER + PASSWORD_LOWER + PASSWORD_DIGITS + PASSWORD_SYMBOLS;

  const chars: string[] = [
    pickRandom(PASSWORD_UPPER),
    pickRandom(PASSWORD_LOWER),
    pickRandom(PASSWORD_DIGITS),
    pickRandom(PASSWORD_SYMBOLS),
  ];

  while (chars.length < targetLength) {
    chars.push(pickRandom(all));
  }

  for (let index = chars.length - 1; index > 0; index -= 1) {
    const swapIndex = randomInt(index + 1);
    const current = chars[index];
    chars[index] = chars[swapIndex];
    chars[swapIndex] = current;
  }

  return chars.join("");
}

function validatePassword(password: string): string | null {
  if (!password) {
    return "Administrator password is required.";
  }
  if (password.length < PASSWORD_MIN_LENGTH || password.length > PASSWORD_MAX_LENGTH) {
    return `Password must be ${PASSWORD_MIN_LENGTH}-${PASSWORD_MAX_LENGTH} characters.`;
  }
  if (!/[A-Z]/.test(password)) {
    return "Password must include at least one uppercase letter.";
  }
  if (!/[a-z]/.test(password)) {
    return "Password must include at least one lowercase letter.";
  }
  if (!/[0-9]/.test(password)) {
    return "Password must include at least one number.";
  }

  const hasSymbol = [...password].some((char) => PASSWORD_SYMBOLS.includes(char));
  if (!hasSymbol) {
    return "Password must include at least one special character.";
  }

  return null;
}

function setPasswordFeedback(message: string | null) {
  passwordCopyFeedback.value = message;

  if (passwordFeedbackTimer !== null) {
    window.clearTimeout(passwordFeedbackTimer);
    passwordFeedbackTimer = null;
  }

  if (message) {
    passwordFeedbackTimer = window.setTimeout(() => {
      passwordCopyFeedback.value = null;
      passwordFeedbackTimer = null;
    }, 3000);
  }
}

function regeneratePassword() {
  generatedPassword.value = generatePassword();
  setPasswordFeedback(null);
}

async function copyCurrentPassword() {
  const password = selectedPassword.value;
  if (!password) {
    setPasswordFeedback("No password to copy.");
    addLog("app", "warn", "Copy password requested with empty password.");
    return;
  }

  try {
    await writeText(password);
    setPasswordFeedback("Password copied.");
    addLog("app", "info", "Copied password to clipboard via Tauri clipboard plugin.");
    return;
  } catch (err) {
    const message = errorToString(err);
    addLog(
      "app",
      "warn",
      `Clipboard plugin write failed: ${message}`
    );
    if (isTauriRuntime()) {
      setPasswordFeedback("Copy failed. Clipboard permission is unavailable.");
      return;
    }
  }

  if (navigator.clipboard?.writeText && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(password);
      setPasswordFeedback("Password copied.");
      addLog("app", "info", "Copied password to clipboard via browser clipboard API.");
      return;
    } catch (err) {
      addLog("app", "error", `Browser clipboard fallback failed: ${errorToString(err)}`);
    }
  }

  setPasswordFeedback("Copy failed. Clipboard permission is unavailable.");
  addLog("app", "error", "Failed to copy password to clipboard.");
}

function isTauriRuntime(): boolean {
  const runtime = window as Window & {
    __TAURI__?: unknown;
    __TAURI_INTERNALS__?: unknown;
  };
  return Boolean(runtime.__TAURI__ || runtime.__TAURI_INTERNALS__);
}

function errorToString(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }
  return String(err);
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const value of bytes) {
    binary += String.fromCharCode(value);
  }
  return window.btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  const binary = window.atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

function startupTasksEnabled(config: StartupTaskConfig | null | undefined): boolean {
  if (!config) {
    return false;
  }
  return config.autoUpdate || config.setupGuiRdp;
}

function startupTaskLabel(config: StartupTaskConfig | null | undefined): string {
  if (!config || !startupTasksEnabled(config)) {
    return "Startup tasks";
  }
  if (config.autoUpdate && config.setupGuiRdp) {
    return "OS update + Desktop/RDP setup";
  }
  if (config.autoUpdate) {
    return "OS update";
  }
  return "Desktop/RDP setup";
}

function startupTaskConfigForServer(serverId: string): StartupTaskConfig | null {
  if (!serverId) {
    return null;
  }
  return startupTaskConfigsByServer.value[serverId] ?? null;
}

function isSshAuthFailureMessage(message: string): boolean {
  return /authentication (failed|rejected)|permission denied|invalid credentials|password/i.test(
    message
  );
}

function buildStartupTaskCommand(config: StartupTaskConfig): string {
  const sections: string[] = ["set -eu"];
  if (config.autoUpdate) {
    sections.push(AUTO_VM_UPDATE_COMMAND);
  }
  if (config.setupGuiRdp) {
    sections.push(SETUP_GUI_RDP_COMMAND);
  }
  sections.push('echo "[hc-forge] Startup task pipeline completed."');
  return sections.join("\n\n");
}

async function ensureNotificationPermission(): Promise<boolean> {
  if (!isTauriRuntime()) {
    return false;
  }
  try {
    const granted = await isPermissionGranted();
    if (granted) {
      notificationPermissionChecked = true;
      return true;
    }
    if (notificationPermissionChecked) {
      return false;
    }
    const permission = await requestPermission();
    notificationPermissionChecked = true;
    return permission === "granted";
  } catch (err) {
    addLog("app", "warn", `Notification permission check failed: ${errorToString(err)}`);
    return false;
  }
}

async function sendUserNotification(title: string, body: string) {
  const granted = await ensureNotificationPermission();
  if (!granted) {
    return;
  }
  try {
    await sendNotification({ title, body });
  } catch (err) {
    addLog("app", "warn", `Failed to send desktop notification: ${errorToString(err)}`);
  }
}

function passwordEncryptionSecret(): string | null {
  const ak = accessKey.value.trim();
  const sk = secretKey.value.trim();
  if (!ak || !sk) {
    return null;
  }
  return `${ak}:${sk}:hc-forge`;
}

async function derivePasswordEncryptionKey(secret: string, salt: Uint8Array): Promise<CryptoKey> {
  const encoder = new TextEncoder();
  const material = await window.crypto.subtle.importKey(
    "raw",
    encoder.encode(secret),
    { name: "PBKDF2" },
    false,
    ["deriveKey"]
  );
  return await window.crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt,
      iterations: PBKDF2_ITERATIONS,
      hash: "SHA-256",
    },
    material,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"]
  );
}

async function encryptPasswordForStore(password: string): Promise<StoredServerPassword | null> {
  const secret = passwordEncryptionSecret();
  if (!secret || !window.crypto?.subtle || !password.trim()) {
    return null;
  }

  const encoder = new TextEncoder();
  const salt = window.crypto.getRandomValues(new Uint8Array(PBKDF2_SALT_BYTES));
  const iv = window.crypto.getRandomValues(new Uint8Array(AES_GCM_IV_BYTES));
  const key = await derivePasswordEncryptionKey(secret, salt);
  const ciphertext = await window.crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    encoder.encode(password)
  );

  return {
    version: 1,
    saltB64: bytesToBase64(salt),
    ivB64: bytesToBase64(iv),
    cipherB64: bytesToBase64(new Uint8Array(ciphertext)),
    updatedAt: new Date().toISOString(),
  };
}

async function decryptPasswordFromStore(
  payload: StoredServerPassword
): Promise<string | null> {
  const secret = passwordEncryptionSecret();
  if (!secret || !window.crypto?.subtle) {
    return null;
  }
  try {
    const key = await derivePasswordEncryptionKey(secret, base64ToBytes(payload.saltB64));
    const plaintext = await window.crypto.subtle.decrypt(
      { name: "AES-GCM", iv: base64ToBytes(payload.ivB64) },
      key,
      base64ToBytes(payload.cipherB64)
    );
    return new TextDecoder().decode(plaintext);
  } catch {
    return null;
  }
}

function decodeStoredServerPassword(value: unknown): StoredServerPassword | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const raw = value as Record<string, unknown>;
  if (raw.version !== 1) {
    return null;
  }
  if (
    typeof raw.saltB64 !== "string" ||
    typeof raw.ivB64 !== "string" ||
    typeof raw.cipherB64 !== "string"
  ) {
    return null;
  }
  return {
    version: 1,
    saltB64: raw.saltB64,
    ivB64: raw.ivB64,
    cipherB64: raw.cipherB64,
    updatedAt: typeof raw.updatedAt === "string" ? raw.updatedAt : new Date().toISOString(),
  };
}

function parseStoredPasswordMap(input: unknown): Record<string, StoredServerPassword> {
  if (!input || typeof input !== "object") {
    return {};
  }
  const parsed: Record<string, StoredServerPassword> = {};
  for (const [serverId, raw] of Object.entries(input as Record<string, unknown>)) {
    const decoded = decodeStoredServerPassword(raw);
    if (!decoded) {
      continue;
    }
    parsed[serverId] = decoded;
  }
  return parsed;
}

function decodeStartupTaskConfig(value: unknown): StartupTaskConfig | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const raw = value as Record<string, unknown>;
  const regionValue = typeof raw.region === "string" ? raw.region.trim() : "";
  if (!regionValue) {
    return null;
  }
  const autoUpdate = Boolean(raw.autoUpdate);
  const setupGuiRdp = Boolean(raw.setupGuiRdp);
  if (!autoUpdate && !setupGuiRdp) {
    return null;
  }
  const statusRaw =
    raw.lastStatus === "done" || raw.lastStatus === "failed" ? raw.lastStatus : "pending";

  return {
    region: regionValue,
    autoUpdate,
    setupGuiRdp,
    lastStatus: statusRaw,
    createdAt: typeof raw.createdAt === "string" ? raw.createdAt : new Date().toISOString(),
    updatedAt: typeof raw.updatedAt === "string" ? raw.updatedAt : new Date().toISOString(),
  };
}

function parseStartupTaskConfigMap(input: unknown): Record<string, StartupTaskConfig> {
  if (!input || typeof input !== "object") {
    return {};
  }
  const parsed: Record<string, StartupTaskConfig> = {};
  for (const [serverId, rawConfig] of Object.entries(input as Record<string, unknown>)) {
    const decoded = decodeStartupTaskConfig(rawConfig);
    if (!decoded) {
      continue;
    }
    parsed[serverId] = decoded;
  }
  return parsed;
}

function safeJsonParse(input: string): unknown | null {
  try {
    return JSON.parse(input);
  } catch {
    return null;
  }
}

function extractServerId(payload: unknown): string | null {
  if (!payload || typeof payload !== "object") {
    return null;
  }
  const data = payload as Record<string, unknown>;

  if (Array.isArray(data.server_ids) && data.server_ids.length > 0) {
    const id = data.server_ids[0];
    return typeof id === "string" ? id : null;
  }

  if (data.server && typeof data.server === "object") {
    const server = data.server as Record<string, unknown>;
    if (typeof server.id === "string") {
      return server.id;
    }
    if (typeof server.server_id === "string") {
      return server.server_id;
    }
  }

  if (typeof data.server_id === "string") {
    return data.server_id;
  }
  if (typeof data.id === "string") {
    return data.id;
  }

  return null;
}

function extractJobId(payload: unknown): string | null {
  if (!payload || typeof payload !== "object") {
    return null;
  }
  const data = payload as Record<string, unknown>;
  if (typeof data.job_id === "string") {
    return data.job_id;
  }
  if (typeof data.order_id === "string") {
    return data.order_id;
  }
  return null;
}

function flavorArchitecture(flavor: FlavorOption): string {
  return flavor.os_extra_specs?.["ecs:instance_architecture"] ?? "x86 (unspecified)";
}

function architectureSortRank(label: string): number {
  const normalized = label.toLowerCase();
  if (normalized === "x86 (unspecified)") {
    return 0;
  }
  if (normalized.includes("x86")) {
    return 1;
  }
  if (normalized.includes("arm")) {
    return 2;
  }
  return 3;
}

function matchesVcpuBucket(flavor: FlavorOption, bucket: string): boolean {
  if (bucket === "all") {
    return true;
  }

  const vcpus = flavor.vcpus ?? 0;

  if (bucket === "1-2") {
    return vcpus >= 1 && vcpus <= 2;
  }
  if (bucket === "4-8") {
    return vcpus >= 4 && vcpus <= 8;
  }
  if (bucket === "16+") {
    return vcpus >= 16;
  }

  return true;
}

function findEipForServer(serverId: string): EipRecord | null {
  return (
    eips.value.find(
      (eip) =>
        eip.associate_instance_id === serverId ||
        eip.vnic?.instance_id === serverId ||
        eip.vnic?.device_id === serverId
    ) ?? null
  );
}

function evsRole(volume: EvsVolume): "Boot" | "Data" {
  return volume.bootable ? "Boot" : "Data";
}

function evsAttachedServer(volume: EvsVolume): string {
  return volume.attachments?.[0]?.server_id ?? "Not attached";
}

function sshSessionIdForServer(serverId: string): string {
  return `ecs:${serverId}`;
}

function findSshHostForServer(ecs: EcsServer): string | null {
  const serverId = ecs.id ?? "";
  if (!serverId) {
    return null;
  }
  const eip = findEipForServer(serverId);
  const host = eip?.public_ip_address?.trim();
  return host || null;
}

function isSshConnectedForEcs(ecs: EcsServer): boolean {
  const serverId = ecs.id ?? "";
  if (!serverId || !sshSession.value) {
    return false;
  }
  return sshSession.value.serverId === serverId;
}

function canConnectSsh(ecs: EcsServer): boolean {
  return !!ecs.id && !!findSshHostForServer(ecs);
}

function isSshOpenForEcs(ecs: EcsServer): boolean {
  const serverId = ecs.id ?? "";
  return !!serverId && sshPanelOpen.value && sshPanelServerId.value === serverId;
}

function sshButtonLabel(ecs: EcsServer): string {
  const serverId = ecs.id ?? "";
  if (sshBusyServerId.value === serverId) {
    return isSshConnectedForEcs(ecs) ? "Disconnecting..." : "Connecting...";
  }
  return "SSH (root)";
}

function statusTone(status: string | null | undefined): string {
  const normalized = (status ?? "").toUpperCase();
  if (normalized === "ACTIVE" || normalized === "RUNNING") {
    return "status-ok";
  }
  if (normalized === "BUILD" || normalized === "REBOOT") {
    return "status-progress";
  }
  if (normalized === "ERROR") {
    return "status-error";
  }
  if (normalized === "SHUTOFF" || normalized === "STOPPED") {
    return "status-muted";
  }
  return "status-neutral";
}

function pushUniqueServerId(listRef: { value: string[] }, serverId: string) {
  if (!serverId || listRef.value.includes(serverId)) {
    return;
  }
  listRef.value = [...listRef.value, serverId];
}

function removeServerId(listRef: { value: string[] }, serverId: string) {
  if (!serverId || !listRef.value.includes(serverId)) {
    return;
  }
  listRef.value = listRef.value.filter((item) => item !== serverId);
}

function isAutoUpdateSessionId(sessionId: string): boolean {
  return sessionId.startsWith(AUTO_UPDATE_SESSION_PREFIX);
}

function newAutoUpdateProgressInfo(): AutoUpdateProgressInfo {
  return {
    sessionId: null,
    startedAt: null,
    finishedAt: null,
    percent: null,
    lastLine: null,
  };
}

function setAutoUpdateProgress(serverId: string, patch: Partial<AutoUpdateProgressInfo>) {
  if (!serverId) {
    return;
  }
  const current = autoUpdateProgressByServer.value[serverId] ?? newAutoUpdateProgressInfo();
  autoUpdateProgressByServer.value = {
    ...autoUpdateProgressByServer.value,
    [serverId]: {
      ...current,
      ...patch,
    },
  };
}

function clearAutoUpdateProgress(serverId: string) {
  if (!serverId || !(serverId in autoUpdateProgressByServer.value)) {
    return;
  }
  const next = { ...autoUpdateProgressByServer.value };
  delete next[serverId];
  autoUpdateProgressByServer.value = next;
}

function formatElapsedMs(diffMs: number): string {
  const totalSeconds = Math.max(0, Math.floor(diffMs / 1000));
  if (totalSeconds < 60) {
    return `${totalSeconds}s`;
  }
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (minutes < 60) {
    return `${minutes}m ${seconds}s`;
  }
  const hours = Math.floor(minutes / 60);
  const remMinutes = minutes % 60;
  return `${hours}h ${remMinutes}m`;
}

function autoUpdateProgressHint(serverId: string): string | null {
  if (!serverId) {
    return null;
  }
  const progress = autoUpdateProgressByServer.value[serverId];
  if (!progress) {
    return null;
  }
  const state = autoUpdateStatusForServer(serverId);
  const parts: string[] = [];
  if (state === "running" && progress.startedAt) {
    const startedAtMs = Date.parse(progress.startedAt);
    if (!Number.isNaN(startedAtMs)) {
      parts.push(`Running ${formatElapsedMs(nowMs.value - startedAtMs)}`);
    }
  }
  if (progress.percent != null) {
    parts.push(`Progress ${progress.percent}%`);
  }
  if (state !== "running" && progress.finishedAt) {
    parts.push(`Finished ${formatDateTime(progress.finishedAt)}`);
  }
  if (progress.lastLine) {
    parts.push(progress.lastLine);
  }
  return parts.length ? parts.join(" • ") : null;
}

function trackAutoUpdateLineForSession(sessionId: string, kind: string, text: string) {
  const serverId = autoUpdateSessionToServerId.get(sessionId);
  if (!serverId) {
    return;
  }
  const normalized = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  const pending = `${autoUpdateSessionLineBuffer.get(sessionId) ?? ""}${normalized}`;
  const lines = pending.split("\n");
  autoUpdateSessionLineBuffer.set(sessionId, lines.pop() ?? "");

  for (const rawLine of lines) {
    const line = sanitizeSshText(rawLine).trim();
    if (!line) {
      continue;
    }
    const progressMarker = line.match(/\[hc-forge\]\s*\[progress\]\s*(\d{1,3})/i);
    const percentMatch = progressMarker ?? line.match(/(\d{1,3})%/);
    const parsedPercent =
      percentMatch && kind !== "stderr"
        ? Math.min(100, Math.max(0, Number.parseInt(percentMatch[1], 10)))
        : null;
    const currentPercent = autoUpdateProgressByServer.value[serverId]?.percent;
    const nextPercent =
      parsedPercent == null ? null : Math.max(currentPercent ?? 0, parsedPercent);
    setAutoUpdateProgress(serverId, {
      lastLine: line.slice(0, 220),
      ...(nextPercent != null ? { percent: nextPercent } : {}),
    });
  }
}

function autoUpdateStatusForServer(serverId: string): "queued" | "running" | "done" | "failed" | "idle" {
  if (!serverId) {
    return "idle";
  }
  if (autoUpdateRunningServerId.value === serverId) {
    return "running";
  }
  if (autoUpdatePendingServerIds.value.includes(serverId)) {
    return "queued";
  }
  if (autoUpdateDoneServerIds.value.includes(serverId)) {
    return "done";
  }
  if (autoUpdateFailedServerIds.value.includes(serverId)) {
    return "failed";
  }
  const config = startupTaskConfigForServer(serverId);
  if (config?.lastStatus === "done") {
    return "done";
  }
  if (config?.lastStatus === "failed") {
    return "failed";
  }
  if (config?.lastStatus === "pending") {
    return "queued";
  }
  return "idle";
}

function autoUpdateStatusLabel(ecs: EcsServer): string {
  const serverId = ecs.id ?? "";
  const config = startupTaskConfigForServer(serverId);
  const taskLabel = startupTaskLabel(config);
  if (!startupTasksEnabled(config)) {
    return "Not configured";
  }
  const state = autoUpdateStatusForServer(serverId);
  const progress = autoUpdateProgressByServer.value[serverId];
  if (state === "running") {
    if (progress?.percent != null) {
      return `${taskLabel} running (${progress.percent}%)`;
    }
    return `${taskLabel} running...`;
  }
  if (state === "done") {
    return `${taskLabel} completed`;
  }
  if (state === "failed") {
    return `${taskLabel} failed`;
  }
  if ((ecs.status ?? "").toUpperCase() === "ACTIVE" && findSshHostForServer(ecs)) {
    return `${taskLabel} queued`;
  }
  return `${taskLabel} waiting for ACTIVE + EIP`;
}

function queueAutoUpdateForServer(serverId: string) {
  const config = startupTaskConfigForServer(serverId);
  if (!serverId || !config || !startupTasksEnabled(config) || config.lastStatus !== "pending") {
    return;
  }
  if (autoUpdateRunningServerId.value === serverId) {
    return;
  }
  if (
    autoUpdatePendingServerIds.value.includes(serverId) ||
    autoUpdateDoneServerIds.value.includes(serverId) ||
    autoUpdateFailedServerIds.value.includes(serverId)
  ) {
    return;
  }

  autoUpdatePendingServerIds.value = [...autoUpdatePendingServerIds.value, serverId];
  setAutoUpdateProgress(serverId, {
    percent: null,
    lastLine: `Queued: ${startupTaskLabel(config)}.`,
    finishedAt: null,
  });
}

function sanitizeSshText(text: string): string {
  let cleaned = text;

  cleaned = cleaned.replace(/\r\n/g, "\n").replace(/\r/g, "");
  cleaned = cleaned.replace(/\uFFFD\[[0-9;?]*[ -/]*[@-~]/g, "");
  cleaned = cleaned.replace(/\u001b\[[0-9;?]*[ -/]*[@-~]/g, "");
  cleaned = cleaned.replace(/\u001b\][^\u0007]*(\u0007|\u001b\\)/g, "");
  cleaned = cleaned.replace(/\u001b[PX^_].*?\u001b\\/gs, "");
  cleaned = cleaned.replace(/\u001b[\(\)][0-9A-Za-z]/g, "");
  cleaned = cleaned.replace(/\u001b./g, "");
  cleaned = cleaned.replace(/\uFFFD\[\?2004[hl]/g, "");
  cleaned = cleaned.replace(/\[\?2004[hl]/g, "");
  cleaned = cleaned.replace(/[^\x09\x0a\x20-\x7e\u00a0-\uffff]/g, "");
  cleaned = cleaned.replace(/\n{3,}/g, "\n\n");

  return cleaned;
}

function addSshTerminalEntry(
  kind: SshTerminalEntry["kind"],
  text: string,
  at?: string
) {
  const normalized = sanitizeSshText(text);
  if (!normalized) {
    return;
  }

  const next: SshTerminalEntry = {
    id: (sshTerminalSeq += 1),
    at: at && !Number.isNaN(Date.parse(at)) ? at : new Date().toISOString(),
    kind,
    text: normalized,
  };
  sshTerminalEntries.value = [...sshTerminalEntries.value, next].slice(-300);
}

function clearSshTerminal() {
  sshTerminalEntries.value = [];
}

function setActiveSshServer(ecs: EcsServer) {
  const nextServerId = ecs.id ?? null;
  if (!nextServerId) {
    return;
  }

  if (sshPanelServerId.value !== nextServerId) {
    clearSshTerminal();
    sshCommandInput.value = "";
    sshHistoryCursor.value = -1;
  }

  sshPanelServerId.value = nextServerId;
  sshPanelOpen.value = true;
}

function pushSshHistory(command: string) {
  const trimmed = command.trim();
  if (!trimmed) {
    return;
  }
  const deduped = sshCommandHistory.value.filter((item) => item !== trimmed);
  sshCommandHistory.value = [...deduped.slice(-49), trimmed];
  sshHistoryCursor.value = -1;
}

function browseSshHistory(direction: -1 | 1) {
  const history = sshCommandHistory.value;
  if (!history.length) {
    return;
  }

  if (direction === -1) {
    sshHistoryCursor.value = Math.min(sshHistoryCursor.value + 1, history.length - 1);
  } else {
    sshHistoryCursor.value = Math.max(sshHistoryCursor.value - 1, -1);
  }

  if (sshHistoryCursor.value === -1) {
    sshCommandInput.value = "";
    return;
  }

  const index = history.length - 1 - sshHistoryCursor.value;
  sshCommandInput.value = history[index] ?? "";
}

function handleSshCommandKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    void runSshCommand();
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    browseSshHistory(-1);
    return;
  }

  if (event.key === "ArrowDown") {
    event.preventDefault();
    browseSshHistory(1);
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "l") {
    event.preventDefault();
    clearSshTerminal();
  }
}

async function sendSshControlShortcut(control: "ctrl+c" | "ctrl+d" | "ctrl+u") {
  if (!sshSession.value) {
    addSshTerminalEntry("stderr", "Connect SSH before sending control shortcuts.");
    return;
  }

  const session = sshSession.value;
  try {
    await invoke<SshSendControlResult>("ssh_send_control", {
      params: {
        sessionId: session.sessionId,
        control,
      },
    });
    addLog("app", "info", `Sent ${control.toUpperCase()} to ${session.host}.`);
  } catch (err) {
    const message = errorToString(err);
    addSshTerminalEntry("stderr", `${control.toUpperCase()} failed: ${message}`);
    addLog("app", "warn", `Failed to send ${control.toUpperCase()}: ${message}`);
  }
}

function handleSshTerminalResize(size: { cols: number; rows: number }) {
  const session = sshSession.value;
  if (!session) {
    return;
  }

  const cols = Math.max(40, Math.trunc(size.cols));
  const rows = Math.max(10, Math.trunc(size.rows));
  const previous = sshLastResize.value;
  if (previous && previous.cols === cols && previous.rows === rows) {
    return;
  }

  sshLastResize.value = { cols, rows };
  if (sshResizeTimer !== null) {
    window.clearTimeout(sshResizeTimer);
  }
  sshResizeTimer = window.setTimeout(async () => {
    sshResizeTimer = null;
    const activeSession = sshSession.value;
    if (!activeSession || activeSession.sessionId !== session.sessionId) {
      return;
    }
    try {
      await invoke<SshResizeResult>("ssh_resize", {
        params: {
          sessionId: activeSession.sessionId,
          cols,
          rows,
          pixelWidth: 0,
          pixelHeight: 0,
        },
      });
    } catch (err) {
      addLog("app", "warn", `Failed to resize SSH terminal: ${errorToString(err)}`);
    }
  }, 120);
}

async function disconnectActiveSsh(options: { silent?: boolean } = {}) {
  if (!sshSession.value) {
    return;
  }

  const session = sshSession.value;
  sshBusyServerId.value = session.serverId;

  try {
    await invoke<SshDisconnectResult>("ssh_disconnect", {
      params: { sessionId: session.sessionId },
    });

    addLog(
      "app",
      "info",
      `SSH disconnected from ${session.username}@${session.host}:${session.port}.`
    );
    if (!options.silent) {
      addSshTerminalEntry("meta", `Disconnected from ${session.host}.`);
    }
  } catch (err) {
    const message = errorToString(err);
    if (!options.silent) {
      addSshTerminalEntry("stderr", `Disconnect failed: ${message}`);
    }
    addLog("app", "warn", `SSH disconnect failed: ${message}`);
  } finally {
    if (sshSession.value?.sessionId === session.sessionId) {
      sshSession.value = null;
    }
    sshBusyServerId.value = null;
    sshRunningCommand.value = false;
    sshLastResize.value = null;
  }
}

async function connectSshForEcs(ecs: EcsServer) {
  const serverId = ecs.id ?? "";
  const label = ecs.name ?? serverId;
  if (!serverId) {
    addLog("app", "error", "Missing ECS ID for SSH connection.");
    return;
  }

  const host = findSshHostForServer(ecs);
  if (!host) {
    addSshTerminalEntry("stderr", `No public EIP found for "${label}".`);
    addLog("app", "warn", `SSH unavailable for ${label}: no associated public EIP.`);
    return;
  }

  const password = resolveSshPasswordForServer(serverId);
  if (!password) {
    addSshTerminalEntry("stderr", "SSH password is required before connecting.");
    return;
  }

  if (sshSession.value && sshSession.value.serverId !== serverId) {
    await disconnectActiveSsh({ silent: true });
  }

  sshBusyServerId.value = serverId;
  addLog("app", "info", `Connecting SSH to root@${host} (${label}).`);

  try {
    const response = await invoke<SshConnectResult>("ssh_connect", {
      params: {
        sessionId: sshSessionIdForServer(serverId),
        host,
        port: 22,
        username: "root",
        password,
      },
    });

    sshSession.value = {
      ...response,
      serverId,
      serverName: label,
    };
    await persistServerPassword(serverId, password);
    addSshTerminalEntry("meta", `Connected to root@${host}.`);
    addLog("app", "info", `SSH connected to ${label} (${host}).`);
  } catch (err) {
    const message = errorToString(err);
    addSshTerminalEntry("stderr", `Connect failed: ${message}`);
    addLog("app", "error", `SSH connect failed for ${label}: ${message}`);
  } finally {
    sshBusyServerId.value = null;
  }
}

async function openAndConnectSsh(ecs: EcsServer) {
  if (!ecs.id) {
    return;
  }
  setActiveSshServer(ecs);

  if (isSshConnectedForEcs(ecs)) {
    return;
  }

  await connectSshForEcs(ecs);
}

async function closeSshPanel() {
  if (sshSession.value) {
    await disconnectActiveSsh({ silent: true });
  }
  sshPanelOpen.value = false;
  sshPanelServerId.value = null;
  sshRunningCommand.value = false;
  sshCommandInput.value = "";
  sshLastResize.value = null;
}

async function toggleSshForEcs(ecs: EcsServer) {
  const serverId = ecs.id ?? "";
  if (!serverId) {
    return;
  }

  if (isSshOpenForEcs(ecs)) {
    await closeSshPanel();
    return;
  }

  if (sshSession.value?.serverId === serverId && !sshPanelOpen.value) {
    setActiveSshServer(ecs);
    return;
  }

  await openAndConnectSsh(ecs);
}

async function reconnectSshForPanel() {
  if (!sshPanelServer.value) {
    addSshTerminalEntry("stderr", "Select an ECS instance first.");
    return;
  }

  const sameServerConnected =
    sshSession.value && sshSession.value.serverId === sshPanelServer.value.id;
  if (sameServerConnected) {
    await disconnectActiveSsh({ silent: true });
  }
  await connectSshForEcs(sshPanelServer.value);
}

async function runSshCommand() {
  if (!sshSession.value) {
    addSshTerminalEntry("stderr", "Connect SSH before running commands.");
    return;
  }
  const command = sshCommandInput.value.trim();
  if (!command) {
    return;
  }

  pushSshHistory(command);
  sshRunningCommand.value = true;

  try {
    await invoke<SshExecResult>("ssh_exec", {
      params: {
        sessionId: sshSession.value.sessionId,
        command,
      },
    });
    addLog(
      "app",
      "info",
      `SSH command completed on ${sshSession.value.host}: ${command}`
    );
    sshCommandInput.value = "";
  } catch (err) {
    const message = errorToString(err);
    addSshTerminalEntry("stderr", message);
    addLog("app", "error", `SSH command failed: ${message}`);
    sshSession.value = null;
  } finally {
    sshRunningCommand.value = false;
  }
}

async function runAutoUpdateForServer(server: EcsServer, host: string) {
  const serverId = server.id ?? "";
  if (!serverId) {
    return;
  }
  const config = startupTaskConfigForServer(serverId);
  if (!config || !startupTasksEnabled(config)) {
    return;
  }
  const password = resolveSshPasswordForServer(serverId);
  const label = server.name ?? serverId;
  if (!password) {
    throw new Error("SSH password is required for startup updates.");
  }

  const sessionId = `${AUTO_UPDATE_SESSION_PREFIX}${serverId}:${Date.now()}`;
  autoUpdateSessionToServerId.set(sessionId, serverId);
  autoUpdateSessionLineBuffer.delete(sessionId);
  setAutoUpdateProgress(serverId, {
    sessionId,
    startedAt: new Date().toISOString(),
    finishedAt: null,
    percent: 0,
    lastLine: `Connecting for ${startupTaskLabel(config)}...`,
  });

  const startupCommand = buildStartupTaskCommand(config);
  addLog("app", "info", `Running ${startupTaskLabel(config)} on ${label} (${host}).`);
  try {
    const response = await invoke<SshExecOneShotResult>("ssh_exec_one_shot", {
      params: {
        sessionId,
        host,
        port: 22,
        username: "root",
        password,
        command: startupCommand,
      },
    });

    const exitStatus = response.exitStatus ?? 0;
    const stderr = sanitizeSshText(response.stderr).trim();
    const stdout = sanitizeSshText(response.stdout).trim();
    if (exitStatus !== 0) {
      const summary = stderr || stdout || `exit status ${exitStatus}`;
      throw new Error(`${startupTaskLabel(config)} failed for ${label}: ${summary}`);
    }

    setAutoUpdateProgress(serverId, {
      percent: 100,
      finishedAt: new Date().toISOString(),
      lastLine: `${startupTaskLabel(config)} completed successfully.`,
    });
    addLog("app", "info", `${startupTaskLabel(config)} completed on ${label}.`);
    await persistServerPassword(serverId, password);
    if (stderr) {
      addLog("app", "warn", `${startupTaskLabel(config)} warnings on ${label}: ${stderr.slice(0, 240)}`);
    }
  } finally {
    const tail = sanitizeSshText(autoUpdateSessionLineBuffer.get(sessionId) ?? "").trim();
    if (tail) {
      setAutoUpdateProgress(serverId, {
        lastLine: tail.slice(0, 220),
      });
    }
    setAutoUpdateProgress(serverId, {
      sessionId: null,
    });
    autoUpdateSessionToServerId.delete(sessionId);
    autoUpdateSessionLineBuffer.delete(sessionId);
  }
}

async function drainAutoUpdateQueue() {
  if (autoUpdateDrainInFlight) {
    return;
  }

  autoUpdateDrainInFlight = true;
  let stalledCandidates = 0;
  try {
    while (true) {
      const queueSize = autoUpdatePendingServerIds.value.length;
      const nextServerId = autoUpdatePendingServerIds.value[0];
      if (!nextServerId) {
        break;
      }

      const config = startupTaskConfigForServer(nextServerId);
      if (!config || config.region !== region.value || config.lastStatus !== "pending") {
        removeServerId(autoUpdatePendingServerIds, nextServerId);
        continue;
      }

      const server = ecses.value.find((item) => item.id === nextServerId);
      if (!server) {
        removeServerId(autoUpdatePendingServerIds, nextServerId);
        continue;
      }
      if ((server.status ?? "").toUpperCase() !== "ACTIVE") {
        autoUpdatePendingServerIds.value = [
          ...autoUpdatePendingServerIds.value.slice(1),
          nextServerId,
        ];
        stalledCandidates += 1;
        if (stalledCandidates >= queueSize) {
          break;
        }
        continue;
      }

      const host = findSshHostForServer(server);
      if (!host) {
        autoUpdatePendingServerIds.value = [
          ...autoUpdatePendingServerIds.value.slice(1),
          nextServerId,
        ];
        stalledCandidates += 1;
        if (stalledCandidates >= queueSize) {
          break;
        }
        continue;
      }
      stalledCandidates = 0;

      removeServerId(autoUpdatePendingServerIds, nextServerId);
      autoUpdateRunningServerId.value = nextServerId;
      try {
        await runAutoUpdateForServer(server, host);
        await upsertStartupTaskConfig(nextServerId, { lastStatus: "done" });
        pushUniqueServerId(autoUpdateDoneServerIds, nextServerId);
        removeServerId(autoUpdateFailedServerIds, nextServerId);
        setAutoUpdateProgress(nextServerId, {
          finishedAt: new Date().toISOString(),
        });
        await sendUserNotification(
          "Startup tasks completed",
          `${server.name ?? nextServerId}: ${startupTaskLabel(config)} finished.`
        );
      } catch (err) {
        const rawMessage = errorToString(err);
        const authError = isSshAuthFailureMessage(rawMessage);
        const failureMessage = authError
          ? "Startup tasks failed: SSH authentication failed. Verify VM root password."
          : rawMessage;
        await upsertStartupTaskConfig(nextServerId, { lastStatus: "failed" });
        pushUniqueServerId(autoUpdateFailedServerIds, nextServerId);
        removeServerId(autoUpdateDoneServerIds, nextServerId);
        setAutoUpdateProgress(nextServerId, {
          finishedAt: new Date().toISOString(),
          lastLine: failureMessage.slice(0, 220),
        });
        addLog("app", authError ? "error" : "warn", failureMessage);
        await sendUserNotification(
          "Startup tasks failed",
          `${server.name ?? nextServerId}: ${failureMessage}`
        );
      } finally {
        autoUpdateRunningServerId.value = null;
      }
    }
  } finally {
    autoUpdateDrainInFlight = false;
  }
}

function formatFlavorLabel(flavor: FlavorOption): string {
  const vcpus = flavor.vcpus ?? "?";
  const ram = flavor.ram ?? "?";
  const arch = flavorArchitecture(flavor);
  return `${flavor.name} (${vcpus} vCPU, ${ram} MB, ${arch})`;
}

function pickLatestServer(servers: EcsServer[]): EcsServer | null {
  let latest: EcsServer | null = null;
  let latestCreated = "";

  for (const server of servers) {
    if (!server.created) {
      continue;
    }
    if (!latest || server.created > latestCreated) {
      latest = server;
      latestCreated = server.created;
    }
  }

  return latest ?? servers[0] ?? null;
}

function formatRelativeTimestamp(updatedAt: string | null): string {
  if (!updatedAt) {
    return "not cached yet";
  }

  const parsed = Date.parse(updatedAt);
  if (Number.isNaN(parsed)) {
    return "unknown";
  }

  const diffMs = Math.max(0, nowMs.value - parsed);
  const diffSeconds = Math.floor(diffMs / 1000);

  if (diffSeconds < 60) {
    return `${diffSeconds}s ago`;
  }

  const diffMinutes = Math.floor(diffSeconds / 60);
  if (diffMinutes < 60) {
    return `${diffMinutes}m ago`;
  }

  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) {
    return `${diffHours}h ago`;
  }

  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays}d ago`;
}

function formatDateTime(value: string | null | undefined): string {
  if (!value) {
    return "—";
  }
  const parsed = Date.parse(value);
  if (Number.isNaN(parsed)) {
    return value;
  }
  return USER_DATE_FORMATTER.format(parsed);
}

function regionCacheKey(resource: Exclude<CachedResource, "subnets">): string {
  return `${CACHE_PREFIX}.${resource}.${region.value}`;
}

function subnetCacheKey(regionName: string, vpcId: string): string {
  return `${CACHE_PREFIX}.subnets.${regionName}.${vpcId}`;
}

function isCachedEntry<T>(value: unknown): value is CachedEntry<T> {
  if (!value || typeof value !== "object") {
    return false;
  }
  const data = value as Record<string, unknown>;
  return typeof data.updatedAt === "string" && "data" in data;
}

async function readCachedResource<T>(
  resource: CachedResource,
  key: string
): Promise<T | null> {
  if (!store) {
    return null;
  }

  const value = await store.get<unknown>(key);
  if (!isCachedEntry<T>(value)) {
    cacheUpdatedAt.value[resource] = null;
    return null;
  }

  cacheUpdatedAt.value[resource] = value.updatedAt;
  return value.data;
}

async function writeCachedResource<T>(
  resource: CachedResource,
  key: string,
  data: T
): Promise<void> {
  if (!store) {
    return;
  }

  const entry: CachedEntry<T> = {
    updatedAt: new Date().toISOString(),
    data,
  };

  await store.set(key, entry);
  cacheUpdatedAt.value[resource] = entry.updatedAt;
}

async function persistStartupTaskConfigMap(): Promise<void> {
  if (!store) {
    return;
  }
  await store.set(STORE_KEY_STARTUP_TASKS_BY_SERVER, startupTaskConfigsByServer.value);
}

function syncStartupTaskResultsFromConfig() {
  const done: string[] = [];
  const failed: string[] = [];

  for (const [serverId, config] of Object.entries(startupTaskConfigsByServer.value)) {
    if (config.lastStatus === "done") {
      done.push(serverId);
      continue;
    }
    if (config.lastStatus === "failed") {
      failed.push(serverId);
    }
  }

  autoUpdateDoneServerIds.value = done;
  autoUpdateFailedServerIds.value = failed;
}

async function upsertStartupTaskConfig(
  serverId: string,
  patch: Partial<StartupTaskConfig>
): Promise<void> {
  if (!serverId) {
    return;
  }

  const current = startupTaskConfigsByServer.value[serverId];
  if (!current) {
    return;
  }

  startupTaskConfigsByServer.value = {
    ...startupTaskConfigsByServer.value,
    [serverId]: {
      ...current,
      ...patch,
      updatedAt: new Date().toISOString(),
    },
  };
  syncStartupTaskResultsFromConfig();
  await persistStartupTaskConfigMap();
}

async function removeStartupTaskConfig(serverId: string): Promise<void> {
  if (!serverId || !(serverId in startupTaskConfigsByServer.value)) {
    return;
  }
  const next = { ...startupTaskConfigsByServer.value };
  delete next[serverId];
  startupTaskConfigsByServer.value = next;
  syncStartupTaskResultsFromConfig();
  await persistStartupTaskConfigMap();
}

function serverPasswordFor(serverId: string): string | null {
  if (!serverId) {
    return null;
  }
  return serverPasswords.value[serverId] ?? null;
}

function resolveSshPasswordForServer(serverId: string): string {
  if (!sshUseFormPassword.value) {
    return sshManualPassword.value.trim();
  }
  return serverPasswordFor(serverId) ?? selectedPassword.value;
}

async function persistServerPassword(serverId: string, password: string): Promise<void> {
  const trimmed = password.trim();
  if (!serverId || !trimmed) {
    return;
  }

  serverPasswords.value = {
    ...serverPasswords.value,
    [serverId]: trimmed,
  };

  const encrypted = await encryptPasswordForStore(trimmed);
  if (!encrypted || !store) {
    return;
  }

  encryptedServerPasswords.value = {
    ...encryptedServerPasswords.value,
    [serverId]: encrypted,
  };
  await store.set(STORE_KEY_PASSWORDS_BY_SERVER, encryptedServerPasswords.value);
}

async function removeServerPassword(serverId: string): Promise<void> {
  if (!serverId) {
    return;
  }
  if (serverId in serverPasswords.value) {
    const nextPlain = { ...serverPasswords.value };
    delete nextPlain[serverId];
    serverPasswords.value = nextPlain;
  }
  if (serverId in encryptedServerPasswords.value) {
    const nextEncrypted = { ...encryptedServerPasswords.value };
    delete nextEncrypted[serverId];
    encryptedServerPasswords.value = nextEncrypted;
    if (store) {
      await store.set(STORE_KEY_PASSWORDS_BY_SERVER, encryptedServerPasswords.value);
    }
  }
}

async function hydrateServerPasswordsFromStore() {
  if (!store) {
    return;
  }

  const parsed = parseStoredPasswordMap(await store.get(STORE_KEY_PASSWORDS_BY_SERVER));
  encryptedServerPasswords.value = parsed;

  const decrypted: Record<string, string> = {};
  let recovered = 0;
  for (const [serverId, payload] of Object.entries(parsed)) {
    const value = await decryptPasswordFromStore(payload);
    if (!value) {
      continue;
    }
    decrypted[serverId] = value;
    recovered += 1;
  }
  serverPasswords.value = decrypted;
  addLog("app", "info", `Recovered ${recovered} encrypted VM password(s) from local store.`);
}

async function hydrateStartupTaskConfigsFromStore() {
  if (!store) {
    return;
  }
  startupTaskConfigsByServer.value = parseStartupTaskConfigMap(
    await store.get(STORE_KEY_STARTUP_TASKS_BY_SERVER)
  );
  syncStartupTaskResultsFromConfig();
}

async function registerStartupTasksForServer(
  serverId: string,
  config: StartupTaskConfig,
  password: string
) {
  if (!serverId || !startupTasksEnabled(config)) {
    return;
  }

  await persistServerPassword(serverId, password);
  startupTaskConfigsByServer.value = {
    ...startupTaskConfigsByServer.value,
    [serverId]: {
      ...config,
      region: config.region || region.value,
      lastStatus: "pending",
      updatedAt: new Date().toISOString(),
    },
  };
  removeServerId(autoUpdateDoneServerIds, serverId);
  removeServerId(autoUpdateFailedServerIds, serverId);
  clearAutoUpdateProgress(serverId);
  syncStartupTaskResultsFromConfig();
  await persistStartupTaskConfigMap();

  queueAutoUpdateForServer(serverId);
  void drainAutoUpdateQueue();
}

function queueStartupTaskCandidates(servers: EcsServer[]) {
  if (!servers.length) {
    return;
  }

  for (const server of servers) {
    const serverId = server.id ?? "";
    if (!serverId) {
      continue;
    }
    const config = startupTaskConfigForServer(serverId);
    if (!config || config.region !== region.value || config.lastStatus !== "pending") {
      continue;
    }
    if ((server.status ?? "").toUpperCase() !== "ACTIVE") {
      continue;
    }
    queueAutoUpdateForServer(serverId);
  }
}

function cleanupStartupTaskTracking(servers: EcsServer[]) {
  const present = new Set<string>();
  for (const server of servers) {
    if (server.id) {
      present.add(server.id);
    }
  }

  for (const trackedId of Object.keys(startupTaskConfigsByServer.value)) {
    const config = startupTaskConfigsByServer.value[trackedId];
    if (!config || config.region !== region.value || present.has(trackedId)) {
      continue;
    }
    removeServerId(autoUpdatePendingServerIds, trackedId);
    removeServerId(autoUpdateDoneServerIds, trackedId);
    removeServerId(autoUpdateFailedServerIds, trackedId);
    clearAutoUpdateProgress(trackedId);
    for (const [sessionId, sessionServerId] of Array.from(autoUpdateSessionToServerId.entries())) {
      if (sessionServerId !== trackedId) {
        continue;
      }
      autoUpdateSessionToServerId.delete(sessionId);
      autoUpdateSessionLineBuffer.delete(sessionId);
    }
    if (autoUpdateRunningServerId.value === trackedId) {
      autoUpdateRunningServerId.value = null;
    }
    void removeStartupTaskConfig(trackedId);
  }
}

function pickPreferredImageId(data: ImageOption[]): string | null {
  const normalizedQuery = DEFAULT_IMAGE_QUERY.toLowerCase();
  const exact = data.find((image) => {
    const text = `${image.name} ${image.id}`.toLowerCase();
    return text.includes(normalizedQuery);
  });
  if (exact) {
    return exact.id;
  }

  const ubuntu24 = data.find((image) => {
    const text = `${image.name} ${image.id}`.toLowerCase();
    return text.includes("ubuntu") && (text.includes("24") || text.includes("24.04"));
  });
  if (ubuntu24) {
    return ubuntu24.id;
  }

  const ubuntu = data.find((image) => `${image.name} ${image.id}`.toLowerCase().includes("ubuntu"));
  return ubuntu?.id ?? null;
}

function pickPreferredFlavorId(data: FlavorOption[]): string | null {
  const normalizedQuery = DEFAULT_FLAVOR_QUERY.toLowerCase();
  const exact = data.find((flavor) => {
    const text = `${flavor.name} ${flavor.id}`.toLowerCase();
    return text.includes(normalizedQuery);
  });
  if (exact) {
    return exact.id;
  }

  const byShape = data.find((flavor) => {
    const text = `${flavor.name} ${flavor.id}`.toLowerCase();
    return (flavor.vcpus ?? 0) === 4 && (flavor.ram ?? 0) === 8192 && text.includes("x1");
  });
  return byShape?.id ?? null;
}

function applyImageSelection(data: ImageOption[]) {
  if (data.find((image) => image.id === imageId.value)) {
    return;
  }
  imageId.value = pickPreferredImageId(data) ?? data[0]?.id ?? "";
}

function applyFlavorSelection(data: FlavorOption[]) {
  if (!data.find((flavor) => flavor.id === flavorId.value)) {
    flavorId.value = pickPreferredFlavorId(data) ?? data[0]?.id ?? "";
  }

  if (
    flavorArchFilter.value !== "all" &&
    !data.some((flavor) => flavorArchitecture(flavor) === flavorArchFilter.value)
  ) {
    flavorArchFilter.value = "all";
  }
}

function applyVpcSelection(data: VpcOption[]) {
  if (!data.find((vpc) => vpc.id === selectedVpc.value)) {
    selectedVpc.value = data[0]?.id ?? "";
  }
}

function applySubnetSelection(data: SubnetOption[]) {
  if (!data.find((subnet) => subnet.id === selectedSubnet.value)) {
    selectedSubnet.value = data[0]?.id ?? "";
  }
}

async function hydrateSubnetsCache(vpcId: string): Promise<boolean> {
  if (!vpcId) {
    subnets.value = [];
    selectedSubnet.value = "";
    cacheUpdatedAt.value.subnets = null;
    return false;
  }

  const key = subnetCacheKey(region.value, vpcId);
  const cached = await readCachedResource<SubnetOption[]>("subnets", key);
  if (!cached) {
    subnets.value = [];
    selectedSubnet.value = "";
    return false;
  }

  subnets.value = cached;
  applySubnetSelection(cached);
  addLog("app", "info", `Loaded ${cached.length} cached subnets for VPC ${vpcId}.`);
  return true;
}

async function hydrateRegionCache(): Promise<boolean> {
  let hasAnyCache = false;

  const cachedImages = await readCachedResource<ImageOption[]>(
    "images",
    regionCacheKey("images")
  );
  if (cachedImages) {
    images.value = cachedImages;
    applyImageSelection(cachedImages);
    hasAnyCache = true;
  } else {
    images.value = [];
    imageId.value = "";
  }

  const cachedFlavors = await readCachedResource<FlavorOption[]>(
    "flavors",
    regionCacheKey("flavors")
  );
  if (cachedFlavors) {
    flavors.value = cachedFlavors;
    applyFlavorSelection(cachedFlavors);
    hasAnyCache = true;
  } else {
    flavors.value = [];
    flavorId.value = "";
  }

  const cachedVpcs = await readCachedResource<VpcOption[]>(
    "vpcs",
    regionCacheKey("vpcs")
  );
  if (cachedVpcs) {
    vpcs.value = cachedVpcs;
    applyVpcSelection(cachedVpcs);
    hasAnyCache = true;
  } else {
    vpcs.value = [];
    selectedVpc.value = "";
  }

  await hydrateSubnetsCache(selectedVpc.value);

  const cachedEips = await readCachedResource<EipRecord[]>(
    "eips",
    regionCacheKey("eips")
  );
  if (cachedEips) {
    eips.value = cachedEips;
    hasAnyCache = true;
  } else {
    eips.value = [];
  }

  const cachedEvss = await readCachedResource<EvsVolume[]>(
    "evss",
    regionCacheKey("evss")
  );
  if (cachedEvss) {
    evss.value = cachedEvss;
    hasAnyCache = true;
  } else {
    evss.value = [];
  }

  const cachedEcses = await readCachedResource<EcsServer[]>(
    "ecses",
    regionCacheKey("ecses")
  );
  if (cachedEcses) {
    ecses.value = cachedEcses;
    hasAnyCache = true;
  } else {
    ecses.value = [];
  }

  if (hasAnyCache) {
    addLog("app", "info", `Loaded cached resources for region ${region.value}.`);
  }

  return hasAnyCache;
}

function stopPolling() {
  if (pollingTimer !== null) {
    window.clearTimeout(pollingTimer);
    pollingTimer = null;
  }
  pollingEcs.value = false;
  pollingActiveRefreshDone.value = false;
}

function startPolling(serverId: string | null) {
  stopPolling();
  pollingEcs.value = true;
  pollingAttempts.value = 0;
  pollingStatus.value = null;
  pollingError.value = null;
  pollingActiveRefreshDone.value = false;
  const watchEips = allocateEip.value;

  const tick = async () => {
    if (!pollingEcs.value) {
      return;
    }

    pollingAttempts.value += 1;

    try {
      const shouldRefreshLists = pollingAttempts.value <= 10;
      if (shouldRefreshLists) {
        const tasks: Promise<void>[] = [loadEcses({ log: false }), loadEvss({ log: false })];
        if (watchEips) {
          tasks.push(loadEips({ log: false }));
        }
        await Promise.all(tasks);
      }

      const server = await refreshCreatedInstance(serverId, {
        withEips: watchEips,
        skipReload: shouldRefreshLists,
      });
      pollingStatus.value = server?.status ?? null;

      if (!server) {
        pollingError.value = "No ECS instances found to watch yet.";
      } else {
        pollingError.value = null;
        if (pendingCreatedServerPassword.value && server.id) {
          await persistServerPassword(server.id, pendingCreatedServerPassword.value);
          pendingCreatedServerPassword.value = null;
        }
        if (pendingStartupTaskCreate.value && server.id) {
          const pending = pendingStartupTaskCreate.value;
          pendingStartupTaskCreate.value = null;
          await registerStartupTasksForServer(server.id, pending.config, pending.password);
        }
      }

      const status = (server?.status ?? "").toUpperCase();
      if (status === "ACTIVE" && !pollingActiveRefreshDone.value) {
        pollingActiveRefreshDone.value = true;
        if (pollingAttempts.value > 10) {
          await Promise.all([loadEcses({ log: false }), loadEips({ log: false }), loadEvss({ log: false })]);
        }
        addLog("app", "info", "ECS reached ACTIVE state. Refreshed ECS, EIP, and EVS lists.");
      }
      if (status === "ACTIVE" || status === "ERROR") {
        stopPolling();
        return;
      }
    } catch (err) {
      pollingError.value = errorToString(err);
    }

    if (pollingAttempts.value >= POLL_MAX_ATTEMPTS) {
      stopPolling();
      return;
    }

    pollingTimer = window.setTimeout(tick, POLL_INTERVAL_MS);
  };

  pollingTimer = window.setTimeout(tick, 1000);
}

function buildCredentialsPayload(): CredentialsPayload {
  const ak = accessKey.value.trim();
  const sk = secretKey.value.trim();

  if (!ak && !sk) {
    throw new Error("Access Key and Secret Key are required.");
  }

  if (!ak || !sk) {
    throw new Error("Provide both Access Key and Secret Key.");
  }

  return { accessKey: ak, secretKey: sk };
}

async function loadVpcs() {
  loadingVpcs.value = true;
  errorMsg.value = "";
  deleteMsg.value = null;
  result.value = null;
  addLog("app", "info", `Listing VPCs for region ${region.value}.`);

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<VpcOption[]>("list_vpcs", args);
    vpcs.value = data;
    applyVpcSelection(data);
    addLog("app", "info", `Loaded ${data.length} VPCs for region ${region.value}.`);

    await writeCachedResource("vpcs", regionCacheKey("vpcs"), data);
    await hydrateSubnetsCache(selectedVpc.value);
  } catch (err) {
    setError(`Failed to load VPCs: ${errorToString(err)}`);
  } finally {
    loadingVpcs.value = false;
  }
}

async function loadSubnets() {
  if (!selectedVpc.value) {
    setError("Select a VPC before loading subnets.");
    return;
  }

  loadingSubnets.value = true;
  errorMsg.value = "";
  deleteMsg.value = null;
  result.value = null;
  addLog("app", "info", `Listing subnets for VPC ${selectedVpc.value}.`);

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      region: region.value,
      vpcId: selectedVpc.value,
    };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<SubnetOption[]>("list_subnets", args);
    subnets.value = data;
    applySubnetSelection(data);
    addLog("app", "info", `Loaded ${data.length} subnets for VPC ${selectedVpc.value}.`);

    await writeCachedResource(
      "subnets",
      subnetCacheKey(region.value, selectedVpc.value),
      data
    );
  } catch (err) {
    setError(`Failed to load subnets: ${errorToString(err)}`);
  } finally {
    loadingSubnets.value = false;
  }
}

async function loadImages() {
  loadingImages.value = true;
  errorMsg.value = "";
  deleteMsg.value = null;
  result.value = null;
  addLog("app", "info", `Listing images for region ${region.value}.`);

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };
    const filters: Record<string, unknown> = {};

    if (credentials) {
      args.credentials = credentials;
    }

    if (imageVisibility.value !== "all") {
      filters.visibility = imageVisibility.value;
    }
    if (imageType.value !== "all") {
      filters.imageType = imageType.value;
    }
    if (flavorId.value) {
      filters.flavorId = flavorId.value;
    }
    if (Object.keys(filters).length > 0) {
      args.filters = filters;
    }

    const data = await invoke<ImageOption[]>("list_images", args);
    images.value = data;
    applyImageSelection(data);
    addLog("app", "info", `Loaded ${data.length} images for region ${region.value}.`);

    await writeCachedResource("images", regionCacheKey("images"), data);
  } catch (err) {
    setError(`Failed to load images: ${errorToString(err)}`);
  } finally {
    loadingImages.value = false;
  }
}

async function loadFlavors() {
  loadingFlavors.value = true;
  errorMsg.value = "";
  deleteMsg.value = null;
  result.value = null;
  addLog("app", "info", `Listing flavors for region ${region.value}.`);

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<FlavorOption[]>("list_flavors", args);
    flavors.value = data;
    applyFlavorSelection(data);
    addLog("app", "info", `Loaded ${data.length} flavors for region ${region.value}.`);

    await writeCachedResource("flavors", regionCacheKey("flavors"), data);
  } catch (err) {
    setError(`Failed to load flavors: ${errorToString(err)}`);
  } finally {
    loadingFlavors.value = false;
  }
}

async function loadEips(options: { log?: boolean } = {}) {
  const shouldLog = options.log ?? true;
  loadingEips.value = true;
  errorMsg.value = "";
  if (shouldLog) {
    addLog("app", "info", `Listing EIPs for region ${region.value}.`);
  }

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      region: region.value,
      params: { limit: 1000 },
    };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<EipListResponse>("list_eips", args);
    const publicips = data.publicips ?? [];
    eips.value = publicips;
    if (shouldLog) {
      addLog("app", "info", `Loaded ${publicips.length} EIPs for region ${region.value}.`);
    }

    await writeCachedResource("eips", regionCacheKey("eips"), publicips);
  } catch (err) {
    setError(`Failed to load EIPs: ${errorToString(err)}`);
  } finally {
    loadingEips.value = false;
  }
}

async function loadEvss(options: { log?: boolean } = {}) {
  const shouldLog = options.log ?? true;
  loadingEvss.value = true;
  errorMsg.value = "";
  if (shouldLog) {
    addLog("app", "info", `Listing EVS disks for region ${region.value}.`);
  }

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      region: region.value,
      params: { limit: 1000 },
    };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<EvsListResponse>("list_evss", args);
    const volumes = data.volumes ?? [];
    evss.value = volumes;
    if (shouldLog) {
      addLog("app", "info", `Loaded ${volumes.length} EVS disks for region ${region.value}.`);
    }

    await writeCachedResource("evss", regionCacheKey("evss"), volumes);
  } catch (err) {
    setError(`Failed to load EVS disks: ${errorToString(err)}`);
  } finally {
    loadingEvss.value = false;
  }
}

async function loadEcses(options: { log?: boolean } = {}) {
  const shouldLog = options.log ?? true;
  loadingEcses.value = true;
  errorMsg.value = "";
  if (shouldLog) {
    addLog("app", "info", `Listing ECS instances for region ${region.value}.`);
  }

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      region: region.value,
      params: { limit: 1000 },
    };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<EcsListResponse>("list_ecses", args);
    const servers = data.servers ?? [];
    ecses.value = servers;
    if (shouldLog) {
      addLog("app", "info", `Loaded ${servers.length} ECS instances for region ${region.value}.`);
    }

    await writeCachedResource("ecses", regionCacheKey("ecses"), servers);
  } catch (err) {
    setError(`Failed to load ECS instances: ${errorToString(err)}`);
  } finally {
    loadingEcses.value = false;
  }
}

async function refreshCreatedInstance(
  serverId: string | null,
  options: { withEips?: boolean; skipReload?: boolean } = {}
): Promise<EcsServer | null> {
  createdServer.value = null;
  if (options.withEips) {
    createdEip.value = null;
  }

  if (!serverId && !useCustomName.value) {
    if (ecses.value.length === 0) {
      return null;
    }
    const latest = pickLatestServer(ecses.value);
    if (latest) {
      createdServer.value = latest;
      if (options.withEips && latest.id) {
        createdEip.value = findEipForServer(latest.id);
      }
    }
    return latest;
  }

  if (!options.skipReload) {
    const tasks: Promise<void>[] = [loadEcses({ log: false })];
    if (options.withEips) {
      tasks.push(loadEips({ log: false }));
    }
    await Promise.all(tasks);
  }

  let server: EcsServer | undefined;
  if (serverId) {
    server = ecses.value.find((item) => item.id === serverId);
  }
  if (!server && useCustomName.value) {
    const customName = name.value.trim();
    if (customName) {
      server = ecses.value.find((item) => item.name === customName);
    }
  }

  if (server) {
    createdServer.value = server;
    if (options.withEips && server.id) {
      createdEip.value = findEipForServer(server.id);
    }
  }

  return server ?? null;
}

async function loadAll() {
  if (loadingAll.value) {
    return;
  }

  loadingAll.value = true;
  addLog("app", "info", `Reloading all resources for region ${region.value}.`);
  try {
    await Promise.all([loadImages(), loadFlavors(), loadVpcs(), loadEips(), loadEvss(), loadEcses()]);
    if (selectedVpc.value) {
      await loadSubnets();
    }
    addLog("app", "info", `Finished reloading all resources for region ${region.value}.`);
  } finally {
    loadingAll.value = false;
  }
}

async function saveCredentials() {
  if (!store) {
    setError("Credential store is not ready yet.");
    return;
  }

  savingCredentials.value = true;
  try {
    await store.set("accessKey", accessKey.value);
    await store.set("secretKey", secretKey.value);
    await hydrateServerPasswordsFromStore();
    addLog("app", "info", "Saved API credentials and refreshed encrypted VM passwords.");
  } finally {
    savingCredentials.value = false;
  }
}

async function initStore() {
  let hadCache = false;

  try {
    store = await load("store.json", { autoSave: true, defaults: {} });
    accessKey.value = (await store.get<string>("accessKey")) ?? "";
    secretKey.value = (await store.get<string>("secretKey")) ?? "";
    autoUpdateVmOnStartup.value = (await store.get<boolean>("autoUpdateVmOnStartup")) ?? false;
    setupGuiRdpOnStartup.value = (await store.get<boolean>("setupGuiRdpOnStartup")) ?? false;
    storeReady.value = true;
    addLog("app", "info", "Initialized local credential/cache store.");
    await hydrateServerPasswordsFromStore();
    await hydrateStartupTaskConfigsFromStore();
    hadCache = await hydrateRegionCache();
    queueStartupTaskCandidates(ecses.value);
    void drainAutoUpdateQueue();
  } catch (err) {
    setError(`Failed to load credential store: ${errorToString(err)}`);
  }

  if (!hadCache) {
    await loadAll();
  }
}

function summarizeDeleteResult(response: DeleteEcsResult): string {
  const ecsCode = response.ecs.status_code ?? 0;
  let summary = `ECS delete: ${ecsCode} ${response.ecs.status}`;

  if (response.eip) {
    const eipCode = response.eip.status_code ?? "n/a";
    summary += ` | EIP delete: ${eipCode} ${response.eip.status}`;
  }

  return summary;
}

function summarizeStopResult(response: StopEcsResult): string {
  const code = response.ecs.status_code ?? 0;
  return `ECS stop: ${code} ${response.ecs.status}`;
}

function canStopEcs(ecs: EcsServer): boolean {
  const status = (ecs.status ?? "").toUpperCase();
  if (!ecs.id) {
    return false;
  }
  return status !== "SHUTOFF" && status !== "STOPPED";
}

async function deleteEcs(ecs: EcsServer) {
  const serverId = ecs.id ?? "";
  if (!serverId) {
    return;
  }

  const linkedEip = findEipForServer(serverId);
  const label = ecs.name ?? serverId;
  const confirmed = await showConfirmDialog(
    `Do you want to delete "${label}"? This also requests deletion of the attached EIP.`,
    {
      title: "Delete ECS",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    }
  );
  if (!confirmed) {
    return;
  }

  deletingServerId.value = serverId;
  errorMsg.value = "";
  deleteMsg.value = null;
  addLog("app", "info", `Deleting ECS ${label}.`);

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        serverId,
        eipId: linkedEip?.id ?? null,
        deleteVolume: true,
      },
    };

    if (credentials) {
      args.credentials = credentials;
    }

    const response = await invoke<DeleteEcsResult>("delete_ecs_with_eip", args);
    deleteMsg.value = summarizeDeleteResult(response);
    addLog("app", "info", deleteMsg.value);

    if (createdServer.value?.id === serverId) {
      createdServer.value = null;
      createdEip.value = null;
      stopPolling();
    }
    pendingStartupTaskCreate.value = null;
    removeServerId(autoUpdatePendingServerIds, serverId);
    removeServerId(autoUpdateDoneServerIds, serverId);
    removeServerId(autoUpdateFailedServerIds, serverId);
    clearAutoUpdateProgress(serverId);
    for (const [sessionId, sessionServerId] of Array.from(autoUpdateSessionToServerId.entries())) {
      if (sessionServerId !== serverId) {
        continue;
      }
      autoUpdateSessionToServerId.delete(sessionId);
      autoUpdateSessionLineBuffer.delete(sessionId);
    }
    if (autoUpdateRunningServerId.value === serverId) {
      autoUpdateRunningServerId.value = null;
    }
    if (sshSession.value?.serverId === serverId) {
      await disconnectActiveSsh({ silent: true });
    }
    await removeStartupTaskConfig(serverId);
    await removeServerPassword(serverId);

    await Promise.all([loadEcses(), loadEips(), loadEvss()]);
  } catch (err) {
    setError(`Delete failed: ${errorToString(err)}`);
  } finally {
    deletingServerId.value = null;
  }
}

async function stopEcs(ecs: EcsServer) {
  const serverId = ecs.id ?? "";
  if (!serverId || !canStopEcs(ecs)) {
    return;
  }

  const label = ecs.name ?? serverId;
  const confirmed = await showConfirmDialog(
    `Do you want to stop "${label}" now? This performs a SOFT stop via the ECS API.`,
    {
      title: "Stop ECS",
      kind: "warning",
      okLabel: "Stop",
      cancelLabel: "Cancel",
    }
  );
  if (!confirmed) {
    return;
  }

  stoppingServerId.value = serverId;
  errorMsg.value = "";
  deleteMsg.value = null;
  addLog("app", "info", `Stopping ECS ${label}.`);

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        serverId,
        stopType: "SOFT",
      },
    };

    if (credentials) {
      args.credentials = credentials;
    }

    const response = await invoke<StopEcsResult>("stop_ecs", args);
    deleteMsg.value = summarizeStopResult(response);
    addLog("app", "info", deleteMsg.value);
    await loadEcses();
  } catch (err) {
    setError(`Stop failed: ${errorToString(err)}`);
  } finally {
    stoppingServerId.value = null;
  }
}

async function createEcs() {
  if (!imageId.value || !flavorId.value) {
    setError("Image ID and Flavor ID are required.");
    return;
  }

  if (!selectedVpc.value || !selectedSubnet.value) {
    setError("Select a VPC and subnet before creating the server.");
    return;
  }

  if (passwordError.value) {
    setError(passwordError.value);
    return;
  }

  if (
    allocateEip.value &&
    (eipBandwidthSize.value < EIP_BANDWIDTH_MIN_MBIT ||
      eipBandwidthSize.value > EIP_BANDWIDTH_MAX_MBIT)
  ) {
    setError(
      `EIP bandwidth must be ${EIP_BANDWIDTH_MIN_MBIT}-${EIP_BANDWIDTH_MAX_MBIT} Mbit/s.`
    );
    return;
  }

  if (
    includeDataDisk.value &&
    (dataDiskSize.value < DATA_DISK_MIN_GB || dataDiskSize.value > DATA_DISK_MAX_GB)
  ) {
    setError(`Data disk size must be ${DATA_DISK_MIN_GB}-${DATA_DISK_MAX_GB} GB.`);
    return;
  }

  if (
    includeDataDisk.value &&
    (dataDiskCount.value < DATA_DISK_MIN_COUNT || dataDiskCount.value > DATA_DISK_MAX_COUNT)
  ) {
    setError(`Data disk count must be ${DATA_DISK_MIN_COUNT}-${DATA_DISK_MAX_COUNT}.`);
    return;
  }

  const passwordForNewServer = selectedPassword.value;
  const startupConfigForCreate: StartupTaskConfig = {
    region: region.value,
    autoUpdate: autoUpdateVmOnStartup.value,
    setupGuiRdp: setupGuiRdpOnStartup.value,
    lastStatus: "pending",
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
  const shouldScheduleStartupTasks = startupTasksEnabled(startupConfigForCreate);

  creating.value = true;
  errorMsg.value = "";
  deleteMsg.value = null;
  result.value = null;
  createSummary.value = null;
  createdServer.value = null;
  createdEip.value = null;
  pendingStartupTaskCreate.value = null;
  pendingCreatedServerPassword.value = null;
  stopPolling();

  try {
    const credentials = buildCredentialsPayload();
    const payload = {
      name: useCustomName.value ? name.value.trim() : "",
      imageId: imageId.value,
      flavorId: flavorId.value,
      region: region.value,
      vpcId: selectedVpc.value,
      subnetId: selectedSubnet.value,
      rootVolumeType: rootVolumeType.value,
      rootVolumeSize: rootVolumeSize.value,
      eip: allocateEip.value,
      eipBandwidthSize: eipBandwidthSize.value,
      adminPassword: passwordForNewServer,
      dataVolumes: includeDataDisk.value
        ? [
            {
              volumeType: dataDiskType.value,
              size: dataDiskSize.value,
              count: dataDiskCount.value,
              multiattach: dataDiskMultiattach.value,
              hwPassthrough: dataDiskHwPassthrough.value,
            },
          ]
        : [],
    };

    const args: Record<string, unknown> = { params: payload };

    if (credentials) {
      args.credentials = credentials;
    }

    result.value = await invoke<CreateEcsResult>("create_ecs", args);
    const responsePayload = safeJsonParse(result.value.body);
    const serverId = extractServerId(responsePayload);
    const jobId = extractJobId(responsePayload);
    const errorMessage =
      responsePayload && typeof responsePayload === "object"
        ? (
            (responsePayload as Record<string, unknown>).error as
              | Record<string, unknown>
              | undefined
          )?.message
        : null;

    createSummary.value = {
      status: result.value.status,
      statusCode: result.value.status_code,
      serverId,
      jobId,
      message: typeof errorMessage === "string" ? errorMessage : null,
    };

    const isSuccess =
      result.value.status_code >= 200 && result.value.status_code < 300;
    addLog(
      "app",
      isSuccess ? "info" : "warn",
      `Create ECS response: ${result.value.status_code} ${result.value.status}`
    );

    if (isSuccess) {
      const refreshedServer = await refreshCreatedInstance(serverId, {
        withEips: allocateEip.value,
      });
      const watchId = serverId ?? refreshedServer?.id ?? null;
      if (watchId) {
        await persistServerPassword(watchId, passwordForNewServer);
      } else {
        pendingCreatedServerPassword.value = passwordForNewServer;
      }
      if (shouldScheduleStartupTasks) {
        if (watchId) {
          await registerStartupTasksForServer(watchId, startupConfigForCreate, passwordForNewServer);
        } else {
          pendingStartupTaskCreate.value = {
            config: startupConfigForCreate,
            password: passwordForNewServer,
          };
        }
      }
      await sendUserNotification(
        "ECS create accepted",
        `${useCustomName.value ? name.value.trim() || "new server" : "new server"} (${watchId ?? "pending ID"}) in ${region.value}.`
      );
      startPolling(watchId);
    }
  } catch (err) {
    setError(`Create failed: ${errorToString(err)}`);
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <main class="page">
    <header class="topbar">
      <div class="brand">
        <p class="eyebrow">HC Forge</p>
        <h1>Cloud Ops Console</h1>
        <p class="subtitle">
          Shared credentials across Huawei Cloud services.
        </p>
      </div>
      <div class="credentials-card">
        <div class="cred-grid">
          <label class="mini-field">
            <span>Access Key</span>
            <input
              v-model="accessKey"
              autocomplete="off"
              spellcheck="false"
              placeholder="AK..."
            />
          </label>

          <label class="mini-field">
            <span>Secret Key</span>
            <input
              v-model="secretKey"
              type="password"
              autocomplete="off"
              spellcheck="false"
              placeholder="SK..."
            />
          </label>
        </div>
        <div class="cred-actions">
          <button
            class="ghost minor"
            :disabled="savingCredentials || !storeReady"
            @click="saveCredentials"
          >
            {{ savingCredentials ? "Saving..." : "Save Credentials" }}
          </button>
        </div>
      </div>
    </header>

    <section class="hero service-hero">
      <div>
        <p class="eyebrow">Service</p>
        <h2>Elastic Cloud Server</h2>
      </div>
      <div class="chip tw-pill">ECS Module</div>
    </section>

    <div class="layout">
      <section class="panel">
        <div class="panel-head">
          <h2>Server Inputs</h2>
          <button class="primary quick-create" :disabled="!canCreate" @click="createEcs">
            {{ creating ? "Creating..." : "Create ECS" }}
          </button>
        </div>
        <div class="grid inputs-grid">
          <label class="field region-field">
            <span>Region</span>
            <select v-model="region">
              <option v-for="item in regions" :key="item" :value="item">
                {{ item }}
              </option>
            </select>
          </label>

          <div class="field">
            <span>Name</span>
            <div class="toggle-inline">
              <input id="custom-name" v-model="useCustomName" type="checkbox" />
              <label for="custom-name">Use custom name</label>
            </div>
            <input
              v-model="name"
              :disabled="!useCustomName"
              placeholder="my-ecs-prod"
            />
          </div>

          <div class="field span-2 startup-update-field">
            <span>Startup Tasks (new VM only)</span>
            <div class="startup-task-toggles">
              <div class="toggle-inline">
                <input id="auto-update-vm" v-model="autoUpdateVmOnStartup" type="checkbox" />
                <label for="auto-update-vm">Update VM on startup</label>
              </div>
              <div class="toggle-inline">
                <input id="setup-gui-rdp" v-model="setupGuiRdpOnStartup" type="checkbox" />
                <label for="setup-gui-rdp">Setup GUI + RDP on startup</label>
              </div>
            </div>
            <span class="muted tiny">
              Applies only to VMs created by the current create action. Existing VMs are never changed.
            </span>
            <span class="muted tiny">
              Tasks start when the VM becomes ACTIVE and has a public EIP; progress appears in the ECS card and logs.
            </span>
          </div>

          <div class="field span-2">
            <div class="field-title-row">
              <span>Image ({{ filteredImages.length }}/{{ images.length }})</span>
              <ReloadIconButton
                :disabled="!canListImages"
                :loading="loadingImages"
                :title="loadingImages ? 'Reloading images...' : 'Reload images'"
                @click="loadImages"
              />
            </div>
            <div class="combo">
              <input v-model="imageSearch" placeholder="Search images..." />
              <select v-model="imageId">
                <option value="" disabled>Select an image</option>
                <option
                  v-for="image in filteredImages"
                  :key="image.id"
                  :value="image.id"
                >
                  {{ image.name }} ({{ image.id }})
                </option>
              </select>
            </div>
          </div>

          <div class="field span-2">
            <div class="field-title-row">
              <span>Flavor ({{ filteredFlavors.length }}/{{ flavors.length }})</span>
              <ReloadIconButton
                :disabled="!canListFlavors"
                :loading="loadingFlavors"
                :title="loadingFlavors ? 'Reloading flavors...' : 'Reload flavors'"
                @click="loadFlavors"
              />
            </div>
            <div class="combo">
              <input v-model="flavorSearch" placeholder="Search flavors..." />
              <div class="inline-pairs">
                <label class="mini-field">
                  <span>Architecture</span>
                  <select v-model="flavorArchFilter">
                    <option value="all">All</option>
                    <option
                      v-for="arch in flavorArchitectureOptions.filter((item) => item !== 'all')"
                      :key="arch"
                      :value="arch"
                    >
                      {{ arch }}
                    </option>
                  </select>
                </label>
                <label class="mini-field">
                  <span>vCPU Bucket</span>
                  <select v-model="flavorVcpuFilter">
                    <option value="all">All</option>
                    <option value="1-2">1-2</option>
                    <option value="4-8">4-8</option>
                    <option value="16+">16+</option>
                  </select>
                </label>
              </div>
              <select v-model="flavorId">
                <option value="" disabled>Select a flavor</option>
                <optgroup
                  v-for="group in flavorGroups"
                  :key="group.key"
                  :label="group.label"
                >
                  <option
                    v-for="flavor in group.flavors"
                    :key="flavor.id"
                    :value="flavor.id"
                  >
                    {{ formatFlavorLabel(flavor) }}
                  </option>
                </optgroup>
              </select>
            </div>
            <span v-if="imageMinRam" class="muted tiny">
              Image requires at least {{ imageMinRam }} MB RAM.
            </span>
            <span v-if="!filteredFlavors.length && flavors.length" class="muted tiny">
              No flavors meet current filters. Adjust architecture, vCPU bucket, or image choice.
            </span>
          </div>

          <div class="fold-section span-2">
            <div class="fold-head">
              <button
                class="fold-toggle"
                type="button"
                @click="passwordSectionOpen = !passwordSectionOpen"
              >
                <span>Administrator Access</span>
                <span class="fold-state">{{ passwordSectionOpen ? "Hide" : "Show" }}</span>
              </button>
              <span v-if="passwordCopyFeedback" class="copy-feedback tiny">
                {{ passwordCopyFeedback }}
              </span>
              <button class="ghost minor fold-copy" type="button" @click="copyCurrentPassword">
                Copy Password
              </button>
            </div>
            <transition name="fold">
              <div v-show="passwordSectionOpen" class="fold-body">
                <div class="field password-field">
                  <div class="field-head">
                    <span>Administrator Password</span>
                    <button
                      class="ghost minor"
                      type="button"
                      :disabled="!useGeneratedPassword"
                      @click="regeneratePassword"
                    >
                      Regenerate
                    </button>
                  </div>
                  <div class="toggle-inline">
                    <input
                      id="generated-password"
                      v-model="useGeneratedPassword"
                      type="checkbox"
                    />
                    <label for="generated-password">
                      Use generated password (recommended)
                    </label>
                  </div>
                  <div class="password-input-row">
                    <input
                      v-if="useGeneratedPassword"
                      :value="generatedPassword"
                      :type="showAdminPassword ? 'text' : 'password'"
                      readonly
                      spellcheck="false"
                    />
                    <input
                      v-else
                      v-model="customPassword"
                      :type="showAdminPassword ? 'text' : 'password'"
                      placeholder="Enter your own admin password"
                      spellcheck="false"
                    />
                    <button
                      class="ghost minor eye-toggle"
                      type="button"
                      :aria-label="showAdminPassword ? 'Hide password' : 'Show password'"
                      @click="showAdminPassword = !showAdminPassword"
                    >
                      {{ showAdminPassword ? "🙈" : "👁️" }}
                    </button>
                  </div>
                  <div class="password-actions">
                    <button class="ghost minor" type="button" @click="copyCurrentPassword">
                      Copy Password
                    </button>
                    <span class="muted tiny">
                      {{
                        passwordCopyFeedback ??
                        `Must be ${PASSWORD_MIN_LENGTH}-${PASSWORD_MAX_LENGTH} chars with upper/lower/number/symbol.`
                      }}
                    </span>
                  </div>
                  <span v-if="passwordError" class="field-error tiny">
                    {{ passwordError }}
                  </span>
                </div>
              </div>
            </transition>
          </div>

          <div class="fold-section span-2">
            <button
              class="fold-toggle"
              type="button"
              @click="storageSectionOpen = !storageSectionOpen"
            >
              <span>Storage And Public Network</span>
              <span class="fold-state">{{ storageSectionOpen ? "Hide" : "Show" }}</span>
            </button>
            <transition name="fold">
              <div v-show="storageSectionOpen" class="fold-body">
                <div class="grid">
                  <label class="field">
                    <span>Root Volume Type</span>
                    <select v-model="rootVolumeType" :disabled="!imageId">
                      <option value="GPSSD">GPSSD (General Purpose SSD)</option>
                      <option value="SATA">SATA (Common I/O)</option>
                      <option value="SAS">SAS (High I/O)</option>
                      <option value="SSD">Ultra-I/O SSD (Ultra I/O)</option>
                      <option value="ESSD">ESSD (Extreme SSD)</option>
                      <option value="GPSSD2">GPSSD2 (General Purpose SSD V2)</option>
                      <option value="ESSD2">ESSD2 (Extreme SSD V2)</option>
                    </select>
                  </label>

                  <div class="field">
                    <span>Public Network</span>
                    <div class="toggle-inline">
                      <input id="eip" v-model="allocateEip" type="checkbox" />
                      <label for="eip">Allocate public EIP</label>
                    </div>
                    <label class="mini-field">
                      <span>EIP Bandwidth (Mbit/s)</span>
                      <input
                        v-model.number="eipBandwidthSize"
                        type="number"
                        :min="EIP_BANDWIDTH_MIN_MBIT"
                        :max="EIP_BANDWIDTH_MAX_MBIT"
                        step="1"
                        :disabled="!allocateEip"
                      />
                    </label>
                    <span class="muted tiny">
                      Charge mode is fixed to traffic. Huawei ECS API allows
                      {{ EIP_BANDWIDTH_MIN_MBIT }}-{{ EIP_BANDWIDTH_MAX_MBIT }} Mbit/s.
                    </span>
                  </div>

                  <div class="field span-2">
                    <span>Root Volume Size (GB)</span>
                    <div class="range-row">
                      <input
                        v-model.number="rootVolumeSize"
                        type="range"
                        :min="imageMinDisk"
                        max="1024"
                        step="1"
                      />
                      <input
                        v-model.number="rootVolumeSize"
                        type="number"
                        :min="imageMinDisk"
                        max="1024"
                      />
                    </div>
                    <div class="range-meta">
                      <span>{{ rootVolumeSize }} GB</span>
                      <span class="muted">Min {{ imageMinDisk }} GB</span>
                    </div>
                  </div>

                  <div class="field span-2">
                    <span>EVS Data Disk (optional)</span>
                    <div class="toggle-inline">
                      <input id="include-data-disk" v-model="includeDataDisk" type="checkbox" />
                      <label for="include-data-disk">Attach EVS data disk on create</label>
                    </div>
                    <div class="inline-pairs">
                      <label class="mini-field">
                        <span>Volume Type</span>
                        <select v-model="dataDiskType" :disabled="!includeDataDisk">
                          <option value="GPSSD">GPSSD</option>
                          <option value="SATA">SATA</option>
                          <option value="SAS">SAS</option>
                          <option value="SSD">SSD</option>
                          <option value="ESSD">ESSD</option>
                          <option value="GPSSD2">GPSSD2</option>
                          <option value="ESSD2">ESSD2</option>
                        </select>
                      </label>
                      <label class="mini-field">
                        <span>Size (GB)</span>
                        <input
                          v-model.number="dataDiskSize"
                          type="number"
                          :min="DATA_DISK_MIN_GB"
                          :max="DATA_DISK_MAX_GB"
                          :disabled="!includeDataDisk"
                        />
                      </label>
                      <label class="mini-field">
                        <span>Count</span>
                        <input
                          v-model.number="dataDiskCount"
                          type="number"
                          :min="DATA_DISK_MIN_COUNT"
                          :max="DATA_DISK_MAX_COUNT"
                          :disabled="!includeDataDisk"
                        />
                      </label>
                      <div class="mini-field">
                        <span>Flags</span>
                        <div class="toggle-inline">
                          <input
                            id="data-disk-multiattach"
                            v-model="dataDiskMultiattach"
                            type="checkbox"
                            :disabled="!includeDataDisk"
                          />
                          <label for="data-disk-multiattach">Shareable (multiattach)</label>
                        </div>
                        <div class="toggle-inline">
                          <input
                            id="data-disk-scsi"
                            v-model="dataDiskHwPassthrough"
                            type="checkbox"
                            :disabled="!includeDataDisk"
                          />
                          <label for="data-disk-scsi">SCSI passthrough</label>
                        </div>
                      </div>
                    </div>
                    <span class="muted tiny">
                      Defaults: no data disk attached; when enabled uses {{ DEFAULT_DATA_DISK_SIZE_GB }} GB
                      GPSSD.
                    </span>
                  </div>
                </div>
              </div>
            </transition>
          </div>
        </div>

        <div class="fold-section">
          <button
            class="fold-toggle"
            type="button"
            @click="imageFilterSectionOpen = !imageFilterSectionOpen"
          >
            <span>Image Filters</span>
            <span class="fold-state">{{ imageFilterSectionOpen ? "Hide" : "Show" }}</span>
          </button>
          <transition name="fold">
            <div v-show="imageFilterSectionOpen" class="fold-body">
              <div class="advanced">
                <div class="advanced-header">
                  <span>Image Filters (optional)</span>
                  <span class="muted tiny">Usually keep defaults.</span>
                </div>
                <div class="grid minor-grid">
                  <label class="field">
                    <span>Visibility</span>
                    <select v-model="imageVisibility">
                      <option value="all">All</option>
                      <option value="public">Public</option>
                      <option value="private">Private</option>
                      <option value="shared">Shared</option>
                    </select>
                  </label>

                  <label class="field">
                    <span>Image Type</span>
                    <select v-model="imageType">
                      <option value="all">All</option>
                      <option value="gold">Gold (Public)</option>
                      <option value="private">Private</option>
                      <option value="shared">Shared</option>
                      <option value="market">Marketplace</option>
                    </select>
                  </label>
                </div>
              </div>
            </div>
          </transition>
        </div>

        <div class="fold-section">
          <button
            class="fold-toggle"
            type="button"
            @click="networkSectionOpen = !networkSectionOpen"
          >
            <span>Network</span>
            <span class="fold-state">{{ networkSectionOpen ? "Hide" : "Show" }}</span>
          </button>
          <transition name="fold">
            <div v-show="networkSectionOpen" class="fold-body">
              <div class="grid">
                <label class="field">
                  <div class="field-title-row">
                    <span>VPC</span>
                    <ReloadIconButton
                      :disabled="loadingVpcs"
                      :loading="loadingVpcs"
                      :title="loadingVpcs ? 'Reloading VPCs...' : 'Reload VPCs'"
                      @click="loadVpcs"
                    />
                  </div>
                  <select v-model="selectedVpc">
                    <option value="" disabled>Select a VPC</option>
                    <option v-for="vpc in vpcs" :key="vpc.id" :value="vpc.id">
                      {{ vpc.name }}
                    </option>
                  </select>
                </label>

                <label class="field">
                  <div class="field-title-row">
                    <span>Subnet</span>
                    <ReloadIconButton
                      :disabled="!canLoadSubnets"
                      :loading="loadingSubnets"
                      :title="loadingSubnets ? 'Reloading subnets...' : 'Reload subnets'"
                      @click="loadSubnets"
                    />
                  </div>
                  <select v-model="selectedSubnet">
                    <option value="" disabled>Select a subnet</option>
                    <option
                      v-for="subnet in subnets"
                      :key="subnet.id"
                      :value="subnet.id"
                    >
                      {{ subnet.name }} ({{ subnet.cidr }})
                    </option>
                  </select>
                </label>
              </div>
            </div>
          </transition>
        </div>

        <div class="actions minor-actions">
          <button class="ghost minor" :disabled="loadingAll" @click="loadAll">
            {{ loadingAll ? "Reloading All..." : "Reload All" }}
          </button>
        </div>

        <div class="bottom-create-row">
          <button class="primary cta bottom-create" :disabled="!canCreate" @click="createEcs">
            {{ creating ? "Creating..." : "Create ECS" }}
          </button>
        </div>

        <p class="muted" v-if="loadingAll">
          Reloading images, flavors, VPCs, subnets, ECSes, EIPs, and EVS disks...
        </p>
        <p class="muted tiny" v-else>
          Images: {{ images.length }} ({{ cacheAge.images }}) • Flavors:
          {{ flavors.length }} ({{ cacheAge.flavors }}) • VPCs: {{ vpcs.length }}
          ({{ cacheAge.vpcs }}) • Subnets: {{ subnets.length }} ({{ cacheAge.subnets }}) • EVS:
          {{ evss.length }} ({{ cacheAge.evss }})
        </p>
      </section>

      <section class="panel output">
        <div class="output-header">
          <h2>Response</h2>
        </div>

        <p v-if="errorMsg" class="error">{{ errorMsg }}</p>
        <p v-if="deleteMsg" class="muted tiny">{{ deleteMsg }}</p>

        <div class="output-grid">
          <div class="output-card wide">
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
                    <span v-if="pollingEcs">
                      Active ({{ pollingAttempts }}/{{ POLL_MAX_ATTEMPTS }})
                    </span>
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
                  <button
                    class="ghost minor"
                    :disabled="!pollingEcs"
                    @click="stopPolling"
                  >
                    Stop
                  </button>
                </div>
              </div>
            </div>
            <p v-else class="muted">No create action yet.</p>
            <details v-if="result" class="raw">
              <summary>Raw create response</summary>
              <pre class="body">{{ result.body }}</pre>
            </details>
          </div>

          <div class="output-card">
            <div class="card-head-inline">
              <div class="card-title">Elastic IPs</div>
              <ReloadIconButton
                :disabled="loadingEips"
                :loading="loadingEips"
                :title="loadingEips ? 'Reloading EIPs...' : 'Reload EIPs'"
                @click="loadEips()"
              />
            </div>
            <div class="card-subtitle">{{ eips.length }} total • Updated {{ cacheAge.eips }}</div>
            <div v-if="eips.length" class="entity-list eip-list">
              <article
                v-for="(eip, index) in eips"
                :key="eip.id ?? eip.public_ip_address ?? `eip-${index}`"
                class="entity-item eip-item"
              >
                <div class="entity-item-head">
                  <div class="entity-title mono">{{ eip.public_ip_address ?? "—" }}</div>
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

          <div class="output-card">
            <div class="card-head-inline">
              <div class="card-title">EVS Disks</div>
              <ReloadIconButton
                :disabled="loadingEvss"
                :loading="loadingEvss"
                :title="loadingEvss ? 'Reloading EVS disks...' : 'Reload EVS disks'"
                @click="loadEvss()"
              />
            </div>
            <div class="card-subtitle">{{ evss.length }} total • Updated {{ cacheAge.evss }}</div>
            <div v-if="evss.length" class="entity-list evs-list">
              <article
                v-for="(volume, index) in evss"
                :key="volume.id ?? volume.name ?? `evs-${index}`"
                class="entity-item evs-item"
              >
                <div class="entity-item-head">
                  <div>
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

          <div class="output-card">
            <div class="card-head-inline">
              <div class="card-title">ECS Instances</div>
              <ReloadIconButton
                :disabled="loadingEcses"
                :loading="loadingEcses"
                :title="loadingEcses ? 'Reloading ECS instances...' : 'Reload ECS instances'"
                @click="loadEcses()"
              />
            </div>
            <div class="card-subtitle">
              {{ ecses.length }} total • Updated {{ cacheAge.ecses }}
            </div>
            <div v-if="ecses.length" class="entity-list ecs-list">
              <article
                v-for="(ecs, index) in ecses"
                :key="ecs.id ?? ecs.name ?? `ecs-${index}`"
                class="entity-item ecs-item"
              >
                <div class="entity-item-head">
                  <div>
                    <div class="entity-title mono">{{ ecs.name ?? ecs.id ?? "—" }}</div>
                    <div class="muted tiny">
                      ID: <span class="mono">{{ ecs.id ?? "—" }}</span>
                    </div>
                  </div>
                  <span class="status-pill" :class="statusTone(ecs.status)">
                    {{ ecs.status ?? "UNKNOWN" }}
                  </span>
                </div>
                <div class="entity-meta-grid">
                  <div class="entity-meta-item">
                    <span class="entity-meta-key">Flavor</span>
                    <span class="entity-meta-value mono">
                      {{ ecs.flavor?.name ?? ecs.flavor?.id ?? "—" }}
                    </span>
                  </div>
                  <div class="entity-meta-item">
                    <span class="entity-meta-key">AZ</span>
                    <span class="entity-meta-value">{{ ecs.availability_zone ?? "—" }}</span>
                  </div>
                  <div class="entity-meta-item">
                    <span class="entity-meta-key">Public EIP</span>
                    <span class="entity-meta-value mono">
                      {{ findSshHostForServer(ecs) ?? "Not assigned" }}
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
                  </div>
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
            @update:use-form-password="sshUseFormPassword = $event"
            @update:manual-password="sshManualPassword = $event"
            @update:command-input="sshCommandInput = $event"
          />
        </div>
      </section>
    </div>
    <transition name="dialog">
      <div v-if="confirmDialog.open" class="dialog-scrim" @click.self="closeConfirmDialog(false)">
        <section class="dialog-shell" role="dialog" aria-modal="true">
          <div class="dialog-head">
            <h3>{{ confirmDialog.title }}</h3>
            <span class="dialog-kind" :class="`kind-${confirmDialog.kind}`">
              {{ confirmDialog.kind }}
            </span>
          </div>
          <p class="dialog-message">{{ confirmDialog.message }}</p>
          <div class="dialog-actions">
            <button class="ghost minor danger" type="button" @click="closeConfirmDialog(true)">
              {{ confirmDialog.okLabel }}
            </button>
            <button class="ghost minor" type="button" @click="closeConfirmDialog(false)">
              {{ confirmDialog.cancelLabel }}
            </button>
          </div>
        </section>
      </div>
    </transition>
    <AppLogsPanel
      :open="logPanelOpen"
      :entries="orderedLogEntries"
      :has-unread-error="logsUnreadError"
      :format-date-time="formatDateTime"
      @toggle="logPanelOpen = !logPanelOpen"
      @clear="clearLogs"
    />
  </main>
</template>

<style>
:root {
  --ink: #2f0f14;
  --muted: #74484f;
  --panel: #ffffff;
  --panel-border: #efc2c7;
  --surface-soft: #fff8f8;
  --surface-tint: #fff0f1;
  --accent: #c32936;
  --accent-strong: #a61f2c;
  --accent-warm: #e07a39;
  --danger: #b42318;
  --bg: #fff5f6;
  --bg-strong: #ffdfe2;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
  font-size: 15px;
  line-height: 1.5;
  color: var(--ink);
  background: var(--bg);
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  background:
    radial-gradient(circle at 6% 8%, rgba(195, 41, 54, 0.12), transparent 34%),
    radial-gradient(circle at 98% 0%, rgba(224, 122, 57, 0.08), transparent 32%),
    linear-gradient(180deg, #fffafb 0%, #fff3f5 100%);
  min-height: 100vh;
}

#app {
  min-height: 100vh;
}

.page {
  max-width: none;
  width: 100%;
  margin: 0;
  padding: 14px 14px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.hero {
  display: flex;
  gap: 18px;
  align-items: center;
  justify-content: space-between;
}

.eyebrow {
  font-size: 0.74rem;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  margin: 0 0 6px;
  font-weight: 700;
  color: #9b6c74;
}

.topbar {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  justify-content: space-between;
  padding: 16px 18px;
  border-radius: 16px;
  background:
    radial-gradient(circle at 82% 16%, rgba(253, 164, 175, 0.2), transparent 30%),
    linear-gradient(135deg, #330f16 0%, #4f1521 56%, #6e1d2b 100%);
  color: #fff7f7;
  border: 1px solid rgba(254, 202, 202, 0.3);
  box-shadow: 0 12px 24px rgba(123, 37, 44, 0.24);
}

.brand {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.topbar h1 {
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 1.9rem;
  margin: 0 0 4px;
  letter-spacing: 0.01em;
}

.subtitle {
  margin: 0;
  max-width: 560px;
  color: rgba(241, 245, 249, 0.8);
  font-size: 0.93rem;
}

.chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 10px;
  border-radius: 999px;
  background: transparent;
  border: 0;
  font-weight: 700;
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  text-align: center;
}

.service-hero .chip.tw-pill {
  border-color: rgba(252, 165, 165, 0.95);
  background: rgba(127, 29, 29, 0.3);
  color: #fee2e2;
  letter-spacing: 0.08em;
}

.credentials-card {
  min-width: 390px;
  background: rgba(255, 249, 250, 0.98);
  color: var(--ink);
  border-radius: 14px;
  padding: 12px;
  border: 1px solid rgba(242, 201, 206, 0.92);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.94),
    0 8px 20px rgba(123, 37, 44, 0.12);
}

.cred-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.cred-actions {
  margin-top: 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.mini-field {
  display: grid;
  gap: 4px;
  font-weight: 600;
}

.mini-field span {
  font-size: 0.72rem;
  color: #7c4e56;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.service-hero {
  padding: 12px 16px;
  border-radius: 16px;
  background:
    linear-gradient(130deg, rgba(82, 20, 29, 0.96), rgba(116, 27, 39, 0.94)),
    linear-gradient(45deg, rgba(224, 122, 57, 0.15), transparent 65%);
  color: #fff5f5;
  border: 1px solid rgba(252, 165, 165, 0.24);
  box-shadow: 0 8px 16px rgba(123, 37, 44, 0.16);
}

.service-hero h2 {
  margin: 0;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 1.62rem;
}

.layout {
  display: grid;
  grid-template-columns: minmax(560px, 1.2fr) minmax(460px, 1fr);
  align-items: start;
  gap: 12px;
}

.panel {
  background: var(--panel);
  border-radius: 16px;
  padding: 16px;
  box-shadow: 0 10px 22px rgba(123, 37, 44, 0.08);
  border: 1px solid var(--panel-border);
  backdrop-filter: blur(1px);
  animation: panel-in 280ms ease both;
}

.panel h2 {
  margin: 0 0 10px;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 1.2rem;
  letter-spacing: 0.01em;
}

.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}

.panel-head h2 {
  margin: 0;
}

.quick-create {
  min-width: 172px;
  min-height: 40px;
}

.fold-section {
  margin-top: 12px;
  border: 1px solid #f0ccd0;
  border-radius: 14px;
  background: linear-gradient(180deg, rgba(255, 250, 250, 0.95), rgba(255, 245, 246, 0.98));
  overflow: hidden;
}

.fold-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-right: 8px;
  border-bottom: 1px solid #f0d0d4;
  background: rgba(255, 243, 244, 0.98);
}

.grid > .fold-section {
  margin-top: 0;
}

.fold-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border: 0;
  border-bottom: 0;
  border-radius: 0;
  box-shadow: none;
  background: rgba(255, 243, 244, 0.98);
  color: #5d2029;
  font-weight: 700;
  letter-spacing: 0.04em;
  font-size: 0.82rem;
  text-transform: uppercase;
}

.fold-toggle:hover,
.fold-toggle:active,
.fold-toggle:focus-visible {
  transform: none;
  opacity: 1;
  box-shadow: none;
}

.fold-copy {
  flex: 0 0 auto;
}

.copy-feedback {
  color: #9f1239;
  white-space: nowrap;
  margin-right: 2px;
}

.fold-state {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: #a1656f;
}

.fold-body {
  padding: 10px;
  display: grid;
  gap: 10px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.inputs-grid {
  grid-template-columns: minmax(240px, 340px) minmax(0, 1fr);
}

.field {
  display: grid;
  gap: 8px;
  font-weight: 500;
  padding: 11px;
  border-radius: 12px;
  border: 1px solid #f0ccd0;
  background: rgba(255, 255, 255, 0.98);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.field > span:first-child {
  font-size: 0.8rem;
  color: #6a2530;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
}

.field-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.field-title-row > span {
  font-size: 0.8rem;
  color: #6a2530;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
}

.field-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.field-error {
  color: var(--danger);
  font-weight: 600;
}

.span-2 {
  grid-column: span 2;
}

.region-field {
  max-width: none;
}

.region-field select {
  max-width: none;
}

.startup-update-field {
  gap: 6px;
}

.startup-task-toggles {
  display: flex;
  flex-wrap: wrap;
  gap: 7px 14px;
}

.combo {
  display: grid;
  gap: 8px;
}

.inline-pairs {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.toggle-inline {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 0.84rem;
  color: #6f4149;
  line-height: 1.3;
}

input,
select,
textarea {
  padding: 0 12px;
  min-height: 40px;
  border-radius: 10px;
  border: 1px solid #e8c0c6;
  background: #ffffff;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
  font-size: 0.9rem;
  color: #3f1820;
  transition: border-color 0.18s, box-shadow 0.18s, background-color 0.18s;
  box-shadow: inset 0 1px 1px rgba(15, 23, 42, 0.04);
}

select {
  appearance: none;
  background-image:
    linear-gradient(45deg, transparent 50%, #9f3a47 50%),
    linear-gradient(135deg, #9f3a47 50%, transparent 50%),
    linear-gradient(to right, transparent, transparent);
  background-position:
    calc(100% - 16px) calc(50% - 3px),
    calc(100% - 11px) calc(50% - 3px),
    100% 0;
  background-size:
    5px 5px,
    5px 5px,
    2.5em 2.5em;
  background-repeat: no-repeat;
  padding-right: 34px;
  line-height: 1.2;
}

textarea {
  padding: 10px 12px;
  min-height: 92px;
  resize: vertical;
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
  line-height: 1.42;
}

input:focus,
select:focus,
textarea:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(195, 41, 54, 0.16);
}

input:disabled,
select:disabled,
textarea:disabled {
  background: #fdf2f3;
  color: #b58d95;
}

input[type="range"] {
  padding: 0;
  accent-color: var(--accent);
}

.password-field {
  background: linear-gradient(180deg, rgba(255, 246, 247, 0.98), rgba(255, 241, 242, 0.96));
  border-color: rgba(195, 41, 54, 0.3);
}

.password-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.password-input-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 46px;
  gap: 7px;
  align-items: center;
}

.password-input-row input {
  min-height: 38px;
}

.eye-toggle {
  padding: 0;
  min-width: 46px;
  min-height: 38px;
  font-size: 0.95rem;
}

.range-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 120px;
  gap: 10px;
  align-items: center;
}

.range-meta {
  display: flex;
  justify-content: space-between;
  font-size: 0.82rem;
  color: #8c5a63;
}

.toggle {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 16px;
  font-weight: 500;
}

.divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(195, 41, 54, 0.4), transparent);
  margin: 24px 0;
}

.advanced {
  margin-top: 0;
  padding: 12px;
  border-radius: 12px;
  background: #fff8f8;
  border: 1px dashed rgba(148, 163, 184, 0.52);
}

.advanced-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-weight: 600;
  color: #6f3942;
}

.minor-grid .field span {
  font-size: 0.78rem;
  color: #8c5a63;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
}

.network-actions {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.minor-actions {
  margin-top: 10px;
}

button {
  border: 1px solid transparent;
  border-radius: 10px;
  min-height: 36px;
  padding: 0 13px;
  font-weight: 650;
  cursor: pointer;
  font-size: 0.84rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  white-space: nowrap;
  transition:
    box-shadow 0.18s ease,
    opacity 0.18s,
    background-color 0.16s ease,
    border-color 0.16s ease,
    color 0.16s ease;
  line-height: 1;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
  box-shadow: none;
}

button:hover,
button:active,
button:focus-visible {
  transform: none;
}

.primary {
  background: linear-gradient(135deg, #a61f2c 0%, #c32936 100%);
  color: #fff;
  border-color: rgba(166, 31, 44, 0.45);
  box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.1);
}

.primary:not(:disabled):hover {
  background: linear-gradient(135deg, #951c28 0%, #ad2632 100%);
  box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.16);
}

.ghost {
  background: #fff8f8;
  border: 1px solid #efc2c7;
  color: #6c2530;
}

.ghost:not(:disabled):hover {
  background: #fff1f2;
  border-color: #e5a8af;
}

.minor {
  padding: 0 10px;
  min-height: 34px;
  font-size: 0.82rem;
  font-weight: 600;
  background: #fff7f7;
  border-color: #efc2c7;
}

.danger {
  border-color: rgba(180, 35, 24, 0.34);
  color: var(--danger);
  background: #fff6f5;
}

.danger:not(:disabled):hover {
  background: #feeceb;
  border-color: rgba(180, 35, 24, 0.45);
}

.cta {
  grid-column: 1 / -1;
  min-height: 40px;
  font-size: 0.94rem;
  padding: 0 14px;
}

.bottom-create-row {
  margin-top: 10px;
}

.bottom-create {
  width: 100%;
}

.card-head-inline {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.output {
  display: flex;
  flex-direction: column;
  gap: 12px;
  position: sticky;
  top: 12px;
  max-height: calc(100vh - 24px);
  overflow: auto;
  padding-right: 4px;
}

.output-header {
  margin-bottom: 2px;
}

.output-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
}

.output-card {
  border-radius: 14px;
  border: 1px solid #efc2c7;
  padding: 12px 14px;
  background: #fffafb;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 170px;
  min-width: 0;
  box-shadow: 0 6px 14px rgba(123, 37, 44, 0.06);
  animation: card-in 260ms ease both;
}

.output-card.wide {
  grid-column: auto;
  animation-delay: 30ms;
}

.output-card:nth-child(2) {
  animation-delay: 70ms;
}

.output-card:nth-child(3) {
  animation-delay: 100ms;
}

.card-title {
  font-weight: 700;
  font-size: 1rem;
}

.card-subtitle {
  font-size: 0.8rem;
  color: var(--muted);
}

.status-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  min-height: 24px;
  padding: 0 10px;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  border: 1px solid transparent;
  white-space: nowrap;
}

.status-ok {
  color: #166534;
  border-color: rgba(22, 101, 52, 0.28);
  background: rgba(134, 239, 172, 0.3);
}

.status-progress {
  color: #854d0e;
  border-color: rgba(133, 77, 14, 0.3);
  background: rgba(253, 224, 71, 0.3);
}

.status-error {
  color: #b42318;
  border-color: rgba(180, 35, 24, 0.35);
  background: rgba(252, 165, 165, 0.26);
}

.status-muted {
  color: #4b5563;
  border-color: rgba(107, 114, 128, 0.32);
  background: rgba(229, 231, 235, 0.45);
}

.status-neutral {
  color: #6b3841;
  border-color: rgba(150, 108, 115, 0.35);
  background: rgba(255, 228, 231, 0.55);
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.polling-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.polling-actions {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
}

.meta {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 9px;
  min-width: 0;
}

.mono {
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
  font-size: 0.84rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.warning {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #b42318;
  padding: 8px 9px;
  border-radius: 10px;
  font-size: 0.82rem;
}

.raw summary {
  cursor: pointer;
  font-weight: 600;
  font-size: 0.82rem;
  color: #4d6178;
}

.raw .body {
  margin-top: 8px;
  max-height: 220px;
}

.entity-list {
  display: grid;
  gap: 10px;
  max-height: 360px;
  overflow: auto;
  padding-right: 2px;
}

.entity-item {
  border: 1px solid #efccd1;
  border-radius: 12px;
  background: linear-gradient(180deg, #ffffff, #fff8f9);
  padding: 10px;
  display: grid;
  gap: 9px;
  min-width: 0;
}

.entity-item-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.entity-title {
  font-size: 0.9rem;
  font-weight: 700;
  line-height: 1.25;
  color: #4f1521;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entity-meta-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.entity-meta-item {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.entity-meta-key {
  font-size: 0.68rem;
  color: #8a5b63;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-weight: 700;
}

.entity-meta-value {
  font-size: 0.82rem;
  color: #4d1d25;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ecs-item-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.ecs-item-actions button {
  flex: 1 1 102px;
  min-width: 98px;
  min-height: 34px;
  padding: 0 8px;
  font-size: 0.78rem;
}

.update-state-failed {
  color: #b42318;
  font-weight: 700;
}

.update-state-running {
  color: #8a5206;
  font-weight: 700;
}

.update-progress-hint {
  display: block;
  margin-top: 2px;
  line-height: 1.35;
  white-space: normal;
}

.ssh-action.active {
  border-color: rgba(166, 31, 44, 0.54);
  color: #a61f2c;
  background: rgba(166, 31, 44, 0.1);
}

.dialog-scrim {
  position: fixed;
  inset: 0;
  background: rgba(24, 10, 15, 0.45);
  backdrop-filter: blur(1.5px);
  z-index: 140;
  display: grid;
  place-items: center;
  padding: 16px;
}

.dialog-shell {
  width: min(560px, calc(100vw - 24px));
  border-radius: 14px;
  border: 1px solid #efc1c5;
  background: linear-gradient(180deg, #fffdfd, #fff4f6);
  box-shadow: 0 18px 36px rgba(35, 11, 17, 0.24);
  padding: 14px;
  display: grid;
  gap: 12px;
}

.dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.dialog-head h3 {
  margin: 0;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 1.06rem;
  color: #5d2029;
}

.dialog-kind {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 22px;
  border-radius: 999px;
  padding: 0 9px;
  font-size: 0.68rem;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  font-weight: 700;
  border: 1px solid transparent;
}

.dialog-kind.kind-warning {
  color: #854d0e;
  border-color: rgba(133, 77, 14, 0.28);
  background: rgba(254, 215, 170, 0.4);
}

.dialog-kind.kind-error {
  color: #b42318;
  border-color: rgba(180, 35, 24, 0.32);
  background: rgba(252, 165, 165, 0.28);
}

.dialog-kind.kind-info {
  color: #1e40af;
  border-color: rgba(30, 64, 175, 0.3);
  background: rgba(191, 219, 254, 0.38);
}

.dialog-message {
  margin: 0;
  color: #4b1f27;
  line-height: 1.45;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
}

.fold-enter-active,
.fold-leave-active {
  transition: max-height 0.22s ease, opacity 0.18s ease;
  overflow: hidden;
}

.fold-enter-from,
.fold-leave-to {
  max-height: 0;
  opacity: 0;
}

.fold-enter-to,
.fold-leave-from {
  max-height: 900px;
  opacity: 1;
}

.dialog-enter-active,
.dialog-leave-active {
  transition: opacity 0.18s ease;
}

.dialog-enter-active .dialog-shell,
.dialog-leave-active .dialog-shell {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
}

.dialog-enter-from .dialog-shell,
.dialog-leave-to .dialog-shell {
  transform: translateY(8px);
  opacity: 0;
}

.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 5px 11px;
  border-radius: 999px;
  background: rgba(166, 31, 44, 0.14);
  color: #a61f2c;
  font-weight: 700;
  font-size: 0.86rem;
}

.body {
  white-space: pre-wrap;
  background: #0f172a;
  color: #e2e8f0;
  border-radius: 12px;
  padding: 14px;
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
  font-size: 0.82rem;
  max-height: 260px;
  overflow: auto;
}

.error {
  color: #b42318;
  background: #fef3f2;
  border: 1px solid #fecdca;
  padding: 8px 10px;
  border-radius: 9px;
  font-weight: 600;
  font-size: 0.84rem;
}

.muted {
  color: var(--muted);
}

.tiny {
  font-size: 0.74rem;
}

@keyframes panel-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes card-in {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 980px) {
  .page {
    padding: 16px 12px 30px;
    gap: 12px;
  }

  .grid,
  .actions,
  .inline-pairs {
    grid-template-columns: 1fr;
  }

  .topbar,
  .service-hero,
  .field-head,
  .password-actions,
  .panel-head {
    flex-direction: column;
    align-items: flex-start;
  }

  .cred-grid,
  .range-row,
  .output-grid {
    grid-template-columns: 1fr;
  }

  .credentials-card {
    width: 100%;
    min-width: 0;
  }

  .quick-create {
    width: 100%;
  }

  .fold-head {
    flex-wrap: wrap;
    padding-right: 6px;
  }

  .region-field {
    max-width: none;
  }

  .entity-meta-grid {
    grid-template-columns: 1fr;
  }

  .ecs-item-actions button {
    flex-basis: 100%;
    min-width: 0;
  }

  .dialog-shell {
    padding: 12px;
  }

  .password-input-row {
    grid-template-columns: 1fr;
  }

  .meta {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 1200px) {
  .layout {
    grid-template-columns: 1fr;
  }

  .output {
    position: static;
    max-height: none;
    overflow: visible;
    padding-right: 0;
  }
}
</style>
