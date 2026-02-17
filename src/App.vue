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
import CceModulePanel from "./components/cce/CceModulePanel.vue";
import EcsResponsePanel from "./components/ecs/EcsResponsePanel.vue";
import ObsModulePanel from "./components/obs/ObsModulePanel.vue";
import ReloadIconButton from "./components/ReloadIconButton.vue";
import { AUTO_VM_UPDATE_COMMAND, SETUP_GUI_RDP_COMMAND } from "./constants/startupTasks";
import {
  DEFAULT_PLATFORM_DOCKERFILE_PATH,
  buildDockerContainersCommand,
  buildDockerImagesCommand,
  buildDockerSetupCommand,
  buildNixPackagesCommand,
  buildNixSetupCommand,
  buildNixStoreUsageCommand,
  buildNixVersionCommand,
  buildMinikubeNodesCommand,
  buildMinikubePodsCommand,
  buildMinikubeSetupCommand,
  buildMinikubeStatusCommand,
  parseNixPackages,
  parseDockerContainers,
  parseDockerImages,
} from "./utils/platformOps.js";
import type {
  CceCluster,
  CceKubeconfigResult,
  CceClusterListResponse,
  CceNatGateway,
  CceNatGatewayListResponse,
  CceNodePool,
  CceNodePoolListResponse,
  CceOperationResult,
} from "./types/cce";
import type {
  AppLogEntry,
  AutoUpdateProgressInfo,
  CachedEntry,
  CachedResource,
  ConfirmDialogKind,
  ConfirmDialogState,
  CredentialsPayload,
  CreateEcsResult,
  DockerContainerSummary,
  DockerImageSummary,
  DeleteEcsResult,
  EcsListResponse,
  EcsServer,
  EipListResponse,
  EipRecord,
  EvsListResponse,
  EvsVolume,
  FlavorGroup,
  FlavorOption,
  ImageOption,
  LogLevelName,
  LogSource,
  NixPackageSummary,
  PendingStartupTaskCreate,
  ServiceModule,
  PlatformOpsTab,
  SshConnectResult,
  SshDisconnectResult,
  SshExecOneShotResult,
  SshExecResult,
  SshResizeResult,
  SshSendControlResult,
  SshSessionInfo,
  SshStreamEventPayload,
  SshTerminalEntry,
  StartupTaskConfig,
  StopEcsResult,
  StoredServerPassword,
  SubnetOption,
  VpcOption,
} from "./types/ecs";
import type {
  ObsBucket,
  ObsGetObjectResult,
  ObsListBucketsResponse,
  ObsListObjectsResponse,
  ObsObject,
  ObsOperationResult,
} from "./types/obs";

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
const OBS_MAX_KEYS_MIN = 1;
const OBS_MAX_KEYS_MAX = 1000;
const OBS_TOTALS_MAX_PAGES = 10000;
const OBS_PUT_OBJECT_MAX_BYTES = 5 * 1024 * 1024 * 1024;
const OBS_BUCKET_NAME_REGEX = /^[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]$/;
const OBS_BUCKET_STORAGE_CLASSES = ["STANDARD", "WARM", "COLD", "DEEP_ARCHIVE"] as const;
const OBS_BUCKET_ACLS = ["private", "public-read", "public-read-write"] as const;
const CCE_CLUSTER_TYPES = ["VirtualMachine", "BareMetal"] as const;
const CCE_CONTAINER_NETWORK_MODES = ["overlay_l2", "underlay_ipvlan", "vpc-router"] as const;
const CCE_AUTHENTICATION_MODES = ["rbac", "authenticating_proxy"] as const;
const CCE_KUBERNETES_VERSIONS = ["v1.27", "v1.28", "v1.29", "v1.30", "v1.31"] as const;
const CCE_CONTROL_PLANE_FLAVORS = [
  "cce.s1.small",
  "cce.s2.small",
  "cce.s2.medium",
  "cce.s3.large",
] as const;
const CCE_NAT_GATEWAY_SPECS = ["1"] as const;
const CCE_CONTAINER_NETWORK_CIDR_OPTIONS = [
  "172.16.0.0/16",
  "172.17.0.0/16",
  "172.20.0.0/16",
  "10.244.0.0/16",
] as const;
const CCE_SERVICE_CIDR_OPTIONS = ["10.247.0.0/16", "10.96.0.0/12", "10.32.0.0/16"] as const;
const DEFAULT_IMAGE_QUERY = "ubuntu 24";
const DEFAULT_FLAVOR_QUERY = "x1.4u.8g";
const AUTO_UPDATE_SESSION_PREFIX = "auto-update:";
const RDP_STARTUP_USERNAME_PREFIX = "hcforge";
const RDP_STARTUP_USERNAME_SUFFIX_LENGTH = 6;
const STORE_KEY_PASSWORDS_BY_SERVER = "serverPasswords.v1";
const STORE_KEY_STARTUP_TASKS_BY_SERVER = "startupTasks.v1";
const PBKDF2_ITERATIONS = 200_000;
const PBKDF2_SALT_BYTES = 16;
const AES_GCM_IV_BYTES = 12;
const serviceModules: Array<{
  id: ServiceModule;
  title: string;
  chip: string;
  subtitle: string;
}> = [
  {
    id: "ecs",
    title: "Elastic Cloud Server",
    chip: "ECS Module",
    subtitle: "Create and operate compute instances with integrated SSH tools.",
  },
  {
    id: "cce",
    title: "Cloud Container Engine",
    chip: "CCE Module",
    subtitle: "Create and manage Kubernetes clusters, node pools, and CCE jobs.",
  },
  {
    id: "obs",
    title: "Object Storage Service",
    chip: "OBS Module",
    subtitle: "Manage buckets, upload objects, and perform object CRUD in one place.",
  },
];
const USER_DATE_FORMATTER = new Intl.DateTimeFormat(undefined, {
  year: "numeric",
  month: "short",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

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
const activeModule = ref<ServiceModule>("ecs");
const moduleShiftDirection = ref<"next" | "prev">("next");

const useGeneratedPassword = ref(true);
const generatedPassword = ref(generatePassword());
const customPassword = ref("");
const passwordCopyFeedback = ref<string | null>(null);
const quickCopyFeedback = ref<string | null>(null);
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
const loadingResponse = ref(false);
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
const platformPanelOpen = ref(false);
const platformPanelServerId = ref<string | null>(null);
const platformBusyServerId = ref<string | null>(null);
const platformActionLabel = ref<string | null>(null);
const platformError = ref<string | null>(null);
const platformInfo = ref<string | null>(null);
const platformActiveTab = ref<PlatformOpsTab>("docker");
const platformDockerInstallEnabled = ref(true);
const platformDockerImages = ref<DockerImageSummary[]>([]);
const platformDockerContainers = ref<DockerContainerSummary[]>([]);
const platformDockerfileTargetPath = DEFAULT_PLATFORM_DOCKERFILE_PATH;
const platformDockerfileContent = ref("");
const platformMinikubeInstallEnabled = ref(true);
const platformMinikubeEnsureDocker = ref(true);
const platformMinikubeAutoStart = ref(true);
const platformMinikubeProfile = ref("hcforge");
const platformMinikubeDriver = ref<"docker" | "none">("docker");
const platformMinikubeCpus = ref(2);
const platformMinikubeMemoryMb = ref(4096);
const platformMinikubeK8sVersion = ref("");
const platformMinikubeStatus = ref("");
const platformMinikubeNodes = ref("");
const platformMinikubePods = ref("");
const platformNixInstallEnabled = ref(true);
const platformNixEnableFlakes = ref(true);
const platformNixRunGarbageCollect = ref(false);
const platformNixPackagesInput = ref("git ripgrep");
const platformNixVersion = ref("");
const platformNixPackages = ref<NixPackageSummary[]>([]);
const platformNixStoreUsage = ref("");
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

const cceClusterName = ref("");
const cceClusterVersion = ref<(typeof CCE_KUBERNETES_VERSIONS)[number]>("v1.29");
const cceClusterFlavor = ref<(typeof CCE_CONTROL_PLANE_FLAVORS)[number]>("cce.s2.small");
const cceClusterType = ref<(typeof CCE_CLUSTER_TYPES)[number]>("VirtualMachine");
const cceClusterDescription = ref("");
const cceClusterTagEnv = ref("");
const cceClusterVpcId = ref("");
const cceClusterSubnetId = ref("");
const cceClusterContainerNetworkMode = ref<(typeof CCE_CONTAINER_NETWORK_MODES)[number]>(
  "overlay_l2"
);
const cceClusterContainerNetworkCidr = ref<(typeof CCE_CONTAINER_NETWORK_CIDR_OPTIONS)[number]>(
  "172.16.0.0/16"
);
const cceClusterServiceCidr = ref<(typeof CCE_SERVICE_CIDR_OPTIONS)[number]>("10.247.0.0/16");
const cceClusterAuthenticationMode = ref<(typeof CCE_AUTHENTICATION_MODES)[number]>("rbac");
const cceVpcs = ref<VpcOption[]>([]);
const cceSubnets = ref<SubnetOption[]>([]);
const cceLoadingVpcs = ref(false);
const cceLoadingSubnets = ref(false);
const cceCreatingCluster = ref(false);
const cceClusters = ref<CceCluster[]>([]);
const cceLoadingClusters = ref(false);
const cceDeletingClusterId = ref<string | null>(null);
const cceSelectedClusterId = ref("");
const cceNodePools = ref<CceNodePool[]>([]);
const cceLoadingNodePools = ref(false);
const cceLastResult = ref<CceOperationResult | null>(null);
const cceLastJobId = ref("");
const cceJobResult = ref<CceOperationResult | null>(null);
const cceLoadingJob = ref(false);
const cceErrorMsg = ref("");
const cceNatGatewayName = ref("cce-nat-gateway");
const cceNatGatewayDescription = ref("");
const cceNatGatewaySpec = ref<(typeof CCE_NAT_GATEWAY_SPECS)[number]>("1");
const cceNatGateways = ref<CceNatGateway[]>([]);
const cceLoadingNatGateways = ref(false);
const cceCreatingNatGateway = ref(false);
const cceDeletingNatGatewayId = ref<string | null>(null);
const cceAccessEips = ref<EipRecord[]>([]);
const cceLoadingAccessEips = ref(false);
const cceSelectedAccessEipId = ref("");
const cceBindingAccessEip = ref(false);
const cceDownloadingKubeconfig = ref(false);

const obsBucketName = ref("");
const obsDefaultStorageClass = ref<(typeof OBS_BUCKET_STORAGE_CLASSES)[number]>("STANDARD");
const obsBucketAcl = ref<(typeof OBS_BUCKET_ACLS)[number]>("private");
const obsBuckets = ref<ObsBucket[]>([]);
const obsLoadingBuckets = ref(false);
const obsCreatingBucket = ref(false);
const obsDeletingBucket = ref<string | null>(null);
const obsSelectedBucket = ref("");
const obsObjects = ref<ObsObject[]>([]);
const obsLoadingObjects = ref(false);
const obsDeletingObject = ref<string | null>(null);
const obsDownloadingObject = ref<string | null>(null);
const obsUploadingObject = ref(false);
const obsObjectPrefix = ref("");
const obsObjectMarker = ref("");
const obsObjectMaxKeys = ref(200);
const obsUploadObjectKey = ref("");
const obsUploadContentType = ref("");
const obsUploadFile = ref<File | null>(null);
const obsUploadProgress = ref<number | null>(null);
const obsDownloadProgress = ref<number | null>(null);
const obsLastResult = ref<ObsOperationResult | null>(null);
const obsErrorMsg = ref("");
let obsObjectsLoadToken = 0;
const obsBucketTotalSizeBytes = ref<number | null>(null);
const obsBucketTotalObjectCount = ref<number | null>(null);
const obsLoadingBucketTotals = ref(false);
const obsBucketTotalsError = ref<string | null>(null);
let obsBucketTotalsLoadToken = 0;

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
let quickCopyFeedbackTimer: number | null = null;
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
const activeModuleIndex = computed(() =>
  serviceModules.findIndex((module) => module.id === activeModule.value)
);
const activeModuleMeta = computed(
  () => serviceModules[activeModuleIndex.value] ?? serviceModules[0]
);
const moduleTransitionName = computed(() =>
  moduleShiftDirection.value === "next" ? "module-slide-next" : "module-slide-prev"
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

const platformPanelServer = computed(() => {
  const serverId = platformPanelServerId.value;
  if (!serverId) {
    return null;
  }
  return ecses.value.find((item) => item.id === serverId) ?? null;
});

const platformPanelHost = computed(() => {
  const server = platformPanelServer.value;
  if (!server) {
    return null;
  }
  return findSshHostForServer(server);
});

const platformPanelBusy = computed(() => {
  if (!platformPanelServerId.value) {
    return false;
  }
  return platformBusyServerId.value === platformPanelServerId.value;
});

const orderedLogEntries = computed(() =>
  [...logEntries.value].sort((a, b) => b.id - a.id)
);
const obsBucketNameError = computed(() => validateObsBucketName(obsBucketName.value));
const obsCanCreateBucket = computed(
  () => !obsBucketNameError.value && !obsCreatingBucket.value
);
const obsSelectedBucketRecord = computed(
  () => obsBuckets.value.find((bucket) => bucket.name === obsSelectedBucket.value) ?? null
);
const obsCanLoadObjects = computed(
  () => !!obsSelectedBucket.value && !obsLoadingObjects.value
);
const obsCanUploadObject = computed(
  () =>
    !!obsSelectedBucket.value &&
    !!obsUploadFile.value &&
    !!obsUploadObjectKey.value.trim() &&
    !obsUploadingObject.value
);
const obsResolvedUploadContentType = computed(() => {
  const custom = obsUploadContentType.value.trim();
  if (custom) {
    return custom;
  }
  const detected = obsUploadFile.value?.type?.trim();
  if (detected) {
    return detected;
  }
  return "application/octet-stream";
});
const obsSinglePutLimitLabel = computed(() => formatObsObjectSize(OBS_PUT_OBJECT_MAX_BYTES));
const cceCanCreateCluster = computed(
  () =>
    !!cceClusterName.value.trim() &&
    !!cceClusterVersion.value.trim() &&
    !!cceClusterFlavor.value.trim() &&
    !!cceClusterVpcId.value &&
    !!cceClusterSubnetId.value &&
    !!cceClusterContainerNetworkCidr.value.trim() &&
    !!cceClusterServiceCidr.value.trim() &&
    !cceCreatingCluster.value
);
const cceCanCreateNatGateway = computed(
  () =>
    !!cceNatGatewayName.value.trim() &&
    !!cceClusterVpcId.value &&
    !!cceClusterSubnetId.value &&
    !cceCreatingNatGateway.value &&
    cceNatGateways.value.length === 0
);
const cceSelectedCluster = computed(
  () =>
    cceClusters.value.find((cluster) => cceClusterId(cluster) === cceSelectedClusterId.value) ??
    null
);
const cceSelectedClusterExternalIp = computed(() => {
  const cluster = cceSelectedCluster.value;
  if (!cluster) {
    return "";
  }
  const spec = cceAsObject(cluster.spec);
  return cceText(
    spec.clusterExternalIP ??
      spec.clusterExternalIp ??
      spec.cluster_external_i_p ??
      spec.cluster_external_ip
  );
});

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

watch(obsObjectMaxKeys, (value) => {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    obsObjectMaxKeys.value = OBS_MAX_KEYS_MIN;
    return;
  }
  const sanitized = Math.min(OBS_MAX_KEYS_MAX, Math.max(OBS_MAX_KEYS_MIN, Math.trunc(parsed)));
  if (sanitized !== value) {
    obsObjectMaxKeys.value = sanitized;
  }
});

watch(region, async () => {
  stopPolling();
  closePlatformPanel();
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

  obsErrorMsg.value = "";
  obsLastResult.value = null;
  obsSelectedBucket.value = "";
  obsObjects.value = [];
  obsBucketTotalSizeBytes.value = null;
  obsBucketTotalObjectCount.value = null;
  obsBucketTotalsError.value = null;
  obsLoadingBucketTotals.value = false;
  obsBucketTotalsLoadToken += 1;
  cceErrorMsg.value = "";
  cceLastResult.value = null;
  cceJobResult.value = null;
  cceLastJobId.value = "";
  cceClusters.value = [];
  cceSelectedClusterId.value = "";
  cceNodePools.value = [];
  cceVpcs.value = [];
  cceSubnets.value = [];
  cceClusterVpcId.value = "";
  cceClusterSubnetId.value = "";
  cceNatGateways.value = [];
  cceNatGatewayDescription.value = "";
  cceNatGatewayName.value = "cce-nat-gateway";
  cceNatGatewaySpec.value = "1";
  cceLoadingNatGateways.value = false;
  cceCreatingNatGateway.value = false;
  cceDeletingNatGatewayId.value = null;
  cceAccessEips.value = [];
  cceLoadingAccessEips.value = false;
  cceSelectedAccessEipId.value = "";
  cceBindingAccessEip.value = false;
  cceDownloadingKubeconfig.value = false;

  if (activeModule.value === "obs") {
    await loadObsBuckets();
    return;
  }
  if (activeModule.value === "cce") {
    await loadCceVpcs({ log: false });
    await loadCceClusters({ log: false });
    await loadCceAccessEips({ log: false });
    return;
  }

  const hadCache = await hydrateRegionCache();
  if (!hadCache) {
    await loadAll();
  }
});

watch(activeModule, async (nextModule) => {
  if (nextModule === "obs") {
    obsErrorMsg.value = "";
    if (!obsBuckets.value.length) {
      await loadObsBuckets();
    }
    return;
  }
  if (nextModule === "cce") {
    cceErrorMsg.value = "";
    const jobs: Array<Promise<void>> = [];
    if (!cceVpcs.value.length) {
      jobs.push(loadCceVpcs({ log: false }));
    }
    if (!cceClusters.value.length) {
      jobs.push(loadCceClusters({ log: false }));
    }
    if (jobs.length > 0) {
      await Promise.all(jobs);
    }
    if (cceSelectedClusterId.value && !cceAccessEips.value.length) {
      await loadCceAccessEips({ log: false });
    }
    if (cceClusterVpcId.value && cceClusterSubnetId.value && !cceNatGateways.value.length) {
      await loadCceNatGateways({ log: false });
    }
    return;
  }

  const hasEcsData =
    images.value.length > 0 ||
    flavors.value.length > 0 ||
    vpcs.value.length > 0 ||
    ecses.value.length > 0;
  if (!hasEcsData) {
    const hadCache = await hydrateRegionCache();
    if (!hadCache) {
      await loadAll();
    }
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

watch(cceClusterVpcId, async (nextVpc, previousVpc) => {
  if (nextVpc === previousVpc) {
    return;
  }
  cceSubnets.value = [];
  cceClusterSubnetId.value = "";
  cceNatGateways.value = [];
  cceDeletingNatGatewayId.value = null;
  if (!nextVpc) {
    return;
  }
  await loadCceSubnets({ log: false });
});

watch(cceClusterSubnetId, async (nextSubnet, previousSubnet) => {
  if (nextSubnet === previousSubnet) {
    return;
  }
  await loadCceNatGateways({ log: false });
});

watch([cceAccessEips, cceSelectedClusterExternalIp], () => {
  syncCceSelectedAccessEip();
});

watch(ecses, (servers) => {
  cleanupStartupTaskTracking(servers);
  queueStartupTaskCandidates(servers);
  void drainAutoUpdateQueue();

  const activePlatformServerId = platformPanelServerId.value;
  if (activePlatformServerId) {
    const platformServerExists = servers.some((server) => server.id === activePlatformServerId);
    if (!platformServerExists) {
      closePlatformPanel();
    }
  }

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
  if (quickCopyFeedbackTimer !== null) {
    window.clearTimeout(quickCopyFeedbackTimer);
    quickCopyFeedbackTimer = null;
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

function generateRdpUsername(length = RDP_STARTUP_USERNAME_SUFFIX_LENGTH): string {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
  const targetLength = Math.max(4, length);
  let suffix = "";
  for (let index = 0; index < targetLength; index += 1) {
    suffix += chars[randomInt(chars.length)];
  }
  return `${RDP_STARTUP_USERNAME_PREFIX}${suffix}`;
}

function normalizeRdpUsername(value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }
  const normalized = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]/g, "")
    .slice(0, 32);
  if (!normalized) {
    return null;
  }
  if (!normalized.startsWith(RDP_STARTUP_USERNAME_PREFIX)) {
    return null;
  }
  return normalized;
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

function setQuickCopyFeedback(message: string | null) {
  quickCopyFeedback.value = message;

  if (quickCopyFeedbackTimer !== null) {
    window.clearTimeout(quickCopyFeedbackTimer);
    quickCopyFeedbackTimer = null;
  }

  if (message) {
    quickCopyFeedbackTimer = window.setTimeout(() => {
      quickCopyFeedback.value = null;
      quickCopyFeedbackTimer = null;
    }, 2400);
  }
}

async function copyTextToClipboard(value: string, label: string): Promise<boolean> {
  const text = value.trim();
  if (!text) {
    addLog("app", "warn", `Copy requested for empty ${label}.`);
    return false;
  }

  try {
    await writeText(text);
    addLog("app", "info", `Copied ${label} to clipboard via Tauri clipboard plugin.`);
    return true;
  } catch (err) {
    addLog(
      "app",
      "warn",
      `Clipboard plugin write failed while copying ${label}: ${errorToString(err)}`
    );
    if (isTauriRuntime()) {
      return false;
    }
  }

  if (navigator.clipboard?.writeText && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
      addLog("app", "info", `Copied ${label} to clipboard via browser clipboard API.`);
      return true;
    } catch (err) {
      addLog(
        "app",
        "error",
        `Browser clipboard fallback failed while copying ${label}: ${errorToString(err)}`
      );
    }
  }

  addLog("app", "error", `Failed to copy ${label} to clipboard.`);
  return false;
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

  const copied = await copyTextToClipboard(password, "password");
  if (copied) {
    setPasswordFeedback("Password copied.");
    return;
  }

  setPasswordFeedback("Copy failed. Clipboard permission is unavailable.");
  addLog("app", "error", "Failed to copy password to clipboard.");
}

async function copyEipAddress(address: string | null | undefined) {
  const copied = await copyTextToClipboard(address ?? "", "EIP address");
  if (copied) {
    setQuickCopyFeedback("EIP copied.");
  } else {
    setQuickCopyFeedback("EIP copy failed.");
  }
}

function loginUsernameForServer(serverId: string): string {
  return startupTaskRdpUserForServer(serverId) ?? "root";
}

async function copyLoginUsernameForServer(serverId: string) {
  const username = loginUsernameForServer(serverId);
  const copied = await copyTextToClipboard(username, "VM username");
  if (copied) {
    setQuickCopyFeedback("Username copied.");
  } else {
    setQuickCopyFeedback("Username copy failed.");
  }
}

function hasSavedPasswordForServer(serverId: string): boolean {
  return !!serverPasswordFor(serverId);
}

async function copyPasswordForServer(serverId: string) {
  const password = serverPasswordFor(serverId);
  if (!password) {
    setQuickCopyFeedback("No saved password for this ECS.");
    addLog("app", "warn", `Password copy requested without saved password for server ${serverId}.`);
    return;
  }

  const copied = await copyTextToClipboard(password, "VM password");
  if (copied) {
    setQuickCopyFeedback("Password copied.");
  } else {
    setQuickCopyFeedback("Password copy failed.");
  }
}

function cycleServiceModule(direction: "next" | "prev") {
  const total = serviceModules.length;
  if (total < 2) {
    return;
  }
  moduleShiftDirection.value = direction;
  const index = activeModuleIndex.value >= 0 ? activeModuleIndex.value : 0;
  const offset = direction === "next" ? 1 : -1;
  const nextIndex = (index + offset + total) % total;
  activeModule.value = serviceModules[nextIndex].id;
}

function validateObsBucketName(value: string): string | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) {
    return "Bucket name is required.";
  }
  if (normalized.length < 3 || normalized.length > 63) {
    return "Bucket name must be 3-63 characters.";
  }
  if (!OBS_BUCKET_NAME_REGEX.test(normalized)) {
    return "Use lowercase letters, numbers, dots, and dashes only.";
  }
  if (normalized.includes("..")) {
    return "Bucket name cannot contain consecutive dots.";
  }
  if (/^\d+\.\d+\.\d+\.\d+$/.test(normalized)) {
    return "Bucket name cannot be an IPv4 address.";
  }
  return null;
}

function normalizeObsBucketName(value: string): string {
  return value.trim().toLowerCase();
}

function normalizeObsObjectKey(value: string): string {
  return value.trim().replace(/^\/+/, "");
}

function formatObsObjectSize(sizeBytes: number | null | undefined): string {
  const bytes = Number(sizeBytes ?? 0);
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return "0 B";
  }

  const kb = 1024;
  const mb = 1024 * 1024;
  const gb = 1024 * 1024 * 1024;
  const tb = 1024 * 1024 * 1024 * 1024;
  if (bytes < kb) {
    return `${Math.round(bytes)} B`;
  }
  if (bytes < mb) {
    return `${(bytes / kb).toFixed(2)} KB`;
  }
  if (bytes >= tb) {
    return `${(bytes / tb).toFixed(2)} TB`;
  }
  if (bytes >= gb) {
    return `${(bytes / gb).toFixed(2)} GB`;
  }
  return `${(bytes / mb).toFixed(2)} MB`;
}

function cceAsObject(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return {};
  }
  return value as Record<string, unknown>;
}

function cceText(value: unknown, fallback = ""): string {
  if (typeof value !== "string") {
    return fallback;
  }
  return value.trim() || fallback;
}

function cceClusterId(cluster: CceCluster): string {
  const metadata = cceAsObject(cluster.metadata);
  return cceText(metadata.id ?? metadata.uid);
}

function cceClusterDisplayName(cluster: CceCluster): string {
  const metadata = cceAsObject(cluster.metadata);
  return cceText(metadata.name, cceClusterId(cluster) || "unnamed-cluster");
}

function cceNodePoolId(nodePool: CceNodePool): string {
  const metadata = cceAsObject(nodePool.metadata);
  return cceText(metadata.id ?? metadata.uid);
}

function cceNodePoolName(nodePool: CceNodePool): string {
  const metadata = cceAsObject(nodePool.metadata);
  return cceText(metadata.name, cceNodePoolId(nodePool) || "node-pool");
}

function cceNatGatewayId(gateway: CceNatGateway): string {
  return cceText(gateway.id);
}

function cceNatGatewayNameValue(gateway: CceNatGateway): string {
  return cceText(gateway.name, cceNatGatewayId(gateway) || "nat-gateway");
}

function cceEipId(value: EipRecord): string {
  return cceText(value.id);
}

function cceEipAddress(value: EipRecord): string {
  return cceText(value.public_ip_address);
}

function cceResultSummary(resultValue: CceOperationResult): string {
  return `${resultValue.status_code} ${resultValue.status}`;
}

function syncCceSelectedAccessEip() {
  const selectedId = cceSelectedAccessEipId.value.trim();
  if (selectedId && cceAccessEips.value.some((item) => cceEipId(item) === selectedId)) {
    return;
  }
  const clusterIp = cceSelectedClusterExternalIp.value.trim();
  if (clusterIp) {
    const matched = cceAccessEips.value.find((item) => cceEipAddress(item) === clusterIp);
    const matchedId = matched ? cceEipId(matched) : "";
    if (matchedId) {
      cceSelectedAccessEipId.value = matchedId;
      return;
    }
  }
  cceSelectedAccessEipId.value = cceEipId(cceAccessEips.value[0] ?? {});
}

function cceKubeconfigFileName(): string {
  const baseName = cceSelectedCluster.value
    ? cceClusterDisplayName(cceSelectedCluster.value)
    : cceSelectedClusterId.value;
  const sanitized = cceText(baseName, "cce-cluster")
    .toLowerCase()
    .replace(/[^a-z0-9.-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return `${sanitized || "cce-cluster"}-kubeconfig.yaml`;
}

function extractCceClusterId(payload: unknown): string | null {
  if (!payload || typeof payload !== "object") {
    return null;
  }
  const data = payload as Record<string, unknown>;
  const metadata = cceAsObject(data.metadata);
  const metadataId = cceText(metadata.id ?? metadata.uid);
  if (metadataId) {
    return metadataId;
  }
  if (data.cluster && typeof data.cluster === "object") {
    const cluster = data.cluster as Record<string, unknown>;
    const clusterMeta = cceAsObject(cluster.metadata);
    const nestedId = cceText(clusterMeta.id ?? clusterMeta.uid ?? cluster.id);
    if (nestedId) {
      return nestedId;
    }
  }
  const direct = cceText(data.cluster_id ?? data.clusterId ?? data.id);
  return direct || null;
}

function extractCceJobId(payload: unknown): string | null {
  const base = extractJobId(payload);
  if (base) {
    return base;
  }
  if (!payload || typeof payload !== "object") {
    return null;
  }
  const data = payload as Record<string, unknown>;
  const status = cceAsObject(data.status);
  const nested = cceText(
    status.job_id ?? status.jobId ?? status.jobID ?? data.jobId ?? data.task_id ?? data.taskId
  );
  return nested || null;
}

async function loadCceVpcs(options: { log?: boolean } = {}) {
  const shouldLog = options.log ?? true;
  cceLoadingVpcs.value = true;
  if (shouldLog) {
    addLog("app", "info", `Listing CCE VPCs for region ${region.value}.`);
  }
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };
    if (credentials) {
      args.credentials = credentials;
    }
    const data = await invoke<VpcOption[]>("list_vpcs", args);
    cceVpcs.value = data;
    const knownVpc = cceVpcs.value.some((item) => item.id === cceClusterVpcId.value)
      ? cceClusterVpcId.value
      : "";
    const nextVpc = knownVpc || cceVpcs.value[0]?.id || "";
    if (nextVpc !== cceClusterVpcId.value) {
      cceClusterVpcId.value = nextVpc;
    } else if (nextVpc && !cceSubnets.value.length) {
      await loadCceSubnets({ log: false });
    }
  } catch (err) {
    const message = `Failed to load CCE VPCs: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceLoadingVpcs.value = false;
  }
}

async function loadCceSubnets(options: { log?: boolean } = {}) {
  if (!cceClusterVpcId.value) {
    cceSubnets.value = [];
    cceClusterSubnetId.value = "";
    return;
  }
  const shouldLog = options.log ?? true;
  cceLoadingSubnets.value = true;
  if (shouldLog) {
    addLog(
      "app",
      "info",
      `Listing CCE subnets for VPC ${cceClusterVpcId.value} in ${region.value}.`
    );
  }
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      region: region.value,
      vpcId: cceClusterVpcId.value,
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const data = await invoke<SubnetOption[]>("list_subnets", args);
    cceSubnets.value = data;
    if (!cceSubnets.value.some((item) => item.id === cceClusterSubnetId.value)) {
      cceClusterSubnetId.value = cceSubnets.value[0]?.id ?? "";
    }
  } catch (err) {
    const message = `Failed to load CCE subnets: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceLoadingSubnets.value = false;
  }
}

async function loadCceClusters(options: { log?: boolean } = {}) {
  const shouldLog = options.log ?? true;
  cceLoadingClusters.value = true;
  cceErrorMsg.value = "";
  if (shouldLog) {
    addLog("app", "info", `Listing CCE clusters for region ${region.value}.`);
  }
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };
    if (credentials) {
      args.credentials = credentials;
    }
    const response = await invoke<CceClusterListResponse>("list_cce_clusters", args);
    cceClusters.value = [...(response.items ?? [])].sort((left, right) =>
      cceClusterDisplayName(left).localeCompare(cceClusterDisplayName(right))
    );
    if (
      cceSelectedClusterId.value &&
      !cceClusters.value.some((cluster) => cceClusterId(cluster) === cceSelectedClusterId.value)
    ) {
      cceSelectedClusterId.value = "";
      cceNodePools.value = [];
      cceAccessEips.value = [];
      cceSelectedAccessEipId.value = "";
    } else if (cceSelectedClusterId.value) {
      syncCceSelectedAccessEip();
    }
    if (shouldLog) {
      addLog(
        "app",
        "info",
        `Loaded ${cceClusters.value.length} CCE cluster(s) for region ${region.value}.`
      );
    }
  } catch (err) {
    const message = `Failed to load CCE clusters: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceLoadingClusters.value = false;
  }
}

async function selectCceCluster(clusterId: string) {
  const normalizedClusterId = clusterId.trim();
  if (!normalizedClusterId || cceSelectedClusterId.value === normalizedClusterId) {
    return;
  }
  cceSelectedClusterId.value = normalizedClusterId;
  cceNodePools.value = [];
  cceAccessEips.value = [];
  cceSelectedAccessEipId.value = "";
  await Promise.all([loadCceNodePools({ log: false }), loadCceAccessEips({ log: false })]);
}

async function loadCceNodePools(options: { log?: boolean } = {}) {
  const clusterId = cceSelectedClusterId.value.trim();
  if (!clusterId) {
    cceNodePools.value = [];
    return;
  }
  const shouldLog = options.log ?? true;
  cceLoadingNodePools.value = true;
  cceErrorMsg.value = "";
  if (shouldLog) {
    addLog("app", "info", `Listing CCE node pools for cluster ${clusterId}.`);
  }
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        clusterId,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const response = await invoke<CceNodePoolListResponse>("list_cce_node_pools", args);
    cceNodePools.value = [...(response.items ?? [])].sort((left, right) =>
      cceNodePoolName(left).localeCompare(cceNodePoolName(right))
    );
    if (shouldLog) {
      addLog(
        "app",
        "info",
        `Loaded ${cceNodePools.value.length} CCE node pool(s) for cluster ${clusterId}.`
      );
    }
  } catch (err) {
    const message = `Failed to load CCE node pools: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceLoadingNodePools.value = false;
  }
}

