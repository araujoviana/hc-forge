<script setup lang="ts">
import { computed } from "vue";
import ReloadIconButton from "../ReloadIconButton.vue";
import TrashIconButton from "../TrashIconButton.vue";
import type { EipRecord, SubnetOption, VpcOption } from "../../types/ecs";
import type {
  CceCluster,
  CceNatGateway,
  CceNodePool,
  CceOperationResult,
} from "../../types/cce";

type PlainObject = Record<string, unknown>;

const props = defineProps<{
  region: string;
  regions: readonly string[];
  clusterName: string;
  clusterVersion: string;
  clusterFlavor: string;
  clusterType: string;
  clusterDescription: string;
  clusterTagEnv: string;
  clusterVpcId: string;
  clusterSubnetId: string;
  clusterContainerNetworkMode: string;
  clusterContainerNetworkCidr: string;
  clusterServiceCidr: string;
  clusterAuthenticationMode: string;
  nodePoolName: string;
  nodePoolFlavor: string;
  nodePoolAvailabilityZone: string;
  nodePoolOs: string;
  nodePoolSshKey: string;
  nodePoolInitialCount: number;
  nodePoolRootVolumeType: string;
  nodePoolRootVolumeSize: number;
  nodePoolDataVolumeType: string;
  nodePoolDataVolumeSize: number;
  nodePoolMaxPods: number;
  natGatewayName: string;
  natGatewayDescription: string;
  natGatewaySpec: string;
  clusterVersions: readonly string[];
  clusterFlavors: readonly string[];
  nodePoolFlavorOptions: Array<{ id: string; label: string }>;
  nodePoolAvailabilityZones: string[];
  nodePoolOsOptions: readonly string[];
  nodeVolumeTypes: readonly string[];
  natGatewaySpecs: readonly string[];
  containerNetworkCidrs: readonly string[];
  serviceCidrs: readonly string[];
  clusterTypes: readonly string[];
  containerNetworkModes: readonly string[];
  authenticationModes: readonly string[];
  vpcs: VpcOption[];
  subnets: SubnetOption[];
  loadingVpcs: boolean;
  loadingSubnets: boolean;
  canCreateCluster: boolean;
  creatingCluster: boolean;
  clusters: CceCluster[];
  loadingClusters: boolean;
  selectedClusterId: string;
  deletingClusterId: string | null;
  nodePools: CceNodePool[];
  loadingNodePoolFlavors: boolean;
  loadingNodePools: boolean;
  canCreateNodePool: boolean;
  creatingNodePool: boolean;
  deletingNodePoolId: string | null;
  lastResult: CceOperationResult | null;
  jobResult: CceOperationResult | null;
  lastJobId: string;
  loadingJob: boolean;
  pollingCce: boolean;
  pollingAttempts: number;
  pollMaxAttempts: number;
  pollingStatus: string | null;
  pollingError: string | null;
  pollingTargetLabel: string | null;
  canWatch: boolean;
  natGateways: CceNatGateway[];
  loadingNatGateways: boolean;
  canCreateNatGateway: boolean;
  creatingNatGateway: boolean;
  deletingNatGatewayId: string | null;
  accessEips: EipRecord[];
  loadingAccessEips: boolean;
  selectedClusterExternalIp: string;
  bindingAccessEip: boolean;
  apiEipBindRequested: boolean;
  downloadingKubeconfig: boolean;
  errorMsg: string;
  quickCopyFeedback: string | null;
}>();

const emit = defineEmits<{
  (e: "update:region", value: string): void;
  (e: "update:clusterName", value: string): void;
  (e: "update:clusterVersion", value: string): void;
  (e: "update:clusterFlavor", value: string): void;
  (e: "update:clusterType", value: string): void;
  (e: "update:clusterDescription", value: string): void;
  (e: "update:clusterTagEnv", value: string): void;
  (e: "update:clusterVpcId", value: string): void;
  (e: "update:clusterSubnetId", value: string): void;
  (e: "update:clusterContainerNetworkMode", value: string): void;
  (e: "update:clusterContainerNetworkCidr", value: string): void;
  (e: "update:clusterServiceCidr", value: string): void;
  (e: "update:clusterAuthenticationMode", value: string): void;
  (e: "update:nodePoolName", value: string): void;
  (e: "update:nodePoolFlavor", value: string): void;
  (e: "update:nodePoolAvailabilityZone", value: string): void;
  (e: "update:nodePoolOs", value: string): void;
  (e: "update:nodePoolSshKey", value: string): void;
  (e: "update:nodePoolInitialCount", value: number): void;
  (e: "update:nodePoolRootVolumeType", value: string): void;
  (e: "update:nodePoolRootVolumeSize", value: number): void;
  (e: "update:nodePoolDataVolumeType", value: string): void;
  (e: "update:nodePoolDataVolumeSize", value: number): void;
  (e: "update:nodePoolMaxPods", value: number): void;
  (e: "update:natGatewayName", value: string): void;
  (e: "update:natGatewayDescription", value: string): void;
  (e: "update:natGatewaySpec", value: string): void;
  (e: "create-cluster"): void;
  (e: "reload-vpcs"): void;
  (e: "reload-subnets"): void;
  (e: "reload-clusters"): void;
  (e: "select-cluster", clusterId: string): void;
  (e: "delete-cluster", cluster: CceCluster): void;
  (e: "reload-node-pools"): void;
  (e: "reload-node-pool-flavors"): void;
  (e: "create-node-pool"): void;
  (e: "delete-node-pool", pool: CceNodePool): void;
  (e: "reload-job"): void;
  (e: "start-polling", clusterId: string | null): void;
  (e: "stop-polling"): void;
  (e: "reload-nat-gateways"): void;
  (e: "create-nat-gateway"): void;
  (e: "delete-nat-gateway", gateway: CceNatGateway): void;
  (e: "reload-access-eips"): void;
  (e: "create-bind-cluster-access-eip"): void;
  (e: "download-kubeconfig"): void;
}>();

const regionModel = computed({
  get: () => props.region,
  set: (value: string) => emit("update:region", value),
});
const clusterNameModel = computed({
  get: () => props.clusterName,
  set: (value: string) => emit("update:clusterName", value),
});
const clusterVersionModel = computed({
  get: () => props.clusterVersion,
  set: (value: string) => emit("update:clusterVersion", value),
});
const clusterFlavorModel = computed({
  get: () => props.clusterFlavor,
  set: (value: string) => emit("update:clusterFlavor", value),
});
const clusterTypeModel = computed({
  get: () => props.clusterType,
  set: (value: string) => emit("update:clusterType", value),
});
const clusterDescriptionModel = computed({
  get: () => props.clusterDescription,
  set: (value: string) => emit("update:clusterDescription", value),
});
const clusterTagEnvModel = computed({
  get: () => props.clusterTagEnv,
  set: (value: string) => emit("update:clusterTagEnv", value),
});
const clusterVpcIdModel = computed({
  get: () => props.clusterVpcId,
  set: (value: string) => emit("update:clusterVpcId", value),
});
const clusterSubnetIdModel = computed({
  get: () => props.clusterSubnetId,
  set: (value: string) => emit("update:clusterSubnetId", value),
});
const clusterContainerNetworkModeModel = computed({
  get: () => props.clusterContainerNetworkMode,
  set: (value: string) => emit("update:clusterContainerNetworkMode", value),
});
const clusterContainerNetworkCidrModel = computed({
  get: () => props.clusterContainerNetworkCidr,
  set: (value: string) => emit("update:clusterContainerNetworkCidr", value),
});
const clusterServiceCidrModel = computed({
  get: () => props.clusterServiceCidr,
  set: (value: string) => emit("update:clusterServiceCidr", value),
});
const clusterAuthenticationModeModel = computed({
  get: () => props.clusterAuthenticationMode,
  set: (value: string) => emit("update:clusterAuthenticationMode", value),
});
const nodePoolNameModel = computed({
  get: () => props.nodePoolName,
  set: (value: string) => emit("update:nodePoolName", value),
});
const nodePoolFlavorModel = computed({
  get: () => props.nodePoolFlavor,
  set: (value: string) => emit("update:nodePoolFlavor", value),
});
const nodePoolAvailabilityZoneModel = computed({
  get: () => props.nodePoolAvailabilityZone,
  set: (value: string) => emit("update:nodePoolAvailabilityZone", value),
});
const nodePoolOsModel = computed({
  get: () => props.nodePoolOs,
  set: (value: string) => emit("update:nodePoolOs", value),
});
const nodePoolSshKeyModel = computed({
  get: () => props.nodePoolSshKey,
  set: (value: string) => emit("update:nodePoolSshKey", value),
});
const nodePoolInitialCountModel = computed({
  get: () => props.nodePoolInitialCount,
  set: (value: number) => emit("update:nodePoolInitialCount", value),
});
const nodePoolRootVolumeTypeModel = computed({
  get: () => props.nodePoolRootVolumeType,
  set: (value: string) => emit("update:nodePoolRootVolumeType", value),
});
const nodePoolRootVolumeSizeModel = computed({
  get: () => props.nodePoolRootVolumeSize,
  set: (value: number) => emit("update:nodePoolRootVolumeSize", value),
});
const nodePoolDataVolumeTypeModel = computed({
  get: () => props.nodePoolDataVolumeType,
  set: (value: string) => emit("update:nodePoolDataVolumeType", value),
});
const nodePoolDataVolumeSizeModel = computed({
  get: () => props.nodePoolDataVolumeSize,
  set: (value: number) => emit("update:nodePoolDataVolumeSize", value),
});
const nodePoolMaxPodsModel = computed({
  get: () => props.nodePoolMaxPods,
  set: (value: number) => emit("update:nodePoolMaxPods", value),
});
const natGatewayNameModel = computed({
  get: () => props.natGatewayName,
  set: (value: string) => emit("update:natGatewayName", value),
});
const natGatewayDescriptionModel = computed({
  get: () => props.natGatewayDescription,
  set: (value: string) => emit("update:natGatewayDescription", value),
});
const natGatewaySpecModel = computed({
  get: () => props.natGatewaySpec,
  set: (value: string) => emit("update:natGatewaySpec", value),
});