async function loadCceJob(jobIdInput?: string, options: { log?: boolean } = {}) {
  const targetJobId = (jobIdInput ?? cceLastJobId.value).trim();
  if (!targetJobId) {
    return;
  }
  const shouldLog = options.log ?? true;
  cceLoadingJob.value = true;
  cceErrorMsg.value = "";
  if (shouldLog) {
    addLog("app", "info", `Querying CCE job ${targetJobId} in ${region.value}.`);
  }
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        jobId: targetJobId,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    cceJobResult.value = await invoke<CceOperationResult>("get_cce_job", args);
    if (shouldLog) {
      addLog(
        "app",
        "info",
        `CCE job ${targetJobId}: ${cceResultSummary(cceJobResult.value)}`
      );
    }
  } catch (err) {
    const message = `Failed to query CCE job ${targetJobId}: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceLoadingJob.value = false;
  }
}

async function loadCceNatGateways(options: { log?: boolean } = {}) {
  const vpcId = cceClusterVpcId.value.trim();
  const subnetId = cceClusterSubnetId.value.trim();
  if (!vpcId || !subnetId) {
    cceNatGateways.value = [];
    return;
  }

  const shouldLog = options.log ?? true;
  cceLoadingNatGateways.value = true;
  if (shouldLog) {
    addLog(
      "app",
      "info",
      `Listing NAT gateways for CCE network vpc=${vpcId} subnet=${subnetId}.`
    );
  }
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        vpcId,
        subnetId,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const response = await invoke<CceNatGatewayListResponse>("list_cce_nat_gateways", args);
    cceNatGateways.value = [...(response.nat_gateways ?? [])].sort((left, right) =>
      cceNatGatewayNameValue(left).localeCompare(cceNatGatewayNameValue(right))
    );
    if (shouldLog) {
      addLog(
        "app",
        "info",
        `Loaded ${cceNatGateways.value.length} NAT gateway(s) for selected CCE network.`
      );
    }
  } catch (err) {
    const message = `Failed to load CCE NAT gateways: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceLoadingNatGateways.value = false;
  }
}

async function loadCceAccessEips(options: { log?: boolean } = {}) {
  const clusterId = cceSelectedClusterId.value.trim();
  if (!clusterId) {
    cceAccessEips.value = [];
    cceSelectedAccessEipId.value = "";
    return;
  }

  const shouldLog = options.log ?? true;
  cceLoadingAccessEips.value = true;
  if (shouldLog) {
    addLog("app", "info", `Listing EIPs for CCE cluster access in ${region.value}.`);
  }
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      region: region.value,
      params: {
        limit: 1000,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const response = await invoke<EipListResponse>("list_eips", args);
    cceAccessEips.value = [...(response.publicips ?? [])]
      .filter((item) => !!cceEipId(item))
      .sort((left, right) => cceEipAddress(left).localeCompare(cceEipAddress(right)));
    syncCceSelectedAccessEip();
    if (shouldLog) {
      addLog("app", "info", `Loaded ${cceAccessEips.value.length} EIP record(s) for CCE access.`);
    }
  } catch (err) {
    const message = `Failed to load CCE access EIPs: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceLoadingAccessEips.value = false;
  }
}

async function bindCceClusterApiEip() {
  const clusterId = cceSelectedClusterId.value.trim();
  if (!clusterId) {
    cceErrorMsg.value = "Select a CCE cluster before binding an API EIP.";
    return;
  }
  const selectedId = cceSelectedAccessEipId.value.trim();
  const selected = cceAccessEips.value.find((item) => cceEipId(item) === selectedId);
  const eipAddress = selected ? cceEipAddress(selected) : "";
  if (!eipAddress) {
    cceErrorMsg.value = "Select a valid EIP with a public address.";
    return;
  }

  cceBindingAccessEip.value = true;
  cceErrorMsg.value = "";
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        clusterId,
        eipAddress,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const resultValue = await invoke<CceOperationResult>("bind_cce_cluster_api_eip", args);
    cceLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Bind CCE API EIP ${eipAddress} to cluster ${clusterId}: ${cceResultSummary(resultValue)}`
    );

    const payload = safeJsonParse(resultValue.body);
    const jobId = extractCceJobId(payload);
    if (jobId) {
      cceLastJobId.value = jobId;
      await loadCceJob(jobId, { log: false });
    }
    if (success) {
      await Promise.all([loadCceClusters({ log: false }), loadCceAccessEips({ log: false })]);
      await sendUserNotification(
        "CCE API EIP bind submitted",
        `Cluster API endpoint is updating to ${eipAddress}.`
      );
    }
  } catch (err) {
    const message = `Failed to bind CCE API EIP: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceBindingAccessEip.value = false;
  }
}

async function downloadCceKubeconfig() {
  const clusterId = cceSelectedClusterId.value.trim();
  if (!clusterId) {
    cceErrorMsg.value = "Select a CCE cluster before requesting kubeconfig.";
    return;
  }

  cceDownloadingKubeconfig.value = true;
  cceErrorMsg.value = "";
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        clusterId,
        context: "external",
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const resultValue = await invoke<CceKubeconfigResult>("get_cce_cluster_kubeconfig", args);
    cceLastResult.value = {
      status: resultValue.status,
      status_code: resultValue.status_code,
      body: resultValue.body,
    };

    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    if (!success) {
      const message = `Failed to request CCE kubeconfig: ${resultValue.status_code} ${resultValue.status}`;
      cceErrorMsg.value = message;
      addLog("app", "error", message);
      return;
    }

    const kubeconfig = cceText(resultValue.kubeconfig);
    if (!kubeconfig) {
      const message =
        "CCE kubeconfig response did not include a kubeconfig payload. Verify external access is enabled.";
      cceErrorMsg.value = message;
      addLog("app", "warn", message);
      return;
    }

    const blob = new Blob([kubeconfig], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = cceKubeconfigFileName();
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 0);

    addLog("app", "info", `Downloaded CCE kubeconfig for cluster ${clusterId}.`);
    setQuickCopyFeedback(`Download started: ${cceKubeconfigFileName()}.`);
  } catch (err) {
    const message = `Failed to download CCE kubeconfig: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceDownloadingKubeconfig.value = false;
  }
}