function asObject(value: unknown): PlainObject {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return {};
  }
  return value as PlainObject;
}

function textValue(value: unknown, fallback = "—"): string {
  if (typeof value !== "string") {
    return fallback;
  }
  const trimmed = value.trim();
  return trimmed || fallback;
}

function clusterId(cluster: CceCluster): string {
  const metadata = asObject(cluster.metadata);
  const direct = metadata.id ?? metadata.uid;
  if (typeof direct === "string" && direct.trim()) {
    return direct.trim();
  }
  return "";
}

function clusterName(cluster: CceCluster): string {
  const metadata = asObject(cluster.metadata);
  return textValue(metadata.name, clusterId(cluster) || "unnamed-cluster");
}

function clusterVersion(cluster: CceCluster): string {
  const spec = asObject(cluster.spec);
  return textValue(spec.version);
}

function clusterPhase(cluster: CceCluster): string {
  const status = asObject(cluster.status);
  const raw = status.phase ?? status.clusterPhase ?? status.status;
  return textValue(raw, "UNKNOWN");
}

function clusterVpc(cluster: CceCluster): string {
  const spec = asObject(cluster.spec);
  const hostNetwork = asObject(spec.hostNetwork);
  return textValue(hostNetwork.vpc);
}

function clusterSubnet(cluster: CceCluster): string {
  const spec = asObject(cluster.spec);
  const hostNetwork = asObject(spec.hostNetwork);
  return textValue(hostNetwork.subnet);
}

function nodePoolId(pool: CceNodePool): string {
  const metadata = asObject(pool.metadata);
  const raw = metadata.id ?? metadata.uid;
  if (typeof raw === "string" && raw.trim()) {
    return raw.trim();
  }
  return "";
}

function nodePoolName(pool: CceNodePool): string {
  const metadata = asObject(pool.metadata);
  return textValue(metadata.name, nodePoolId(pool) || "node-pool");
}

function nodePoolPhase(pool: CceNodePool): string {
  const status = asObject(pool.status);
  return textValue(status.phase ?? status.status, "UNKNOWN");
}

function nodePoolVersion(pool: CceNodePool): string {
  const spec = asObject(pool.spec);
  return textValue(spec.version ?? spec.nodeTemplateVersion);
}

function natGatewayId(gateway: CceNatGateway): string {
  return textValue(gateway.id, "");
}

function natGatewayNameText(gateway: CceNatGateway): string {
  return textValue(gateway.name, natGatewayId(gateway) || "nat-gateway");
}

function natGatewayStatus(gateway: CceNatGateway): string {
  return textValue(gateway.status, "UNKNOWN");
}

function prettyBody(body: string): string {
  const trimmed = String(body ?? "").trim();
  if (!trimmed) {
    return "No body returned.";
  }
  try {
    return JSON.stringify(JSON.parse(trimmed), null, 2);
  } catch {
    return trimmed;
  }
}

function parseJsonMaybe(body: string): unknown | null {
  const trimmed = String(body ?? "").trim();
  if (!trimmed) {
    return null;
  }
  try {
    return JSON.parse(trimmed);
  } catch {
    return null;
  }
}

function jobStatusText(result: CceOperationResult | null): string {
  if (!result) {
    return "No CCE job queried yet.";
  }
  const payload = parseJsonMaybe(result.body);
  if (!payload || typeof payload !== "object") {
    return `${result.status_code} ${result.status}`;
  }
  const data = payload as PlainObject;
  const status = asObject(data.status);
  const value =
    status.state ??
    status.status ??
    status.phase ??
    data.status ??
    data.job_status ??
    data.jobStatus;
  return textValue(value, `${result.status_code} ${result.status}`);
}

const selectedClusterLabel = computed(() => {
  if (!props.selectedClusterId) {
    return "none selected";
  }
  const selected = props.clusters.find((cluster) => clusterId(cluster) === props.selectedClusterId);
  if (!selected) {
    return props.selectedClusterId;
  }
  return `${clusterName(selected)} (${clusterId(selected)})`;
});