async function createCceNatGateway() {
  const name = cceNatGatewayName.value.trim();
  const vpcId = cceClusterVpcId.value.trim();
  const subnetId = cceClusterSubnetId.value.trim();
  if (!name) {
    cceErrorMsg.value = "NAT gateway name is required.";
    return;
  }
  if (!vpcId || !subnetId) {
    cceErrorMsg.value = "Select both CCE VPC and subnet before creating a NAT gateway.";
    return;
  }
  if (cceNatGateways.value.length > 0) {
    cceErrorMsg.value = "A NAT gateway already exists for this selected CCE network.";
    return;
  }

  cceCreatingNatGateway.value = true;
  cceErrorMsg.value = "";
  cceLastResult.value = null;
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        name,
        vpcId,
        subnetId,
        description: cceNatGatewayDescription.value.trim() || null,
        spec: cceNatGatewaySpec.value,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const resultValue = await invoke<CceOperationResult>("create_cce_nat_gateway", args);
    cceLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Create CCE NAT gateway ${name} (auto EIP + SNAT): ${cceResultSummary(resultValue)}`
    );
    if (success) {
      await Promise.all([loadCceNatGateways({ log: false }), loadCceAccessEips({ log: false })]);
      await sendUserNotification(
        "CCE NAT bootstrap accepted",
        `${name} request submitted with EIP + SNAT setup in ${region.value}.`
      );
    }
  } catch (err) {
    const message = `Failed to create CCE NAT gateway: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceCreatingNatGateway.value = false;
  }
}

async function deleteCceNatGateway(gateway: CceNatGateway) {
  const natGatewayId = cceNatGatewayId(gateway);
  if (!natGatewayId) {
    return;
  }
  const gatewayName = cceNatGatewayNameValue(gateway);
  const confirmed = await showConfirmDialog(
    `Delete NAT gateway "${gatewayName}" (${natGatewayId})? This can interrupt egress routing.`,
    {
      title: "Delete CCE NAT Gateway",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    }
  );
  if (!confirmed) {
    return;
  }

  cceDeletingNatGatewayId.value = natGatewayId;
  cceErrorMsg.value = "";
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        natGatewayId,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const resultValue = await invoke<CceOperationResult>("delete_cce_nat_gateway", args);
    cceLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Delete CCE NAT gateway ${gatewayName} (${natGatewayId}): ${cceResultSummary(resultValue)}`
    );
    if (success) {
      await loadCceNatGateways({ log: false });
    }
  } catch (err) {
    const message = `Failed to delete CCE NAT gateway: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceDeletingNatGatewayId.value = null;
  }
}

async function createCceCluster() {
  const clusterName = cceClusterName.value.trim();
  if (!clusterName) {
    cceErrorMsg.value = "CCE cluster name is required.";
    return;
  }
  if (!cceClusterVpcId.value || !cceClusterSubnetId.value) {
    cceErrorMsg.value = "Select both VPC and subnet before creating a cluster.";
    return;
  }

  cceCreatingCluster.value = true;
  cceErrorMsg.value = "";
  cceLastResult.value = null;
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        name: clusterName,
        flavor: cceClusterFlavor.value.trim(),
        version: cceClusterVersion.value.trim(),
        vpcId: cceClusterVpcId.value,
        subnetId: cceClusterSubnetId.value,
        description: cceClusterDescription.value.trim() || null,
        clusterType: cceClusterType.value,
        containerNetworkMode: cceClusterContainerNetworkMode.value,
        containerNetworkCidr: cceClusterContainerNetworkCidr.value.trim(),
        kubernetesSvcIpRange: cceClusterServiceCidr.value.trim(),
        authenticationMode: cceClusterAuthenticationMode.value,
        clusterTagEnv: cceClusterTagEnv.value.trim() || null,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }

    const resultValue = await invoke<CceOperationResult>("create_cce_cluster", args);
    cceLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Create CCE cluster ${clusterName}: ${cceResultSummary(resultValue)}`
    );

    const payload = safeJsonParse(resultValue.body);
    const jobId = extractCceJobId(payload);
    if (jobId) {
      cceLastJobId.value = jobId;
      await loadCceJob(jobId, { log: false });
    }

    if (success) {
      await loadCceClusters({ log: false });
      const createdClusterId = extractCceClusterId(payload);
      const matchedCluster =
        cceClusters.value.find((cluster) => cceClusterId(cluster) === createdClusterId) ??
        cceClusters.value.find((cluster) => cceClusterDisplayName(cluster) === clusterName);
      const targetClusterId = matchedCluster ? cceClusterId(matchedCluster) : "";
      if (targetClusterId) {
        await selectCceCluster(targetClusterId);
      }
      await sendUserNotification(
        "CCE cluster create accepted",
        `${clusterName} request submitted in ${region.value}.`
      );
    }
  } catch (err) {
    const message = `Failed to create CCE cluster: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceCreatingCluster.value = false;
  }
}