const hasSelectedCluster = computed(() => !!props.selectedClusterId);
const canCreateBindAccessEip = computed(
  () =>
    hasSelectedCluster.value &&
    !props.bindingAccessEip &&
    !props.apiEipBindRequested &&
    !props.selectedClusterExternalIp
);
const canDownloadExternalKubeconfig = computed(
  () =>
    hasSelectedCluster.value &&
    !props.downloadingKubeconfig &&
    !props.apiEipBindRequested &&
    !!props.selectedClusterExternalIp
);
const clusterAccessStateText = computed(() => {
  if (props.bindingAccessEip) {
    return "Creating EIP and binding API endpoint...";
  }
  if (props.apiEipBindRequested) {
    return "Bind request submitted. Waiting for CCE to publish the API EIP.";
  }
  if (props.selectedClusterExternalIp) {
    return "API endpoint has a bound public EIP.";
  }
  return "API endpoint is not bound to a public EIP yet.";
});
const clusterAccessActionText = computed(() => {
  if (props.bindingAccessEip) {
    return "Creating + Binding...";
  }
  if (props.apiEipBindRequested) {
    return "Bind Requested...";
  }
  if (props.selectedClusterExternalIp) {
    return "API EIP Bound";
  }
  return "Create + Bind API EIP";
});
const lastResultPrettyBody = computed(() =>
  props.lastResult ? prettyBody(props.lastResult.body) : "No body returned."
);
const jobResultPrettyBody = computed(() =>
  props.jobResult ? prettyBody(props.jobResult.body) : "No body returned."
);
const jobStatusSummary = computed(() => jobStatusText(props.jobResult));
</script>