async function deleteCceCluster(cluster: CceCluster) {
  const clusterId = cceClusterId(cluster);
  if (!clusterId) {
    return;
  }
  const clusterName = cceClusterDisplayName(cluster);
  const confirmed = await showConfirmDialog(
    `Delete CCE cluster "${clusterName}" (${clusterId})? This removes cluster resources managed by CCE.`,
    {
      title: "Delete CCE Cluster",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    }
  );
  if (!confirmed) {
    return;
  }

  cceDeletingClusterId.value = clusterId;
  cceErrorMsg.value = "";
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        clusterId,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }
    const resultValue = await invoke<CceOperationResult>("delete_cce_cluster", args);
    cceLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Delete CCE cluster ${clusterName} (${clusterId}): ${cceResultSummary(resultValue)}`
    );

    const payload = safeJsonParse(resultValue.body);
    const jobId = extractCceJobId(payload);
    if (jobId) {
      cceLastJobId.value = jobId;
      await loadCceJob(jobId, { log: false });
    }

    if (success) {
      await loadCceClusters({ log: false });
      if (cceSelectedClusterId.value === clusterId) {
        cceSelectedClusterId.value = "";
        cceNodePools.value = [];
      }
    }
  } catch (err) {
    const message = `Failed to delete CCE cluster: ${errorToString(err)}`;
    cceErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    cceDeletingClusterId.value = null;
  }
}

function obsDownloadFileName(objectKey: string): string {
  const key = normalizeObsObjectKey(objectKey);
  if (!key) {
    return "obs-object";
  }
  const segments = key.split("/");
  const baseName = segments[segments.length - 1]?.trim() || key;
  const sanitized = baseName.replace(/[\\/:*?"<>|]/g, "_");
  return sanitized || "obs-object";
}

function obsResultSummary(resultValue: ObsOperationResult): string {
  return `${resultValue.status_code} ${resultValue.status}`;
}

function onObsUploadFileChange(inputValue: Event | File | null) {
  let file: File | null = null;
  if (inputValue instanceof File || inputValue === null) {
    file = inputValue;
  } else {
    const input = inputValue.target as HTMLInputElement | null;
    file = input?.files?.[0] ?? null;
  }
  obsUploadFile.value = file;
  if (file && !obsUploadObjectKey.value.trim()) {
    obsUploadObjectKey.value = file.name;
  }
}

function clearObsUploadSelection() {
  obsUploadFile.value = null;
  obsUploadObjectKey.value = "";
  obsUploadContentType.value = "";
}

function resetObsBucketTotals() {
  obsBucketTotalSizeBytes.value = null;
  obsBucketTotalObjectCount.value = null;
  obsBucketTotalsError.value = null;
}

function obsRegionForBucket(bucketName: string): string {
  const normalized = bucketName.trim();
  if (!normalized) {
    return region.value;
  }
  const bucketRegion = obsBuckets.value
    .find((bucket) => bucket.name === normalized)
    ?.location?.trim();
  return bucketRegion || region.value;
}

async function encodeFileToBase64(
  file: File,
  onProgress?: (percent: number) => void
): Promise<string> {
  return await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => {
      reject(new Error("Failed to read file for upload."));
    };
    reader.onprogress = (event) => {
      if (!onProgress || !event.lengthComputable || event.total <= 0) {
        return;
      }
      onProgress(Math.min(100, Math.max(0, Math.round((event.loaded / event.total) * 100))));
    };
    reader.onload = () => {
      const result = typeof reader.result === "string" ? reader.result : "";
      const marker = "base64,";
      const markerIndex = result.indexOf(marker);
      if (markerIndex < 0) {
        reject(new Error("Could not encode file to base64."));
        return;
      }
      resolve(result.slice(markerIndex + marker.length));
    };
    reader.readAsDataURL(file);
  });
}

async function loadObsBuckets(options: { log?: boolean } = {}) {
  const shouldLog = options.log ?? true;
  obsLoadingBuckets.value = true;
  obsErrorMsg.value = "";
  if (shouldLog) {
    addLog("app", "info", `Listing OBS buckets for region ${region.value}.`);
  }

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };
    if (credentials) {
      args.credentials = credentials;
    }
    const data = await invoke<ObsListBucketsResponse>("list_obs_buckets", args);
    obsBuckets.value = (data.buckets ?? []).sort((a, b) => a.name.localeCompare(b.name));
    if (
      obsSelectedBucket.value &&
      !obsBuckets.value.some((bucket) => bucket.name === obsSelectedBucket.value)
    ) {
      obsObjectsLoadToken += 1;
      obsBucketTotalsLoadToken += 1;
      obsSelectedBucket.value = "";
      obsObjects.value = [];
      obsObjectMarker.value = "";
      obsLoadingObjects.value = false;
      obsLoadingBucketTotals.value = false;
      resetObsBucketTotals();
    }
    if (shouldLog) {
      addLog(
        "app",
        "info",
        `Loaded ${obsBuckets.value.length} OBS buckets for region ${region.value}.`
      );
    }
  } catch (err) {
    const message = `Failed to load OBS buckets: ${errorToString(err)}`;
    obsErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    obsLoadingBuckets.value = false;
  }
}

async function selectObsBucket(bucketName: string) {
  if (!bucketName || obsSelectedBucket.value === bucketName) {
    return;
  }
  obsSelectedBucket.value = bucketName;
  obsObjectMarker.value = "";
  obsObjects.value = [];
  await Promise.all([loadObsObjects(), loadObsBucketTotals()]);
}

async function loadObsObjects(options: { log?: boolean } = {}) {
  if (!obsSelectedBucket.value) {
    obsObjects.value = [];
    obsObjectMarker.value = "";
    resetObsBucketTotals();
    return;
  }
  const requestedBucket = obsSelectedBucket.value;
  const bucketRegion = obsRegionForBucket(requestedBucket);
  const token = ++obsObjectsLoadToken;
  const shouldLog = options.log ?? true;
  obsLoadingObjects.value = true;
  obsErrorMsg.value = "";
  obsObjects.value = [];
  if (shouldLog) {
    addLog(
      "app",
      "info",
      `Listing OBS objects for bucket ${requestedBucket} in ${bucketRegion}.`
    );
  }

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: bucketRegion,
        bucketName: requestedBucket,
        prefix: obsObjectPrefix.value.trim() || null,
        marker: obsObjectMarker.value.trim() || null,
        maxKeys: obsObjectMaxKeys.value,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }

    const response = await invoke<ObsListObjectsResponse>("list_obs_objects", args);
    if (token !== obsObjectsLoadToken || requestedBucket !== obsSelectedBucket.value) {
      return;
    }
    obsObjects.value = response.objects ?? [];
    if (response.next_marker) {
      obsObjectMarker.value = response.next_marker;
    } else if (!(response.is_truncated ?? false)) {
      obsObjectMarker.value = "";
    }
    if (shouldLog) {
      addLog(
        "app",
        "info",
        `Loaded ${obsObjects.value.length} objects from bucket ${requestedBucket} in ${bucketRegion}.`
      );
    }
  } catch (err) {
    if (token !== obsObjectsLoadToken || requestedBucket !== obsSelectedBucket.value) {
      return;
    }
    obsObjects.value = [];
    obsObjectMarker.value = "";
    const message = `Failed to load OBS objects: ${errorToString(err)}`;
    obsErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    if (token === obsObjectsLoadToken) {
      obsLoadingObjects.value = false;
    }
  }
}

async function loadObsBucketTotals(options: { log?: boolean } = {}) {
  if (!obsSelectedBucket.value) {
    resetObsBucketTotals();
    return;
  }

  const requestedBucket = obsSelectedBucket.value;
  const bucketRegion = obsRegionForBucket(requestedBucket);
  const token = ++obsBucketTotalsLoadToken;
  const shouldLog = options.log ?? false;
  obsLoadingBucketTotals.value = true;
  obsBucketTotalsError.value = null;
  obsBucketTotalSizeBytes.value = null;
  obsBucketTotalObjectCount.value = null;

  if (shouldLog) {
    addLog(
      "app",
      "info",
      `Calculating OBS bucket usage for ${requestedBucket} in ${bucketRegion}.`
    );
  }

  let marker: string | null = null;
  let totalSizeBytes = 0;
  let totalObjectCount = 0;
  let pageCount = 0;
  const seenMarkers = new Set<string>();

  try {
    const credentials = buildCredentialsPayload();
    while (true) {
      if (token !== obsBucketTotalsLoadToken || requestedBucket !== obsSelectedBucket.value) {
        return;
      }

      pageCount += 1;
      if (pageCount > OBS_TOTALS_MAX_PAGES) {
        throw new Error(
          `Stopped usage scan after ${OBS_TOTALS_MAX_PAGES} pages to avoid an infinite loop.`
        );
      }

      const args: Record<string, unknown> = {
        params: {
          region: bucketRegion,
          bucketName: requestedBucket,
          maxKeys: OBS_MAX_KEYS_MAX,
          marker,
          prefix: null,
        },
      };
      if (credentials) {
        args.credentials = credentials;
      }

      const response = await invoke<ObsListObjectsResponse>("list_obs_objects", args);
      if (token !== obsBucketTotalsLoadToken || requestedBucket !== obsSelectedBucket.value) {
        return;
      }

      const pageObjects = response.objects ?? [];
      totalObjectCount += pageObjects.length;
      for (const object of pageObjects) {
        const objectSize = Number(object.size ?? 0);
        if (Number.isFinite(objectSize) && objectSize > 0) {
          totalSizeBytes += objectSize;
        }
      }

      const nextMarker = response.next_marker?.trim() ?? "";
      if (!(response.is_truncated ?? false) || !nextMarker) {
        break;
      }
      if (seenMarkers.has(nextMarker)) {
        throw new Error(`OBS pagination repeated marker "${nextMarker}".`);
      }
      seenMarkers.add(nextMarker);
      marker = nextMarker;
    }

    obsBucketTotalSizeBytes.value = totalSizeBytes;
    obsBucketTotalObjectCount.value = totalObjectCount;
    if (shouldLog) {
      addLog(
        "app",
        "info",
        `OBS usage for ${requestedBucket}: ${formatObsObjectSize(totalSizeBytes)} across ${totalObjectCount} object(s).`
      );
    }
  } catch (err) {
    if (token !== obsBucketTotalsLoadToken || requestedBucket !== obsSelectedBucket.value) {
      return;
    }
    obsBucketTotalSizeBytes.value = null;
    obsBucketTotalObjectCount.value = null;
    const message = `Failed to calculate OBS bucket usage: ${errorToString(err)}`;
    obsBucketTotalsError.value = message;
    addLog("app", "error", message);
  } finally {
    if (token === obsBucketTotalsLoadToken) {
      obsLoadingBucketTotals.value = false;
    }
  }
}

async function reloadObsObjectsAndTotals() {
  await Promise.all([loadObsObjects(), loadObsBucketTotals()]);
}

async function createObsBucket() {
  const bucketName = normalizeObsBucketName(obsBucketName.value);
  const bucketError = validateObsBucketName(bucketName);
  if (bucketError) {
    obsErrorMsg.value = bucketError;
    addLog("app", "warn", bucketError);
    return;
  }

  obsCreatingBucket.value = true;
  obsErrorMsg.value = "";
  obsLastResult.value = null;
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: region.value,
        bucketName,
        defaultStorageClass: obsDefaultStorageClass.value,
        acl: obsBucketAcl.value,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }

    const resultValue = await invoke<ObsOperationResult>("create_obs_bucket", args);
    obsLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Create OBS bucket ${bucketName}: ${obsResultSummary(resultValue)}`
    );
    if (success) {
      obsBucketName.value = "";
      await loadObsBuckets({ log: false });
      await selectObsBucket(bucketName);
      await sendUserNotification(
        "OBS bucket created",
        `${bucketName} created in ${region.value}.`
      );
    }
  } catch (err) {
    const message = `Failed to create OBS bucket: ${errorToString(err)}`;
    obsErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    obsCreatingBucket.value = false;
  }
}

async function deleteObsBucket(bucket: ObsBucket) {
  const bucketName = bucket.name;
  if (!bucketName) {
    return;
  }
  const bucketRegion = bucket.location?.trim() || obsRegionForBucket(bucketName);
  const confirmed = await showConfirmDialog(
    `Delete bucket "${bucketName}"? The bucket must be empty before OBS accepts deletion.`,
    {
      title: "Delete OBS Bucket",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    }
  );
  if (!confirmed) {
    return;
  }

  obsDeletingBucket.value = bucketName;
  obsErrorMsg.value = "";
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: { region: bucketRegion, bucketName },
    };
    if (credentials) {
      args.credentials = credentials;
    }

    const resultValue = await invoke<ObsOperationResult>("delete_obs_bucket", args);
    obsLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Delete OBS bucket ${bucketName} (${bucketRegion}): ${obsResultSummary(resultValue)}`
    );
    if (success) {
      await loadObsBuckets({ log: false });
      if (obsSelectedBucket.value === bucketName) {
        obsObjectsLoadToken += 1;
        obsBucketTotalsLoadToken += 1;
        obsSelectedBucket.value = "";
        obsObjects.value = [];
        obsObjectMarker.value = "";
        obsLoadingObjects.value = false;
        obsLoadingBucketTotals.value = false;
        resetObsBucketTotals();
      }
    }
  } catch (err) {
    const message = `Failed to delete OBS bucket: ${errorToString(err)}`;
    obsErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    obsDeletingBucket.value = null;
  }
}

async function uploadObsObject() {
  const bucketName = obsSelectedBucket.value;
  const bucketRegion = obsRegionForBucket(bucketName);
  const file = obsUploadFile.value;
  const objectKey = normalizeObsObjectKey(obsUploadObjectKey.value);
  if (!bucketName) {
    obsErrorMsg.value = "Select a bucket before uploading.";
    return;
  }
  if (!file) {
    obsErrorMsg.value = "Select a file to upload.";
    return;
  }
  if (!objectKey) {
    obsErrorMsg.value = "Object key is required.";
    return;
  }
  if (file.size > OBS_PUT_OBJECT_MAX_BYTES) {
    obsErrorMsg.value =
      `File is too large for OBS single PUT upload (${formatObsObjectSize(file.size)}). ` +
      `Maximum supported by PutObject is ${obsSinglePutLimitLabel.value}.`;
    return;
  }

  obsUploadingObject.value = true;
  obsUploadProgress.value = 0;
  obsErrorMsg.value = "";
  try {
    const contentBase64 = await encodeFileToBase64(file, (percent) => {
      const mapped = Math.round(percent * 0.85);
      obsUploadProgress.value = Math.min(85, Math.max(obsUploadProgress.value ?? 0, mapped));
    });
    obsUploadProgress.value = 92;
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: bucketRegion,
        bucketName,
        objectKey,
        contentBase64,
        contentType: obsResolvedUploadContentType.value,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }

    obsUploadProgress.value = 96;
    const resultValue = await invoke<ObsOperationResult>("put_obs_object", args);
    obsLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Upload OBS object ${objectKey}: ${obsResultSummary(resultValue)}`
    );
    if (success) {
      obsUploadProgress.value = 100;
      clearObsUploadSelection();
      await Promise.all([loadObsObjects({ log: false }), loadObsBucketTotals({ log: false })]);
      window.setTimeout(() => {
        obsUploadProgress.value = null;
      }, 1200);
    } else {
      obsUploadProgress.value = null;
    }
  } catch (err) {
    const message = `Failed to upload OBS object: ${errorToString(err)}`;
    obsErrorMsg.value = message;
    addLog("app", "error", message);
    obsUploadProgress.value = null;
  } finally {
    obsUploadingObject.value = false;
  }
}

async function downloadObsObject(objectKeyValue: string) {
  const bucketName = obsSelectedBucket.value;
  const bucketRegion = obsRegionForBucket(bucketName);
  const objectKey = normalizeObsObjectKey(objectKeyValue);
  if (!bucketName || !objectKey) {
    return;
  }

  obsDownloadingObject.value = objectKey;
  obsDownloadProgress.value = 5;
  obsErrorMsg.value = "";
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: bucketRegion,
        bucketName,
        objectKey,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }

    obsDownloadProgress.value = 65;
    const response = await invoke<ObsGetObjectResult>("get_obs_object", args);
    const success = response.status_code >= 200 && response.status_code < 300;
    if (!success || response.content_base64 == null) {
      const summary = response.body?.trim();
      const message = summary
        ? `Failed to download OBS object: ${response.status_code} ${response.status} (${summary})`
        : `Failed to download OBS object: ${response.status_code} ${response.status}`;
      obsErrorMsg.value = message;
      addLog("app", "error", message);
      obsDownloadProgress.value = null;
      return;
    }

    obsDownloadProgress.value = 85;
    const bytes = base64ToBytes(response.content_base64);
    const blob = new Blob([bytes], {
      type: response.content_type?.trim() || "application/octet-stream",
    });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = obsDownloadFileName(objectKey);
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 0);
    obsDownloadProgress.value = 100;
    window.setTimeout(() => {
      obsDownloadProgress.value = null;
    }, 1200);

    addLog(
      "app",
      "info",
      `Downloaded OBS object ${objectKey} from bucket ${bucketName}.`
    );
    setQuickCopyFeedback(`Download started: ${obsDownloadFileName(objectKey)}.`);
  } catch (err) {
    const message = `Failed to download OBS object: ${errorToString(err)}`;
    obsErrorMsg.value = message;
    addLog("app", "error", message);
    obsDownloadProgress.value = null;
  } finally {
    obsDownloadingObject.value = null;
  }
}

async function deleteObsObject(objectKeyValue: string) {
  const bucketName = obsSelectedBucket.value;
  const bucketRegion = obsRegionForBucket(bucketName);
  const objectKey = normalizeObsObjectKey(objectKeyValue);
  if (!bucketName || !objectKey) {
    return;
  }
  const confirmed = await showConfirmDialog(`Delete object "${objectKey}" from ${bucketName}?`, {
    title: "Delete OBS Object",
    kind: "warning",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!confirmed) {
    return;
  }

  obsDeletingObject.value = objectKey;
  obsErrorMsg.value = "";
  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = {
      params: {
        region: bucketRegion,
        bucketName,
        objectKey,
      },
    };
    if (credentials) {
      args.credentials = credentials;
    }

    const resultValue = await invoke<ObsOperationResult>("delete_obs_object", args);
    obsLastResult.value = resultValue;
    const success = resultValue.status_code >= 200 && resultValue.status_code < 300;
    addLog(
      "app",
      success ? "info" : "warn",
      `Delete OBS object ${objectKey}: ${obsResultSummary(resultValue)}`
    );
    if (success) {
      await Promise.all([loadObsObjects({ log: false }), loadObsBucketTotals({ log: false })]);
    }
  } catch (err) {
    const message = `Failed to delete OBS object: ${errorToString(err)}`;
    obsErrorMsg.value = message;
    addLog("app", "error", message);
  } finally {
    obsDeletingObject.value = null;
  }
}