<template>
  <div class="ccex-shell">
    <aside class="ccex-sidebar">
      <section class="ccex-card">
        <div class="ccex-row-between">
          <h3>Create CCE Cluster</h3>
          <button class="primary ccex-top-cta" :disabled="!canCreateCluster" @click="emit('create-cluster')">
            {{ creatingCluster ? "Creating..." : "Create Cluster" }}
          </button>
        </div>

        <div class="ccex-grid-2">
          <label class="ccex-input-group">
            <span>Region</span>
            <select v-model="regionModel" class="ccex-input">
              <option v-for="item in regions" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group">
            <span>Cluster Name</span>
            <input v-model="clusterNameModel" class="ccex-input" spellcheck="false" placeholder="my-cce-cluster" />
          </label>
          <label class="ccex-input-group">
            <span>Kubernetes Version</span>
            <select v-model="clusterVersionModel" class="ccex-input">
              <option v-for="item in clusterVersions" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group">
            <span>Control Plane Flavor</span>
            <select v-model="clusterFlavorModel" class="ccex-input">
              <option v-for="item in clusterFlavors" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group">
            <span>Cluster Type</span>
            <select v-model="clusterTypeModel" class="ccex-input">
              <option v-for="item in clusterTypes" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group">
            <span>Authentication</span>
            <select v-model="clusterAuthenticationModeModel" class="ccex-input">
              <option v-for="item in authenticationModes" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
        </div>

        <label class="ccex-input-group">
          <span>Description (optional)</span>
          <textarea
            v-model="clusterDescriptionModel"
            class="ccex-input ccex-textarea"
            rows="3"
            placeholder="Dev/test cluster for team workloads"
          ></textarea>
        </label>

        <label class="ccex-input-group">
          <span>Environment Tag (optional)</span>
          <input v-model="clusterTagEnvModel" class="ccex-input" spellcheck="false" placeholder="dev" />
        </label>

        <div class="ccex-subgroup">
          <div class="ccex-subgroup-head">
            <strong>Cluster Network</strong>
          </div>
          <div class="ccex-grid-2">
            <label class="ccex-input-group">
              <span>VPC</span>
              <div class="ccex-inline-input">
                <select v-model="clusterVpcIdModel" class="ccex-input">
                  <option value="" disabled>Select a VPC</option>
                  <option v-for="item in vpcs" :key="item.id" :value="item.id">{{ item.name }}</option>
                </select>
                <ReloadIconButton
                  :disabled="loadingVpcs"
                  :loading="loadingVpcs"
                  :title="loadingVpcs ? 'Reloading VPCs...' : 'Reload VPCs'"
                  @click="emit('reload-vpcs')"
                />
              </div>
            </label>
            <label class="ccex-input-group">
              <span>Subnet</span>
              <div class="ccex-inline-input">
                <select v-model="clusterSubnetIdModel" class="ccex-input">
                  <option value="" disabled>Select a subnet</option>
                  <option v-for="item in subnets" :key="item.id" :value="item.id">
                    {{ item.name }} ({{ item.cidr }})
                  </option>
                </select>
                <ReloadIconButton
                  :disabled="loadingSubnets || !clusterVpcIdModel"
                  :loading="loadingSubnets"
                  :title="loadingSubnets ? 'Reloading subnets...' : 'Reload subnets'"
                  @click="emit('reload-subnets')"
                />
              </div>
            </label>
            <label class="ccex-input-group">
              <span>Container Network Mode</span>
              <select v-model="clusterContainerNetworkModeModel" class="ccex-input">
                <option v-for="item in containerNetworkModes" :key="item" :value="item">{{ item }}</option>
              </select>
            </label>
            <label class="ccex-input-group">
              <span>Container CIDR</span>
              <select v-model="clusterContainerNetworkCidrModel" class="ccex-input">
                <option v-for="item in containerNetworkCidrs" :key="item" :value="item">{{ item }}</option>
              </select>
            </label>
            <label class="ccex-input-group ccex-span-2">
              <span>Kubernetes Service CIDR</span>
              <select v-model="clusterServiceCidrModel" class="ccex-input">
                <option v-for="item in serviceCidrs" :key="item" :value="item">{{ item }}</option>
              </select>
            </label>
          </div>
        </div>
      </section>

      <section class="ccex-card">
        <h3>Last CCE Action</h3>
        <div v-if="lastResult" class="ccex-last-action">
          <div class="ccex-status-row">
            <span class="badge">{{ lastResult.status_code }}</span>
            <span>{{ lastResult.status }}</span>
          </div>
          <details class="ccex-raw-details">
            <summary>Raw response</summary>
            <pre class="ccex-raw">{{ lastResultPrettyBody }}</pre>
          </details>
        </div>
        <p v-else class="ccex-hint">No CCE action yet.</p>
      </section>

      <section class="ccex-card">
        <div class="ccex-row-between">
          <h3>Last Job Status</h3>
          <ReloadIconButton
            :disabled="!lastJobId || loadingJob"
            :loading="loadingJob"
            :title="loadingJob ? 'Refreshing CCE job...' : 'Refresh CCE job'"
            @click="emit('reload-job')"
          />
        </div>
        <p class="ccex-hint">
          Job ID: <span class="mono">{{ lastJobId || "none" }}</span>
        </p>
        <p class="ccex-hint">Status: {{ jobStatusSummary }}</p>
        <div class="ccex-polling-row">
          <div>
            <p class="ccex-hint">
              Watch:
              <span v-if="pollingCce">Active ({{ pollingAttempts }}/{{ pollMaxAttempts }})</span>
              <span v-else>Idle</span>
              <span v-if="pollingStatus"> • {{ pollingStatus }}</span>
            </p>
            <p class="ccex-hint">
              Target:
              <span class="mono">{{ pollingTargetLabel || selectedClusterLabel }}</span>
            </p>
            <p v-if="pollingError" class="ccex-hint">{{ pollingError }}</p>
          </div>
          <div class="ccex-actions">
            <button
              class="ghost minor"
              :disabled="pollingCce || !canWatch"
              @click="emit('start-polling', selectedClusterId || null)"
            >
              Start Watch
            </button>
            <button class="ghost minor" :disabled="!pollingCce" @click="emit('stop-polling')">Stop</button>
          </div>
        </div>
        <details v-if="jobResult" class="ccex-raw-details">
          <summary>Raw job payload</summary>
          <pre class="ccex-raw">{{ jobResultPrettyBody }}</pre>
        </details>
      </section>
    </aside>

    <section class="ccex-main">
      <div v-if="errorMsg" class="ccex-alert ccex-alert-error">{{ errorMsg }}</div>
      <div v-else-if="quickCopyFeedback" class="ccex-alert ccex-alert-info">{{ quickCopyFeedback }}</div>

      <section class="ccex-card">
        <div class="ccex-row-between">
          <h3>Clusters</h3>
          <ReloadIconButton
            :disabled="loadingClusters"
            :loading="loadingClusters"
            :title="loadingClusters ? 'Reloading CCE clusters...' : 'Reload CCE clusters'"
            @click="emit('reload-clusters')"
          />
        </div>
        <p class="ccex-subtitle">{{ clusters.length }} total • account-level in selected region</p>

        <div v-if="clusters.length" class="ccex-list">
          <article
            v-for="cluster in clusters"
            :key="clusterId(cluster) || clusterName(cluster)"
            class="ccex-list-item"
            :class="{ selected: clusterId(cluster) === selectedClusterId }"
          >
            <div class="ccex-row-between ccex-item-head">
              <div>
                <div class="ccex-item-title mono">{{ clusterName(cluster) }}</div>
                <div class="ccex-meta">ID: <span class="mono">{{ clusterId(cluster) || "—" }}</span></div>
              </div>
              <span class="status-pill status-neutral">{{ clusterPhase(cluster) }}</span>
            </div>
            <div class="ccex-meta-grid">
              <span>Version: {{ clusterVersion(cluster) }}</span>
              <span>VPC: <span class="mono">{{ clusterVpc(cluster) }}</span></span>
              <span>Subnet: <span class="mono">{{ clusterSubnet(cluster) }}</span></span>
            </div>
            <div class="ccex-actions">
              <button
                class="ghost minor"
                :disabled="!clusterId(cluster)"
                @click="emit('select-cluster', clusterId(cluster))"
              >
                {{ clusterId(cluster) === selectedClusterId ? "Managing" : "Manage" }}
              </button>
              <TrashIconButton
                :disabled="!clusterId(cluster) || deletingClusterId === clusterId(cluster)"
                :loading="deletingClusterId === clusterId(cluster)"
                :title="deletingClusterId === clusterId(cluster) ? 'Deleting cluster...' : 'Delete cluster'"
                @click="emit('delete-cluster', cluster)"
              />
            </div>
          </article>
        </div>
        <p v-else class="ccex-hint">No CCE clusters found for this region.</p>
      </section>

      <section class="ccex-card">
        <div class="ccex-row-between">
          <h3>Node Pools</h3>
          <ReloadIconButton
            :disabled="loadingNodePools || !hasSelectedCluster"
            :loading="loadingNodePools"
            :title="loadingNodePools ? 'Reloading CCE node pools...' : 'Reload CCE node pools'"
            @click="emit('reload-node-pools')"
          />
        </div>
        <p class="ccex-subtitle">
          Cluster:
          <span class="mono">{{ selectedClusterLabel }}</span>
        </p>

        <div v-if="hasSelectedCluster" class="ccex-grid-2">
          <label class="ccex-input-group">
            <span>Node Pool Name</span>
            <input v-model="nodePoolNameModel" class="ccex-input" placeholder="workload-pool" />
          </label>
          <label class="ccex-input-group">
            <span>Flavor</span>
            <div class="ccex-inline-input">
              <select v-model="nodePoolFlavorModel" class="ccex-input mono">
                <option value="" disabled>Select a flavor</option>
                <option v-for="item in nodePoolFlavorOptions" :key="item.id" :value="item.id">
                  {{ item.label }}
                </option>
              </select>
              <ReloadIconButton
                :disabled="loadingNodePoolFlavors"
                :loading="loadingNodePoolFlavors"
                :title="loadingNodePoolFlavors ? 'Reloading node flavors...' : 'Reload node flavors'"
                @click="emit('reload-node-pool-flavors')"
              />
            </div>
          </label>
          <label class="ccex-input-group">
            <span>Availability Zone</span>
            <select v-model="nodePoolAvailabilityZoneModel" class="ccex-input mono">
              <option value="" disabled>Select an availability zone</option>
              <option v-for="item in nodePoolAvailabilityZones" :key="item" :value="item">
                {{ item }}
              </option>
            </select>
          </label>
          <label class="ccex-input-group">
            <span>OS</span>
            <select v-model="nodePoolOsModel" class="ccex-input">
              <option v-for="item in nodePoolOsOptions" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group ccex-span-2">
            <span>SSH Key Pair (optional)</span>
            <input v-model="nodePoolSshKeyModel" class="ccex-input mono" placeholder="my-keypair" />
          </label>
          <label class="ccex-input-group">
            <span>Initial Nodes</span>
            <input v-model.number="nodePoolInitialCountModel" class="ccex-input" type="number" min="0" />
          </label>
          <label class="ccex-input-group">
            <span>Max Pods / Node</span>
            <input v-model.number="nodePoolMaxPodsModel" class="ccex-input" type="number" min="16" max="256" />
          </label>
          <label class="ccex-input-group">
            <span>Root Volume Type</span>
            <select v-model="nodePoolRootVolumeTypeModel" class="ccex-input">
              <option v-for="item in nodeVolumeTypes" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group">
            <span>Root Volume Size (GB)</span>
            <input v-model.number="nodePoolRootVolumeSizeModel" class="ccex-input" type="number" min="40" max="1024" />
          </label>
          <label class="ccex-input-group">
            <span>Data Volume Type</span>
            <select v-model="nodePoolDataVolumeTypeModel" class="ccex-input">
              <option v-for="item in nodeVolumeTypes" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group">
            <span>Data Volume Size (GB)</span>
            <input v-model.number="nodePoolDataVolumeSizeModel" class="ccex-input" type="number" min="100" max="32768" />
          </label>
        </div>
        <div v-if="hasSelectedCluster" class="ccex-actions">
          <button class="primary" :disabled="!canCreateNodePool" @click="emit('create-node-pool')">
            {{ creatingNodePool ? "Creating..." : "Create Node Pool" }}
          </button>
        </div>
        <p v-if="hasSelectedCluster" class="ccex-hint">
          Tip: use a flavor/AZ pair supported in this region and cluster network.
        </p>

        <div v-if="hasSelectedCluster && nodePools.length" class="ccex-list">
          <article v-for="pool in nodePools" :key="nodePoolId(pool) || nodePoolName(pool)" class="ccex-list-item">
            <div class="ccex-row-between ccex-item-head">
              <div>
                <div class="ccex-item-title mono">{{ nodePoolName(pool) }}</div>
                <div class="ccex-meta">ID: <span class="mono">{{ nodePoolId(pool) || "—" }}</span></div>
              </div>
              <span class="status-pill status-muted">{{ nodePoolPhase(pool) }}</span>
            </div>
            <div class="ccex-meta">Version: {{ nodePoolVersion(pool) }}</div>
            <div class="ccex-actions">
              <TrashIconButton
                :disabled="!nodePoolId(pool) || deletingNodePoolId === nodePoolId(pool)"
                :loading="deletingNodePoolId === nodePoolId(pool)"
                :title="
                  deletingNodePoolId === nodePoolId(pool) ? 'Deleting node pool...' : 'Delete node pool'
                "
                @click="emit('delete-node-pool', pool)"
              />
            </div>
          </article>
        </div>
        <p v-else-if="hasSelectedCluster" class="ccex-hint">No node pools found for this cluster.</p>
        <p v-else class="ccex-hint">Select a cluster in Management to inspect its node pools.</p>
      </section>

      <section v-if="hasSelectedCluster" class="ccex-card">
        <div class="ccex-row-between">
          <h3>Cluster Access</h3>
          <ReloadIconButton
            :disabled="loadingAccessEips || bindingAccessEip || apiEipBindRequested"
            :loading="loadingAccessEips"
            :title="loadingAccessEips ? 'Reloading EIPs...' : 'Reload EIPs'"
            @click="emit('reload-access-eips')"
          />
        </div>
        <p class="ccex-subtitle">
          Cluster:
          <span class="mono">{{ selectedClusterLabel }}</span>
        </p>
        <p class="ccex-hint">
          API EIP: <span class="mono">{{ selectedClusterExternalIp || "not bound" }}</span>
        </p>
        <p class="ccex-hint">{{ clusterAccessStateText }}</p>
        <p class="ccex-hint">Known EIPs in region: {{ accessEips.length }}</p>

        <div class="ccex-actions">
          <button
            class="primary"
            :disabled="!canCreateBindAccessEip"
            @click="emit('create-bind-cluster-access-eip')"
          >
            {{ clusterAccessActionText }}
          </button>
          <button class="ghost minor" :disabled="!canDownloadExternalKubeconfig" @click="emit('download-kubeconfig')">
            {{ downloadingKubeconfig ? "Preparing..." : "Download Kubeconfig" }}
          </button>
        </div>
        <p class="ccex-hint">
          Use this kubeconfig locally with <span class="mono">kubectl --kubeconfig &lt;file&gt; get nodes</span>.
        </p>
        <p v-if="!selectedClusterExternalIp" class="ccex-hint">
          Kubeconfig download unlocks after API EIP is bound.
        </p>
      </section>

      <section class="ccex-card">
        <div class="ccex-row-between">
          <h3>NAT Gateway</h3>
          <ReloadIconButton
            :disabled="loadingNatGateways || !clusterVpcId || !clusterSubnetId"
            :loading="loadingNatGateways"
            :title="loadingNatGateways ? 'Reloading NAT gateways...' : 'Reload NAT gateways'"
            @click="emit('reload-nat-gateways')"
          />
        </div>
        <p class="ccex-subtitle">
          VPC: <span class="mono">{{ clusterVpcId || "—" }}</span> • Subnet:
          <span class="mono">{{ clusterSubnetId || "—" }}</span>
        </p>
        <p class="ccex-hint">
          Create NAT and the app auto-creates an EIP plus SNAT rule for the selected subnet.
        </p>

        <div class="ccex-grid-2">
          <label class="ccex-input-group">
            <span>NAT Name</span>
            <input v-model="natGatewayNameModel" class="ccex-input" placeholder="cce-nat-gateway" />
          </label>
          <label class="ccex-input-group">
            <span>Spec</span>
            <select v-model="natGatewaySpecModel" class="ccex-input">
              <option v-for="item in natGatewaySpecs" :key="item" :value="item">{{ item }}</option>
            </select>
          </label>
          <label class="ccex-input-group ccex-span-2">
            <span>Description (optional)</span>
            <input
              v-model="natGatewayDescriptionModel"
              class="ccex-input"
              placeholder="NAT for selected CCE network"
            />
          </label>
        </div>

        <button class="primary" :disabled="!canCreateNatGateway" @click="emit('create-nat-gateway')">
          {{ creatingNatGateway ? "Creating..." : "Create NAT Gateway" }}
        </button>
        <p v-if="natGateways.length" class="ccex-hint">
          NAT gateway already exists for this selected network. Create is disabled to keep a single gateway.
        </p>

        <div v-if="natGateways.length" class="ccex-list">
          <article
            v-for="gateway in natGateways"
            :key="natGatewayId(gateway) || natGatewayNameText(gateway)"
            class="ccex-list-item"
          >
            <div class="ccex-row-between ccex-item-head">
              <div>
                <div class="ccex-item-title mono">{{ natGatewayNameText(gateway) }}</div>
                <div class="ccex-meta">ID: <span class="mono">{{ natGatewayId(gateway) || "—" }}</span></div>
              </div>
              <span class="status-pill status-neutral">{{ natGatewayStatus(gateway) }}</span>
            </div>
            <div class="ccex-meta-grid">
              <span>Spec: {{ textValue(gateway.spec) }}</span>
              <span>Created: {{ textValue(gateway.created_at) }}</span>
            </div>
            <div class="ccex-actions">
              <TrashIconButton
                :disabled="!natGatewayId(gateway) || deletingNatGatewayId === natGatewayId(gateway)"
                :loading="deletingNatGatewayId === natGatewayId(gateway)"
                :title="
                  deletingNatGatewayId === natGatewayId(gateway)
                    ? 'Deleting NAT gateway...'
                    : 'Delete NAT gateway'
                "
                @click="emit('delete-nat-gateway', gateway)"
              />
            </div>
          </article>
        </div>
        <p v-else class="ccex-hint">No NAT gateway found for this selected CCE network.</p>
      </section>
    </section>
  </div>