async function copyObsBucketName(bucketName: string) {
  const copied = await copyTextToClipboard(bucketName, "OBS bucket name");
  setQuickCopyFeedback(copied ? "Bucket name copied." : "Bucket name copy failed.");
}

async function copyObsObjectKey(objectKey: string) {
  const copied = await copyTextToClipboard(objectKey, "OBS object key");
  setQuickCopyFeedback(copied ? "Object key copied." : "Object key copy failed.");
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

function shellSingleQuote(value: string): string {
  return `'${value.replace(/'/g, `'\"'\"'`)}'`;
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

function startupTaskRdpUser(config: StartupTaskConfig | null | undefined): string | null {
  if (!config?.setupGuiRdp) {
    return null;
  }
  return normalizeRdpUsername(config.rdpUsername);
}

function startupTaskRdpUserForServer(serverId: string): string | null {
  return startupTaskRdpUser(startupTaskConfigForServer(serverId));
}

function isSshAuthFailureMessage(message: string): boolean {
  return /authentication (failed|rejected)|permission denied|invalid credentials|password/i.test(
    message
  );
}

function buildStartupTaskCommand(config: StartupTaskConfig, rdpPassword: string): string {
  const sections: string[] = [
    "set -eu",
    `
hc_forge_progress() {
  __raw="$1"
  shift || true
  __msg="$*"
  __base="\${HC_FORGE_PROGRESS_BASE:-0}"
  __span="\${HC_FORGE_PROGRESS_SPAN:-100}"
  __scaled=$(( __base + (__raw * __span) / 100 ))
  if [ "$__scaled" -lt 0 ]; then __scaled=0; fi
  if [ "$__scaled" -gt 100 ]; then __scaled=100; fi
  echo "[hc-forge] [progress] \${__scaled} \${__msg}"
}
`.trim(),
  ];

  const rdpUser = startupTaskRdpUser(config) ?? generateRdpUsername();

  if (config.autoUpdate) {
    if (config.setupGuiRdp) {
      sections.push("export HC_FORGE_PROGRESS_BASE=0\nexport HC_FORGE_PROGRESS_SPAN=60");
    } else {
      sections.push("export HC_FORGE_PROGRESS_BASE=0\nexport HC_FORGE_PROGRESS_SPAN=100");
    }
    sections.push(AUTO_VM_UPDATE_COMMAND);
  }
  if (config.setupGuiRdp) {
    if (config.autoUpdate) {
      sections.push("export HC_FORGE_PROGRESS_BASE=60\nexport HC_FORGE_PROGRESS_SPAN=40");
    } else {
      sections.push("export HC_FORGE_PROGRESS_BASE=0\nexport HC_FORGE_PROGRESS_SPAN=100");
    }
    sections.push(
      [
        `export HC_FORGE_RDP_USER=${shellSingleQuote(rdpUser)}`,
        `export HC_FORGE_RDP_PASSWORD=${shellSingleQuote(rdpPassword)}`,
        SETUP_GUI_RDP_COMMAND,
      ].join("\n")
    );
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
  const parsedRdpUser = normalizeRdpUsername(
    typeof raw.rdpUsername === "string" ? raw.rdpUsername : null
  );
  const statusRaw =
    raw.lastStatus === "done" || raw.lastStatus === "failed" ? raw.lastStatus : "pending";

  return {
    region: regionValue,
    autoUpdate,
    setupGuiRdp,
    rdpUsername: setupGuiRdp ? parsedRdpUser : null,
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
    const progressMarker = line.match(/\[hc-forge\]\s*\[progress\]\s*(\d{1,3})(?:\s+(.+))?/i);
    const isForgeLine = /\[hc-forge\]/i.test(line);
    if (!isForgeLine) {
      continue;
    }
    const parsedPercent =
      progressMarker && kind !== "stderr"
        ? Math.min(100, Math.max(0, Number.parseInt(progressMarker[1], 10)))
        : null;
    const currentPercent = autoUpdateProgressByServer.value[serverId]?.percent;
    const nextPercent =
      parsedPercent == null ? null : Math.max(currentPercent ?? 0, parsedPercent);
    const markerMessage = progressMarker?.[2]?.trim();
    const displayLine = markerMessage || line;
    setAutoUpdateProgress(serverId, {
      lastLine: displayLine.slice(0, 220),
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

function setSshUseFormPassword(value: boolean) {
  sshUseFormPassword.value = value;
}

function setSshManualPassword(value: string) {
  sshManualPassword.value = value;
}

function setSshCommandInput(value: string) {
  sshCommandInput.value = value;
}

function clearPlatformMessages() {
  platformError.value = null;
  platformInfo.value = null;
}

function clearPlatformCollections() {
  platformDockerImages.value = [];
  platformDockerContainers.value = [];
  platformMinikubeStatus.value = "";
  platformMinikubeNodes.value = "";
  platformMinikubePods.value = "";
  platformNixVersion.value = "";
  platformNixPackages.value = [];
  platformNixStoreUsage.value = "";
}

function setActivePlatformServer(ecs: EcsServer) {
  const nextServerId = ecs.id ?? null;
  if (!nextServerId) {
    return;
  }
  if (platformPanelServerId.value !== nextServerId) {
    clearPlatformCollections();
    clearPlatformMessages();
  }
  platformPanelServerId.value = nextServerId;
  platformPanelOpen.value = true;
}

function closePlatformPanel() {
  platformPanelOpen.value = false;
  platformPanelServerId.value = null;
  platformBusyServerId.value = null;
  platformActionLabel.value = null;
  clearPlatformMessages();
}

function isPlatformOpenForEcs(ecs: EcsServer): boolean {
  const serverId = ecs.id ?? "";
  if (!serverId) {
    return false;
  }
  return platformPanelOpen.value && platformPanelServerId.value === serverId;
}

function platformButtonLabel(ecs: EcsServer): string {
  const serverId = ecs.id ?? "";
  if (serverId && platformBusyServerId.value === serverId) {
    return platformActionLabel.value ?? "Working...";
  }
  return isPlatformOpenForEcs(ecs) ? "Close Ops" : "Platform Ops";
}

function setPlatformActiveTab(value: PlatformOpsTab) {
  platformActiveTab.value = value;
}

function setPlatformDockerInstallEnabled(value: boolean) {
  platformDockerInstallEnabled.value = value;
}

function setPlatformDockerfileContent(value: string) {
  platformDockerfileContent.value = value;
}

function setPlatformMinikubeInstallEnabled(value: boolean) {
  platformMinikubeInstallEnabled.value = value;
}

function setPlatformMinikubeEnsureDocker(value: boolean) {
  platformMinikubeEnsureDocker.value = value;
}

function setPlatformMinikubeAutoStart(value: boolean) {
  platformMinikubeAutoStart.value = value;
}

function setPlatformMinikubeProfile(value: string) {
  platformMinikubeProfile.value = value;
}

function setPlatformMinikubeDriver(value: "docker" | "none") {
  platformMinikubeDriver.value = value === "none" ? "none" : "docker";
}

function setPlatformMinikubeCpus(value: number) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    platformMinikubeCpus.value = 2;
    return;
  }
  platformMinikubeCpus.value = Math.min(64, Math.max(1, Math.trunc(parsed)));
}

function setPlatformMinikubeMemoryMb(value: number) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    platformMinikubeMemoryMb.value = 4096;
    return;
  }
  platformMinikubeMemoryMb.value = Math.min(262144, Math.max(1024, Math.trunc(parsed)));
}

function setPlatformMinikubeK8sVersion(value: string) {
  platformMinikubeK8sVersion.value = value;
}

function setPlatformNixInstallEnabled(value: boolean) {
  platformNixInstallEnabled.value = value;
}

function setPlatformNixEnableFlakes(value: boolean) {
  platformNixEnableFlakes.value = value;
}

function setPlatformNixRunGarbageCollect(value: boolean) {
  platformNixRunGarbageCollect.value = value;
}

function setPlatformNixPackagesInput(value: string) {
  platformNixPackagesInput.value = value;
}

function platformErrorMessage(prefix: string, err: unknown): string {
  return `${prefix}: ${errorToString(err)}`;
}

function setPlatformError(prefix: string, err: unknown) {
  const message = platformErrorMessage(prefix, err);
  platformError.value = message;
  platformInfo.value = null;
  addLog("app", "error", message);
}

function safePlatformProfile(): string {
  const profile = platformMinikubeProfile.value.trim();
  return profile || "hcforge";
}

function safePlatformCpus(): number {
  return Math.min(64, Math.max(1, Math.trunc(platformMinikubeCpus.value || 2)));
}

function safePlatformMemoryMb(): number {
  return Math.min(262144, Math.max(1024, Math.trunc(platformMinikubeMemoryMb.value || 4096)));
}

function formatPlatformOutput(stdout: string, stderr: string): string {
  const out = stdout.trim();
  if (out) {
    return out;
  }
  const err = stderr.trim();
  if (err) {
    return err;
  }
  return "No output returned.";
}

async function runPlatformOneShot(
  ecs: EcsServer,
  command: string,
  actionLabel: string
): Promise<{ stdout: string; stderr: string; host: string; exitStatus: number }> {
  const serverId = ecs.id ?? "";
  const serverName = ecs.name ?? serverId;
  if (!serverId) {
    throw new Error("Missing ECS ID for platform action.");
  }

  const host = findSshHostForServer(ecs);
  if (!host) {
    throw new Error("No public EIP found for selected ECS.");
  }

  const password = resolveSshPasswordForServer(serverId);
  if (!password) {
    throw new Error("SSH password is required for this action.");
  }

  platformBusyServerId.value = serverId;
  platformActionLabel.value = actionLabel;
  platformError.value = null;

  try {
    const response = await invoke<SshExecOneShotResult>("ssh_exec_one_shot", {
      params: {
        sessionId: `platform-ops:${serverId}:${Date.now()}`,
        host,
        port: 22,
        username: "root",
        password,
        command,
      },
    });

    await persistServerPassword(serverId, password);

    const exitStatus = response.exitStatus;
    const stderr = sanitizeSshText(response.stderr).trim();
    const stdout = sanitizeSshText(response.stdout).trim();
    if (exitStatus == null) {
      const summary = stderr || stdout || "missing remote exit status";
      throw new Error(`Remote command did not report an exit status: ${summary}`);
    }
    if (exitStatus !== 0) {
      const summary = stderr || stdout || `exit status ${exitStatus}`;
      throw new Error(summary);
    }

    if (stderr) {
      addLog(
        "app",
        "warn",
        `${actionLabel} on ${serverName} produced warnings: ${stderr.slice(0, 220)}`
      );
    }
    addLog("app", "info", `${actionLabel} completed on ${serverName} (${host}).`);
    return { stdout, stderr, host, exitStatus };
  } finally {
    platformBusyServerId.value = null;
    platformActionLabel.value = null;
  }
}

async function togglePlatformForEcs(ecs: EcsServer) {
  if (platformBusyServerId.value) {
    return;
  }
  const serverId = ecs.id ?? "";
  if (!serverId) {
    return;
  }
  if (isPlatformOpenForEcs(ecs)) {
    closePlatformPanel();
    return;
  }
  if (sshPanelOpen.value) {
    await closeSshPanel();
  }
  setActivePlatformServer(ecs);
}

async function refreshPlatformDockerImages(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const shouldLog = options.log ?? true;
  try {
    const { stdout } = await runPlatformOneShot(
      server,
      buildDockerImagesCommand(),
      "Docker image listing"
    );
    platformDockerImages.value = parseDockerImages(stdout);
    if (shouldLog) {
      platformInfo.value = `Loaded ${platformDockerImages.value.length} Docker image(s).`;
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to list Docker images", err);
  }
}

async function refreshPlatformDockerContainers(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const shouldLog = options.log ?? true;
  try {
    const { stdout } = await runPlatformOneShot(
      server,
      buildDockerContainersCommand(),
      "Docker container listing"
    );
    platformDockerContainers.value = parseDockerContainers(stdout);
    if (shouldLog) {
      platformInfo.value = `Loaded ${platformDockerContainers.value.length} Docker container(s).`;
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to list Docker containers", err);
  }
}

async function runPlatformDockerSetup() {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  clearPlatformMessages();
  try {
    const dockerfileContent = platformDockerfileContent.value.trim()
      ? platformDockerfileContent.value
      : "";
    const command = buildDockerSetupCommand({
      installDocker: platformDockerInstallEnabled.value,
      dockerfileContent,
      dockerfileTargetPath: platformDockerfileTargetPath,
    });
    await runPlatformOneShot(server, command, "Docker setup");
    await refreshPlatformDockerImages({ log: false });
    await refreshPlatformDockerContainers({ log: false });
    platformInfo.value = dockerfileContent
      ? `Docker setup finished for ${server.name ?? server.id ?? "selected ECS"}. Dockerfile uploaded to ${platformDockerfileTargetPath}.`
      : `Docker setup finished for ${server.name ?? server.id ?? "selected ECS"}.`;
    await sendUserNotification(
      "Docker setup complete",
      dockerfileContent
        ? `${server.name ?? server.id ?? "ECS"} is ready for Docker workflows and received a Dockerfile.`
        : `${server.name ?? server.id ?? "ECS"} is ready for Docker workflows.`
    );
  } catch (err) {
    setPlatformError("Docker setup failed", err);
  }
}

async function importPlatformDockerfile(file: File) {
  const server = platformPanelServer.value;
  const label = server?.name ?? server?.id ?? "selected ECS";
  if (!(file instanceof File)) {
    setPlatformError("Failed to import Dockerfile", new Error("No file selected."));
    return;
  }

  try {
    const content = await file.text();
    if (!content.trim()) {
      throw new Error("Dockerfile is empty.");
    }
    platformDockerfileContent.value = content.replace(/\r\n/g, "\n");

    platformInfo.value = `Imported Dockerfile from ${file.name} for ${label}.`;
    platformError.value = null;
    addLog("app", "info", `Imported Dockerfile from ${file.name} for ${label}.`);
  } catch (err) {
    setPlatformError("Failed to import Dockerfile", err);
  }
}

async function runPlatformMinikubeSetup() {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  clearPlatformMessages();
  try {
    const command = buildMinikubeSetupCommand({
      installMinikube: platformMinikubeInstallEnabled.value,
      ensureDocker: platformMinikubeEnsureDocker.value,
      autoStart: platformMinikubeAutoStart.value,
      profile: safePlatformProfile(),
      driver: platformMinikubeDriver.value,
      cpus: safePlatformCpus(),
      memoryMb: safePlatformMemoryMb(),
      kubernetesVersion: platformMinikubeK8sVersion.value.trim(),
    });
    await runPlatformOneShot(server, command, "Minikube setup");
    await refreshPlatformMinikubeStatus({ log: false });
    await refreshPlatformMinikubeNodes({ log: false });
    await refreshPlatformMinikubePods({ log: false });
    platformInfo.value = `Minikube setup finished for profile ${safePlatformProfile()}.`;
    await sendUserNotification(
      "Minikube setup complete",
      `${server.name ?? server.id ?? "ECS"} is ready for local Kubernetes testing.`
    );
  } catch (err) {
    setPlatformError("Minikube setup failed", err);
  }
}

async function refreshPlatformMinikubeStatus(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const profile = safePlatformProfile();
  const shouldLog = options.log ?? true;
  try {
    const { stdout, stderr } = await runPlatformOneShot(
      server,
      buildMinikubeStatusCommand(profile),
      "Minikube status"
    );
    platformMinikubeStatus.value = formatPlatformOutput(stdout, stderr);
    if (shouldLog) {
      platformInfo.value = `Minikube status refreshed for profile ${profile}.`;
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to read Minikube status", err);
  }
}

async function refreshPlatformMinikubeNodes(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const profile = safePlatformProfile();
  const shouldLog = options.log ?? true;
  try {
    const { stdout, stderr } = await runPlatformOneShot(
      server,
      buildMinikubeNodesCommand(profile),
      "Minikube nodes listing"
    );
    platformMinikubeNodes.value = formatPlatformOutput(stdout, stderr);
    if (shouldLog) {
      platformInfo.value = `Cluster nodes refreshed for profile ${profile}.`;
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to list Minikube nodes", err);
  }
}

async function refreshPlatformMinikubePods(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const profile = safePlatformProfile();
  const shouldLog = options.log ?? true;
  try {
    const { stdout, stderr } = await runPlatformOneShot(
      server,
      buildMinikubePodsCommand(profile),
      "Minikube pods listing"
    );
    platformMinikubePods.value = formatPlatformOutput(stdout, stderr);
    if (shouldLog) {
      platformInfo.value = `Cluster pods refreshed for profile ${profile}.`;
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to list Minikube pods", err);
  }
}

async function runPlatformNixSetup() {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  clearPlatformMessages();
  try {
    const command = buildNixSetupCommand({
      installNix: platformNixInstallEnabled.value,
      enableFlakes: platformNixEnableFlakes.value,
      runGarbageCollect: platformNixRunGarbageCollect.value,
      packages: platformNixPackagesInput.value,
    });
    await runPlatformOneShot(server, command, "Nix setup");
    await refreshPlatformNixVersion({ log: false });
    await refreshPlatformNixPackages({ log: false });
    await refreshPlatformNixStoreUsage({ log: false });
    platformInfo.value = `Nix setup finished for ${server.name ?? server.id ?? "selected ECS"}.`;
    await sendUserNotification(
      "Nix setup complete",
      `${server.name ?? server.id ?? "ECS"} is ready for Nix workflows.`
    );
  } catch (err) {
    setPlatformError("Nix setup failed", err);
  }
}

async function refreshPlatformNixVersion(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const shouldLog = options.log ?? true;
  try {
    const { stdout, stderr } = await runPlatformOneShot(
      server,
      buildNixVersionCommand(),
      "Nix version inspection"
    );
    platformNixVersion.value = formatPlatformOutput(stdout, stderr);
    if (shouldLog) {
      platformInfo.value = "Nix version information refreshed.";
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to read Nix version info", err);
  }
}

async function refreshPlatformNixPackages(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const shouldLog = options.log ?? true;
  try {
    const { stdout, stderr } = await runPlatformOneShot(
      server,
      buildNixPackagesCommand(),
      "Nix package listing"
    );
    const output = stdout.trim() ? stdout : stderr;
    platformNixPackages.value = parseNixPackages(output);
    if (shouldLog) {
      platformInfo.value = `Loaded ${platformNixPackages.value.length} Nix package(s).`;
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to list Nix packages", err);
  }
}

async function refreshPlatformNixStoreUsage(options: { log?: boolean } = {}) {
  const server = platformPanelServer.value;
  if (!server) {
    return;
  }
  const shouldLog = options.log ?? true;
  try {
    const { stdout, stderr } = await runPlatformOneShot(
      server,
      buildNixStoreUsageCommand(),
      "Nix store usage"
    );
    platformNixStoreUsage.value = formatPlatformOutput(stdout, stderr);
    if (shouldLog) {
      platformInfo.value = "Nix store usage refreshed.";
    }
  } catch (err) {
    if (!shouldLog) {
      throw err;
    }
    setPlatformError("Failed to read Nix store usage", err);
  }
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
    if (platformPanelOpen.value) {
      closePlatformPanel();
    }
    setActiveSshServer(ecs);
    return;
  }

  if (platformPanelOpen.value) {
    closePlatformPanel();
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

  const startupCommand = buildStartupTaskCommand(config, password);
  const rdpUser = startupTaskRdpUser(config);
  addLog("app", "info", `Running ${startupTaskLabel(config)} on ${label} (${host}).`);
  if (rdpUser) {
    addLog("app", "info", `RDP login for ${label} will use "${rdpUser}" on ${host}:3389.`);
  }
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

    const exitStatus = response.exitStatus;
    const stderr = sanitizeSshText(response.stderr).trim();
    const stdout = sanitizeSshText(response.stdout).trim();
    if (exitStatus == null) {
      const summary = stderr || stdout || "missing remote exit status";
      throw new Error(
        `${startupTaskLabel(config)} failed for ${label}: remote command did not report an exit status (${summary}).`
      );
    }
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
    if (tail && /\[hc-forge\]/i.test(tail)) {
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
  const rdpUsername = config.setupGuiRdp
    ? normalizeRdpUsername(config.rdpUsername) ?? generateRdpUsername()
    : null;
  startupTaskConfigsByServer.value = {
    ...startupTaskConfigsByServer.value,
    [serverId]: {
      ...config,
      region: config.region || region.value,
      rdpUsername,
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

async function loadResponseData(options: { log?: boolean } = {}) {
  if (loadingResponse.value || loadingEips.value || loadingEvss.value || loadingEcses.value) {
    return;
  }

  const shouldLog = options.log ?? true;
  loadingResponse.value = true;
  errorMsg.value = "";

  if (shouldLog) {
    addLog("app", "info", `Reloading response resources for region ${region.value}.`);
  }

  try {
    await Promise.all([loadEips({ log: false }), loadEvss({ log: false }), loadEcses({ log: false })]);
    await refreshCreatedInstance(createSummary.value?.serverId ?? createdServer.value?.id ?? null, {
      withEips: true,
      skipReload: true,
    });

    if (shouldLog) {
      addLog("app", "info", `Finished reloading response resources for region ${region.value}.`);
    }
  } finally {
    loadingResponse.value = false;
  }
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
    rdpUsername: setupGuiRdpOnStartup.value ? generateRdpUsername() : null,
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
        <h2>{{ activeModuleMeta.title }}</h2>
        <p class="service-subtitle">{{ activeModuleMeta.subtitle }}</p>
      </div>
      <div class="module-switcher">
        <button
          class="ghost minor module-switch-btn"
          type="button"
          title="Previous service module"
          @click="cycleServiceModule('prev')"
        >
          &lt;
        </button>
        <div class="chip tw-pill">{{ activeModuleMeta.chip }}</div>
        <button
          class="ghost minor module-switch-btn"
          type="button"
          title="Next service module"
          @click="cycleServiceModule('next')"
        >
          &gt;
        </button>
      </div>
    </section>

    <Transition :name="moduleTransitionName" mode="out-in">
      <div v-if="activeModule === 'ecs'" key="module-ecs" class="layout">
      <section class="panel">
        <div class="panel-head">
          <h2>Server Inputs</h2>
          <button class="primary quick-create" :disabled="!canCreate" @click="createEcs">
            {{ creating ? "Creating..." : "Create" }}
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
                <label for="setup-gui-rdp">Install graphical session + RDP on startup (optional)</label>
              </div>
            </div>
            <div class="startup-tip-box muted tiny">
              <p>Applies only to newly created VMs. Existing VMs are never changed.</p>
              <p>For RDP, open inbound TCP 3389 in the security group and log in with the generated <span class="mono">hcforge&lt;random&gt;</span> user shown in ECS cards (password = VM admin password).</p>
            </div>
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
              <button class="ghost minor action-chip fold-copy" type="button" @click="copyCurrentPassword">
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
                    <button class="ghost minor action-chip" type="button" @click="copyCurrentPassword">
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
            {{ creating ? "Creating..." : "Create" }}
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

      <EcsResponsePanel
        :quick-copy-feedback="quickCopyFeedback"
        :error-msg="errorMsg"
        :delete-msg="deleteMsg"
        :create-summary="createSummary"
        :created-server="createdServer"
        :created-eip="createdEip"
        :polling-ecs="pollingEcs"
        :polling-attempts="pollingAttempts"
        :poll-max-attempts="POLL_MAX_ATTEMPTS"
        :polling-status="pollingStatus"
        :polling-error="pollingError"
        :can-watch="canWatch"
        :result="result"
        :eips="eips"
        :evss="evss"
        :ecses="ecses"
        :loading-response="loadingResponse || loadingEips || loadingEvss || loadingEcses"
        :loading-eips="loadingEips"
        :loading-evss="loadingEvss"
        :loading-ecses="loadingEcses"
        :cache-age-eips="cacheAge.eips"
        :cache-age-evss="cacheAge.evss"
        :cache-age-ecses="cacheAge.ecses"
        :ssh-panel-open="sshPanelOpen"
        :ssh-panel-server="sshPanelServer"
        :ssh-panel-host="sshPanelHost"
        :ssh-connected-to-panel="sshConnectedToPanel"
        :ssh-busy-server-id="sshBusyServerId"
        :stopping-server-id="stoppingServerId"
        :deleting-server-id="deletingServerId"
        :ssh-running-command="sshRunningCommand"
        :ssh-use-form-password="sshUseFormPassword"
        :ssh-manual-password="sshManualPassword"
        :ssh-command-input="sshCommandInput"
        :ssh-terminal-entries="sshTerminalEntries"
        :status-tone="statusTone"
        :copy-eip-address="copyEipAddress"
        :evs-role="evsRole"
        :evs-attached-server="evsAttachedServer"
        :find-ssh-host-for-server="findSshHostForServer"
        :auto-update-status-for-server="autoUpdateStatusForServer"
        :auto-update-status-label="autoUpdateStatusLabel"
        :auto-update-progress-hint="autoUpdateProgressHint"
        :startup-task-rdp-user-for-server="startupTaskRdpUserForServer"
        :login-username-for-server="loginUsernameForServer"
        :copy-login-username-for-server="copyLoginUsernameForServer"
        :has-saved-password-for-server="hasSavedPasswordForServer"
        :copy-password-for-server="copyPasswordForServer"
        :is-ssh-open-for-ecs="isSshOpenForEcs"
        :can-connect-ssh="canConnectSsh"
        :toggle-ssh-for-ecs="toggleSshForEcs"
        :ssh-button-label="sshButtonLabel"
        :can-stop-ecs="canStopEcs"
        :stop-ecs="stopEcs"
        :delete-ecs="deleteEcs"
        :start-polling="startPolling"
        :stop-polling="stopPolling"
        :reload-response-data="loadResponseData"
        :reload-eips="loadEips"
        :reload-evss="loadEvss"
        :reload-ecses="loadEcses"
        :close-ssh-panel="closeSshPanel"
        :clear-ssh-terminal="clearSshTerminal"
        :reconnect-ssh-for-panel="reconnectSshForPanel"
        :disconnect-active-ssh="disconnectActiveSsh"
        :run-ssh-command="runSshCommand"
        :send-ssh-control-shortcut="sendSshControlShortcut"
        :handle-ssh-terminal-resize="handleSshTerminalResize"
        :handle-ssh-command-keydown="handleSshCommandKeydown"
        :set-ssh-use-form-password="setSshUseFormPassword"
        :set-ssh-manual-password="setSshManualPassword"
        :set-ssh-command-input="setSshCommandInput"
        :platform-panel-open="platformPanelOpen"
        :platform-panel-server="platformPanelServer"
        :platform-panel-host="platformPanelHost"
        :platform-panel-busy="platformPanelBusy"
        :platform-busy-server-id="platformBusyServerId"
        :platform-action-label="platformActionLabel"
        :platform-error="platformError"
        :platform-info="platformInfo"
        :platform-active-tab="platformActiveTab"
        :platform-docker-install-enabled="platformDockerInstallEnabled"
        :platform-docker-images="platformDockerImages"
        :platform-docker-containers="platformDockerContainers"
        :platform-dockerfile-target-path="platformDockerfileTargetPath"
        :platform-dockerfile-content="platformDockerfileContent"
        :platform-minikube-install-enabled="platformMinikubeInstallEnabled"
        :platform-minikube-ensure-docker="platformMinikubeEnsureDocker"
        :platform-minikube-auto-start="platformMinikubeAutoStart"
        :platform-minikube-profile="platformMinikubeProfile"
        :platform-minikube-driver="platformMinikubeDriver"
        :platform-minikube-cpus="platformMinikubeCpus"
        :platform-minikube-memory-mb="platformMinikubeMemoryMb"
        :platform-minikube-k8s-version="platformMinikubeK8sVersion"
        :platform-minikube-status="platformMinikubeStatus"
        :platform-minikube-nodes="platformMinikubeNodes"
        :platform-minikube-pods="platformMinikubePods"
        :platform-nix-install-enabled="platformNixInstallEnabled"
        :platform-nix-enable-flakes="platformNixEnableFlakes"
        :platform-nix-run-garbage-collect="platformNixRunGarbageCollect"
        :platform-nix-packages-input="platformNixPackagesInput"
        :platform-nix-version="platformNixVersion"
        :platform-nix-packages="platformNixPackages"
        :platform-nix-store-usage="platformNixStoreUsage"
        :is-platform-open-for-ecs="isPlatformOpenForEcs"
        :platform-button-label="platformButtonLabel"
        :toggle-platform-for-ecs="togglePlatformForEcs"
        :close-platform-panel="closePlatformPanel"
        :set-platform-active-tab="setPlatformActiveTab"
        :set-platform-docker-install-enabled="setPlatformDockerInstallEnabled"
        :set-platform-dockerfile-content="setPlatformDockerfileContent"
        :set-platform-minikube-install-enabled="setPlatformMinikubeInstallEnabled"
        :set-platform-minikube-ensure-docker="setPlatformMinikubeEnsureDocker"
        :set-platform-minikube-auto-start="setPlatformMinikubeAutoStart"
        :set-platform-minikube-profile="setPlatformMinikubeProfile"
        :set-platform-minikube-driver="setPlatformMinikubeDriver"
        :set-platform-minikube-cpus="setPlatformMinikubeCpus"
        :set-platform-minikube-memory-mb="setPlatformMinikubeMemoryMb"
        :set-platform-minikube-k8s-version="setPlatformMinikubeK8sVersion"
        :set-platform-nix-install-enabled="setPlatformNixInstallEnabled"
        :set-platform-nix-enable-flakes="setPlatformNixEnableFlakes"
        :set-platform-nix-run-garbage-collect="setPlatformNixRunGarbageCollect"
        :set-platform-nix-packages-input="setPlatformNixPackagesInput"
        :run-platform-docker-setup="runPlatformDockerSetup"
        :refresh-platform-docker-images="refreshPlatformDockerImages"
        :refresh-platform-docker-containers="refreshPlatformDockerContainers"
        :import-platform-dockerfile="importPlatformDockerfile"
        :run-platform-minikube-setup="runPlatformMinikubeSetup"
        :refresh-platform-minikube-status="refreshPlatformMinikubeStatus"
        :refresh-platform-minikube-nodes="refreshPlatformMinikubeNodes"
        :refresh-platform-minikube-pods="refreshPlatformMinikubePods"
        :run-platform-nix-setup="runPlatformNixSetup"
        :refresh-platform-nix-version="refreshPlatformNixVersion"
        :refresh-platform-nix-packages="refreshPlatformNixPackages"
        :refresh-platform-nix-store-usage="refreshPlatformNixStoreUsage"
      />
      </div>

      <CceModulePanel
        v-else-if="activeModule === 'cce'"
        key="module-cce"
        v-model:region="region"
        v-model:cluster-name="cceClusterName"
        v-model:cluster-version="cceClusterVersion"
        v-model:cluster-flavor="cceClusterFlavor"
        v-model:cluster-type="cceClusterType"
        v-model:cluster-description="cceClusterDescription"
        v-model:cluster-tag-env="cceClusterTagEnv"
        v-model:cluster-vpc-id="cceClusterVpcId"
        v-model:cluster-subnet-id="cceClusterSubnetId"
        v-model:cluster-container-network-mode="cceClusterContainerNetworkMode"
        v-model:cluster-container-network-cidr="cceClusterContainerNetworkCidr"
        v-model:cluster-service-cidr="cceClusterServiceCidr"
        v-model:cluster-authentication-mode="cceClusterAuthenticationMode"
        v-model:nat-gateway-name="cceNatGatewayName"
        v-model:nat-gateway-description="cceNatGatewayDescription"
        v-model:nat-gateway-spec="cceNatGatewaySpec"
        v-model:selected-access-eip-id="cceSelectedAccessEipId"
        :cluster-versions="CCE_KUBERNETES_VERSIONS"
        :cluster-flavors="CCE_CONTROL_PLANE_FLAVORS"
        :nat-gateway-specs="CCE_NAT_GATEWAY_SPECS"
        :container-network-cidrs="CCE_CONTAINER_NETWORK_CIDR_OPTIONS"
        :service-cidrs="CCE_SERVICE_CIDR_OPTIONS"
        :regions="regions"
        :cluster-types="CCE_CLUSTER_TYPES"
        :container-network-modes="CCE_CONTAINER_NETWORK_MODES"
        :authentication-modes="CCE_AUTHENTICATION_MODES"
        :vpcs="cceVpcs"
        :subnets="cceSubnets"
        :loading-vpcs="cceLoadingVpcs"
        :loading-subnets="cceLoadingSubnets"
        :can-create-cluster="cceCanCreateCluster"
        :creating-cluster="cceCreatingCluster"
        :clusters="cceClusters"
        :loading-clusters="cceLoadingClusters"
        :selected-cluster-id="cceSelectedClusterId"
        :deleting-cluster-id="cceDeletingClusterId"
        :node-pools="cceNodePools"
        :loading-node-pools="cceLoadingNodePools"
        :last-result="cceLastResult"
        :job-result="cceJobResult"
        :last-job-id="cceLastJobId"
        :loading-job="cceLoadingJob"
        :nat-gateways="cceNatGateways"
        :loading-nat-gateways="cceLoadingNatGateways"
        :can-create-nat-gateway="cceCanCreateNatGateway"
        :creating-nat-gateway="cceCreatingNatGateway"
        :deleting-nat-gateway-id="cceDeletingNatGatewayId"
        :access-eips="cceAccessEips"
        :loading-access-eips="cceLoadingAccessEips"
        :selected-cluster-external-ip="cceSelectedClusterExternalIp"
        :binding-access-eip="cceBindingAccessEip"
        :downloading-kubeconfig="cceDownloadingKubeconfig"
        :error-msg="cceErrorMsg"
        :quick-copy-feedback="quickCopyFeedback"
        @create-cluster="createCceCluster"
        @reload-vpcs="loadCceVpcs()"
        @reload-subnets="loadCceSubnets()"
        @reload-clusters="loadCceClusters()"
        @select-cluster="selectCceCluster"
        @delete-cluster="deleteCceCluster"
        @reload-node-pools="loadCceNodePools()"
        @reload-job="loadCceJob()"
        @reload-nat-gateways="loadCceNatGateways()"
        @create-nat-gateway="createCceNatGateway"
        @delete-nat-gateway="deleteCceNatGateway"
        @reload-access-eips="loadCceAccessEips()"
        @bind-cluster-access-eip="bindCceClusterApiEip"
        @download-kubeconfig="downloadCceKubeconfig"
      />

      <ObsModulePanel
        v-else
        key="module-obs"
        v-model:region="region"
        v-model:bucket-name="obsBucketName"
        v-model:default-storage-class="obsDefaultStorageClass"
        v-model:bucket-acl="obsBucketAcl"
        v-model:upload-object-key="obsUploadObjectKey"
        v-model:upload-content-type="obsUploadContentType"
        v-model:object-prefix="obsObjectPrefix"
        v-model:object-marker="obsObjectMarker"
        v-model:object-max-keys="obsObjectMaxKeys"
        :regions="regions"
        :bucket-name-error="obsBucketNameError"
        :storage-classes="OBS_BUCKET_STORAGE_CLASSES"
        :bucket-acls="OBS_BUCKET_ACLS"
        :can-create-bucket="obsCanCreateBucket"
        :creating-bucket="obsCreatingBucket"
        :selected-bucket="obsSelectedBucket"
        :selected-bucket-record="obsSelectedBucketRecord"
        :buckets="obsBuckets"
        :loading-buckets="obsLoadingBuckets"
        :deleting-bucket="obsDeletingBucket"
        :resolved-upload-content-type="obsResolvedUploadContentType"
        :single-put-limit-label="obsSinglePutLimitLabel"
        :can-upload-object="obsCanUploadObject"
        :uploading-object="obsUploadingObject"
        :upload-progress="obsUploadProgress"
        :max-keys-min="OBS_MAX_KEYS_MIN"
        :max-keys-max="OBS_MAX_KEYS_MAX"
        :can-load-objects="obsCanLoadObjects"
        :loading-objects="obsLoadingObjects"
        :objects="obsObjects"
        :bucket-total-size-bytes="obsBucketTotalSizeBytes"
        :bucket-total-object-count="obsBucketTotalObjectCount"
        :loading-bucket-totals="obsLoadingBucketTotals"
        :bucket-totals-error="obsBucketTotalsError"
        :deleting-object="obsDeletingObject"
        :downloading-object="obsDownloadingObject"
        :download-progress="obsDownloadProgress"
        :last-result="obsLastResult"
        :error-msg="obsErrorMsg"
        :quick-copy-feedback="quickCopyFeedback"
        :format-obs-object-size="formatObsObjectSize"
        @create-bucket="createObsBucket"
        @reload-buckets="loadObsBuckets()"
        @select-bucket="selectObsBucket"
        @copy-bucket-name="copyObsBucketName"
        @delete-bucket="deleteObsBucket"
        @upload-file="onObsUploadFileChange"
        @upload-object="uploadObsObject"
        @reload-objects="reloadObsObjectsAndTotals()"
        @search-objects="reloadObsObjectsAndTotals()"
        @download-object="downloadObsObject"
        @copy-object-key="copyObsObjectKey"
        @delete-object="deleteObsObject"
      />
    </Transition>
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

<style src="./styles/app.css"></style>