</template>

<style scoped>
.ccex-shell {
  display: grid;
  grid-template-columns: minmax(340px, 420px) minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.ccex-sidebar,
.ccex-main {
  display: grid;
  gap: 12px;
  min-width: 0;
}

.ccex-card {
  border: 1px solid #efc2c7;
  border-radius: 14px;
  background: #fffafb;
  padding: 10px;
  display: grid;
  gap: 8px;
  min-width: 0;
}

.ccex-card h3 {
  margin: 0;
  font-size: 1rem;
  color: #45121d;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
}

.ccex-row-between {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.ccex-grid-2 {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.ccex-grid-2 > * {
  min-width: 0;
}

.ccex-span-2 {
  grid-column: 1 / -1;
}

.ccex-input-group {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.ccex-input-group > span {
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: #7f1d2d;
  text-transform: uppercase;
}

.ccex-input {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
  min-height: 34px;
  padding: 0 10px;
  border-radius: 9px;
  border: 1px solid #e6bdc4;
  background: #fff;
  color: #3f1820;
  font-size: 0.84rem;
}

.ccex-textarea {
  min-height: 84px;
  padding: 10px;
  resize: vertical;
  line-height: 1.35;
}

.ccex-inline-input {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 6px;
  align-items: center;
}

.ccex-top-cta {
  min-height: 34px;
  min-width: 140px;
}

.ccex-subgroup {
  border: 1px solid #efccd1;
  border-radius: 11px;
  background: #fff;
  padding: 8px;
  display: grid;
  gap: 8px;
}

.ccex-subgroup-head {
  color: #7f1d2d;
  font-size: 0.82rem;
}

.ccex-alert {
  border-radius: 10px;
  padding: 8px 10px;
  font-size: 0.82rem;
  font-weight: 600;
  border: 1px solid transparent;
}

.ccex-alert-error {
  color: #b42318;
  background: #fef3f2;
  border-color: #fecdca;
}

.ccex-alert-info {
  color: #6b3841;
  background: #fff2f5;
  border-color: #f1c6cd;
}

.ccex-subtitle,
.ccex-hint,
.ccex-meta {
  margin: 0;
  font-size: 0.74rem;
  color: #7c4e56;
}

.ccex-list {
  display: grid;
  gap: 8px;
}

.ccex-list-item {
  border: 1px solid #efccd1;
  border-radius: 11px;
  background: #fff;
  padding: 8px;
  display: grid;
  gap: 6px;
  min-width: 0;
}

.ccex-list-item.selected {
  border-color: rgba(166, 31, 44, 0.55);
  box-shadow: inset 0 0 0 1px rgba(166, 31, 44, 0.16);
}

.ccex-item-head {
  align-items: flex-start;
}

.ccex-item-title {
  font-size: 0.86rem;
  font-weight: 700;
  color: #4f1521;
}

.ccex-meta-grid {
  display: grid;
  gap: 2px;
  font-size: 0.74rem;
  color: #7c4e56;
}

.ccex-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.ccex-polling-row {
  display: grid;
  gap: 8px;
}

.ccex-actions button {
  flex: 1 1 96px;
}

.ccex-actions :deep(.icon-trash) {
  flex: 0 0 auto;
  margin-left: auto;
}

.ccex-status-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
}

.ccex-raw-details summary {
  cursor: pointer;
  font-size: 0.78rem;
  font-weight: 600;
}

.ccex-raw {
  margin: 8px 0 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  background: #0f172a;
  color: #e2e8f0;
  border-radius: 10px;
  padding: 10px;
  font-size: 0.78rem;
  max-height: 240px;
  overflow: auto;
}

@media (max-width: 1320px) {
  .ccex-shell {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .ccex-grid-2 {
    grid-template-columns: 1fr;
  }

  .ccex-row-between {
    align-items: center;
    flex-direction: row;
    flex-wrap: wrap;
  }

  .ccex-row-between > :last-child {
    margin-left: auto;
  }
}
</style>
