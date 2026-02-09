<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { load } from '@tauri-apps/plugin-store';

type VpcOption = { id: string; name: string };
type SubnetOption = { id: string; name: string; cidr: string };
type ImageOption = { id: string; name: string; min_disk?: number | null };
type FlavorOption = { id: string; name: string };
type CreateEcsResult = { status: string; status_code: number; body: string };
type CredentialsPayload = { accessKey: string; secretKey: string };

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

const region = ref("sa-brazil-1");
const name = ref("");
const imageId = ref("");
const imageSearch = ref("");
const imageVisibility = ref("public");
const imageType = ref("gold");
const useCustomName = ref(false);
const flavorId = ref("");
const flavorSearch = ref("");
const rootVolumeType = ref("SATA");
const rootVolumeSize = ref(40);
const allocateEip = ref(false);
const accessKey = ref("");
const secretKey = ref("");

const vpcs = ref<VpcOption[]>([]);
const subnets = ref<SubnetOption[]>([]);
const images = ref<ImageOption[]>([]);
const flavors = ref<FlavorOption[]>([]);
const selectedVpc = ref("");
const selectedSubnet = ref("");

const loadingVpcs = ref(false);
const loadingSubnets = ref(false);
const loadingImages = ref(false);
const loadingFlavors = ref(false);
const savingCredentials = ref(false);
const loadingAll = ref(false);
const creating = ref(false);

const errorMsg = ref("");
const result = ref<CreateEcsResult | null>(null);

let store: Awaited<ReturnType<typeof load>> | null = null;
const storeReady = ref(false);

const canLoadSubnets = computed(() => !!selectedVpc.value && !loadingSubnets.value);
const canCreate = computed(
  () =>
    !!imageId.value &&
    !!flavorId.value &&
    !!selectedVpc.value &&
    !!selectedSubnet.value &&
    (!useCustomName.value || !!name.value.trim()) &&
    !creating.value
);

const canListImages = computed(() => !!region.value && !loadingImages.value);
const canListFlavors = computed(() => !!region.value && !loadingFlavors.value);
const imageMinDisk = computed(() => {
  const image = images.value.find((item) => item.id === imageId.value);
  const minDisk = image?.min_disk ?? 1;
  return Math.min(Math.max(minDisk, 1), 1024);
});
const filteredImages = computed(() => {
  const query = imageSearch.value.trim().toLowerCase();
  if (!query) {
    return images.value;
  }
  return images.value.filter((image) => {
    const name = image.name.toLowerCase();
    return name.includes(query) || image.id.toLowerCase().includes(query);
  });
});
const filteredFlavors = computed(() => {
  const query = flavorSearch.value.trim().toLowerCase();
  if (!query) {
    return flavors.value;
  }
  return flavors.value.filter((flavor) =>
    flavor.name.toLowerCase().includes(query)
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

function errorToString(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }
  return String(err);
}

// Build credentials payload when the user provides AK/SK in the UI.
function buildCredentialsPayload(): CredentialsPayload | null {
  const ak = accessKey.value.trim();
  const sk = secretKey.value.trim();

  if (!ak && !sk) {
    return null;
  }

  if (!ak || !sk) {
    throw new Error(
      "Provide both Access Key and Secret Key, or leave both blank to use env vars."
    );
  }

  return { accessKey: ak, secretKey: sk };
}

// Pull the latest VPC list for the chosen region.
async function loadVpcs() {
  loadingVpcs.value = true;
  errorMsg.value = "";
  result.value = null;

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<VpcOption[]>("list_vpcs", args);
    vpcs.value = data;

    if (!data.find((vpc) => vpc.id === selectedVpc.value)) {
      selectedVpc.value = data[0]?.id ?? "";
    }

    // VPC changes invalidate subnets, so reset them.
    subnets.value = [];
    selectedSubnet.value = "";
  } catch (err) {
    errorMsg.value = `Failed to load VPCs: ${errorToString(err)}`;
  } finally {
    loadingVpcs.value = false;
  }
}

// Pull subnets for the current VPC selection.
async function loadSubnets() {
  if (!selectedVpc.value) {
    errorMsg.value = "Select a VPC before loading subnets.";
    return;
  }

  loadingSubnets.value = true;
  errorMsg.value = "";
  result.value = null;

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

    if (!data.find((subnet) => subnet.id === selectedSubnet.value)) {
      selectedSubnet.value = data[0]?.id ?? "";
    }
  } catch (err) {
    errorMsg.value = `Failed to load subnets: ${errorToString(err)}`;
  } finally {
    loadingSubnets.value = false;
  }
}

// Pull images for the chosen region.
async function loadImages() {
  loadingImages.value = true;
  errorMsg.value = "";
  result.value = null;

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
    if (Object.keys(filters).length > 0) {
      args.filters = filters;
    }

    const data = await invoke<ImageOption[]>("list_images", args);
    images.value = data;

    if (!data.find((image) => image.id === imageId.value)) {
      imageId.value = data[0]?.id ?? "";
    }
  } catch (err) {
    errorMsg.value = `Failed to load images: ${errorToString(err)}`;
  } finally {
    loadingImages.value = false;
  }
}

// Pull flavors for the chosen region.
async function loadFlavors() {
  loadingFlavors.value = true;
  errorMsg.value = "";
  result.value = null;

  try {
    const credentials = buildCredentialsPayload();
    const args: Record<string, unknown> = { region: region.value };

    if (credentials) {
      args.credentials = credentials;
    }

    const data = await invoke<FlavorOption[]>("list_flavors", args);
    flavors.value = data;

    if (!data.find((flavor) => flavor.id === flavorId.value)) {
      flavorId.value = data[0]?.id ?? "";
    }
  } catch (err) {
    errorMsg.value = `Failed to load flavors: ${errorToString(err)}`;
  } finally {
    loadingFlavors.value = false;
  }
}

async function loadAll() {
  if (loadingAll.value) {
    return;
  }
  loadingAll.value = true;
  try {
    await loadImages();
    await loadFlavors();
    await loadVpcs();
    if (selectedVpc.value) {
      await loadSubnets();
    }
  } finally {
    loadingAll.value = false;
  }
}

async function saveCredentials() {
  if (!store) {
    errorMsg.value = "Credential store is not ready yet.";
    return;
  }
  savingCredentials.value = true;
  try {
    await store.set("accessKey", accessKey.value);
    await store.set("secretKey", secretKey.value);
  } finally {
    savingCredentials.value = false;
  }
}

async function initStore() {
  try {
    store = await load("store.json", { autoSave: true });
    accessKey.value = (await store.get<string>("accessKey")) ?? "";
    secretKey.value = (await store.get<string>("secretKey")) ?? "";
    storeReady.value = true;
  } catch (err) {
    errorMsg.value = `Failed to load credential store: ${errorToString(err)}`;
  }
  await loadAll();
}

onMounted(() => {
  initStore();
});

watch(region, () => {
  loadAll();
});

watch([imageVisibility, imageType], () => {
  loadImages();
});

// Send the ECS create request using the same inputs as the old CLI.
async function createEcs() {
  if (!imageId.value || !flavorId.value) {
    errorMsg.value = "Image ID and Flavor ID are required.";
    return;
  }

  if (!selectedVpc.value || !selectedSubnet.value) {
    errorMsg.value = "Select a VPC and subnet before creating the server.";
    return;
  }

  creating.value = true;
  errorMsg.value = "";
  result.value = null;

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
    };

    const args: Record<string, unknown> = { params: payload };

    if (credentials) {
      args.credentials = credentials;
    }

    result.value = await invoke<CreateEcsResult>("create_ecs", args);
  } catch (err) {
    errorMsg.value = `Create failed: ${errorToString(err)}`;
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
          Shared credentials across Huawei Cloud services. Leave blank to use
          `HWC_AK` and `HWC_SK`.
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
          <span class="muted tiny">Stored locally on this device.</span>
        </div>
      </div>
    </header>

    <section class="hero service-hero">
      <div>
        <p class="eyebrow">Service</p>
        <h2>Elastic Cloud Server</h2>
        <p class="subtitle">
          Build and launch ECS instances with images, flavors, and network
          wiring.
        </p>
      </div>
      <div class="chip">ECS Module</div>
    </section>

    <div class="layout">
      <section class="panel">
        <h2>Server Inputs</h2>
        <div class="grid">
          <label class="field">
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
            <span v-if="!useCustomName" class="muted tiny">
              Auto-generated when disabled.
            </span>
          </div>

          <div class="field span-2">
            <span>Image ({{ filteredImages.length }}/{{ images.length }})</span>
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
            <span>Flavor</span>
            <div class="combo">
              <input v-model="flavorSearch" placeholder="Search flavors..." />
              <select v-model="flavorId">
                <option value="" disabled>Select a flavor</option>
                <option
                  v-for="flavor in filteredFlavors"
                  :key="flavor.id"
                  :value="flavor.id"
                >
                  {{ flavor.name }}
                </option>
              </select>
            </div>
          </div>

          <label class="field">
            <span>Root Volume Type</span>
            <select v-model="rootVolumeType" :disabled="!imageId">
              <option value="SATA">SATA (Common I/O)</option>
              <option value="SAS">SAS (High I/O)</option>
              <option value="GPSSD">GPSSD (General Purpose SSD)</option>
              <option value="SSD">Ultra-I/O SSD (Ultra I/O)</option>
              <option value="ESSD">ESSD (Extreme SSD)</option>
              <option value="GPSSD2">GPSSD2 (General Purpose SSD V2)</option>
              <option value="ESSD2">ESSD2 (Extreme SSD V2)</option>
            </select>
          </label>

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
        </div>

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

        <div class="actions minor-actions">
          <button class="ghost minor" :disabled="loadingAll" @click="loadAll">
            {{ loadingAll ? "Reloading All..." : "Reload All" }}
          </button>
          <button
            class="ghost minor"
            :disabled="!canListImages"
            @click="loadImages"
          >
            {{ loadingImages ? "Reloading Images..." : "Reload Images" }}
          </button>
          <button
            class="ghost minor"
            :disabled="!canListFlavors"
            @click="loadFlavors"
          >
            {{ loadingFlavors ? "Reloading Flavors..." : "Reload Flavors" }}
          </button>
        </div>
        <p class="muted" v-if="loadingAll">
          Reloading images, flavors, VPCs, and subnets...
        </p>
        <p class="muted" v-else>
          Images: {{ images.length }} • Flavors: {{ flavors.length }} • VPCs:
          {{ vpcs.length }} • Subnets: {{ subnets.length }}
        </p>

        <div class="toggle">
          <input id="eip" v-model="allocateEip" type="checkbox" />
          <label for="eip">Allocate public EIP</label>
        </div>

        <div class="divider"></div>

        <h2>Network</h2>
        <div class="grid">
          <label class="field">
            <span>VPC</span>
            <select v-model="selectedVpc">
              <option value="" disabled>Select a VPC</option>
              <option v-for="vpc in vpcs" :key="vpc.id" :value="vpc.id">
                {{ vpc.name }}
              </option>
            </select>
          </label>

          <label class="field">
            <span>Subnet</span>
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

        <div class="actions">
          <button class="ghost minor" :disabled="loadingVpcs" @click="loadVpcs">
            {{ loadingVpcs ? "Reloading VPCs..." : "Reload VPCs" }}
          </button>
          <button
            class="ghost minor"
            :disabled="!canLoadSubnets"
            @click="loadSubnets"
          >
            {{ loadingSubnets ? "Reloading Subnets..." : "Reload Subnets" }}
          </button>
          <button class="primary cta" :disabled="!canCreate" @click="createEcs">
            {{ creating ? "Creating..." : "Create ECS" }}
          </button>
        </div>
      </section>

      <section class="panel output">
        <h2>Response</h2>
        <p v-if="errorMsg" class="error">{{ errorMsg }}</p>
        <div v-else-if="result" class="status">
          <span class="badge">{{ result.status_code }}</span>
          <span>{{ result.status }}</span>
        </div>
        <p v-else class="muted">No response yet. Run an action to see output.</p>

        <pre v-if="result" class="body">{{ result.body }}</pre>
      </section>
    </div>
  </main>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=Space+Grotesk:wght@500;600;700&display=swap");

:root {
  --ink: #101828;
  --muted: #667085;
  --panel: #ffffff;
  --panel-border: rgba(15, 23, 42, 0.08);
  --accent: #0f766e;
  --accent-strong: #0b5f59;
  --accent-warm: #ef6f48;
  --bg: #f6f5f2;
  --bg-strong: #f0ede7;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: var(--ink);
  background: var(--bg);
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  background: radial-gradient(circle at top, #ffffff 0%, var(--bg-strong) 45%, #e8eef2 100%);
  min-height: 100vh;
}

#app {
  min-height: 100vh;
}

.page {
  max-width: 1240px;
  margin: 0 auto;
  padding: 48px 24px 72px;
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.hero {
  display: flex;
  gap: 24px;
  align-items: center;
  justify-content: space-between;
}

.eyebrow {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.2em;
  margin: 0 0 8px;
  font-weight: 600;
}

.topbar {
  display: flex;
  gap: 24px;
  align-items: flex-start;
  justify-content: space-between;
  padding: 24px 26px;
  border-radius: 18px;
  background: #0f172a;
  color: #f8fafc;
  box-shadow: 0 18px 30px rgba(15, 23, 42, 0.24);
}

.brand {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.topbar h1 {
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 2.3rem;
  margin: 0 0 8px;
}

.subtitle {
  margin: 0;
  max-width: 520px;
  color: rgba(255, 255, 255, 0.82);
}

.chip {
  padding: 10px 16px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.12);
  border: 1px solid rgba(255, 255, 255, 0.22);
  font-weight: 600;
  text-align: center;
}

.credentials-card {
  min-width: 360px;
  background: rgba(248, 250, 252, 0.98);
  color: var(--ink);
  border-radius: 14px;
  padding: 14px 16px;
  border: 1px solid rgba(148, 163, 184, 0.35);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.8);
}

.cred-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.cred-actions {
  margin-top: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.mini-field {
  display: grid;
  gap: 6px;
  font-weight: 600;
}

.mini-field span {
  font-size: 0.8rem;
  color: #475467;
}

.service-hero {
  padding: 18px 22px;
  border-radius: 18px;
  background: linear-gradient(135deg, #1f2937 0%, #111827 100%);
  color: #f9fafb;
  box-shadow: 0 16px 26px rgba(15, 23, 42, 0.22);
}

.service-hero h2 {
  margin: 0 0 6px;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 1.8rem;
}

.service-hero .subtitle {
  color: rgba(249, 250, 251, 0.78);
}

.layout {
  display: grid;
  grid-template-columns: 1fr;
  gap: 24px;
}

.panel {
  background: var(--panel);
  border-radius: 18px;
  padding: 24px;
  box-shadow: 0 18px 30px rgba(15, 23, 42, 0.08);
  border: 1px solid var(--panel-border);
}

.panel h2 {
  margin: 0 0 16px;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 1.2rem;
}

.hint {
  margin: -6px 0 16px;
  color: var(--muted);
  font-size: 0.9rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.field {
  display: grid;
  gap: 8px;
  font-weight: 500;
}

.field span {
  font-size: 0.85rem;
  color: #5f6b72;
}

.span-2 {
  grid-column: span 2;
}

.combo {
  display: grid;
  gap: 10px;
}

.toggle-inline {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  color: #5f6b72;
}

input,
select {
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid #d4d0c7;
  background: #fff;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
  font-size: 0.95rem;
  transition: border-color 0.2s, box-shadow 0.2s;
}

input:focus,
select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(31, 138, 112, 0.18);
}

input:disabled,
select:disabled {
  background: #f3f4f6;
  color: #9ca3af;
}

input[type="range"] {
  padding: 0;
  accent-color: var(--accent);
}

.range-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 120px;
  gap: 12px;
  align-items: center;
}

.range-meta {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
  color: #5f6b72;
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
  background: linear-gradient(90deg, transparent, rgba(15, 118, 110, 0.4), transparent);
  margin: 24px 0;
}

.advanced {
  margin-top: 16px;
  padding: 14px 16px;
  border-radius: 14px;
  background: #f8fafb;
  border: 1px dashed rgba(148, 163, 184, 0.6);
}

.advanced-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
  font-weight: 600;
  color: #344054;
}

.minor-grid .field span {
  font-size: 0.8rem;
  color: #64748b;
}

.actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.minor-actions {
  margin-top: 14px;
}

button {
  border: none;
  border-radius: 12px;
  padding: 12px 14px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.15s ease, box-shadow 0.2s ease, opacity 0.2s;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  transform: none;
  box-shadow: none;
}

.primary {
  background: linear-gradient(135deg, var(--accent-strong) 0%, var(--accent) 100%);
  color: #fff;
  box-shadow: 0 18px 28px rgba(15, 118, 110, 0.28);
}

.ghost {
  background: #f2f4f7;
  border: 1px solid rgba(148, 163, 184, 0.6);
  color: var(--ink);
}

.minor {
  padding: 8px 12px;
  font-size: 0.85rem;
  font-weight: 600;
  background: #f8fafb;
  border-color: rgba(148, 163, 184, 0.4);
}

.cta {
  grid-column: 1 / -1;
  font-size: 1rem;
  padding: 14px 18px;
}

.output {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status {
  display: flex;
  align-items: center;
  gap: 12px;
  font-weight: 600;
}

.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 6px 12px;
  border-radius: 999px;
  background: rgba(31, 138, 112, 0.12);
  color: #0f766e;
  font-weight: 700;
}

.body {
  white-space: pre-wrap;
  background: #0f172a;
  color: #e2e8f0;
  border-radius: 14px;
  padding: 16px;
  font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
  font-size: 0.85rem;
  max-height: 260px;
  overflow: auto;
}

.error {
  color: #b42318;
  background: #fef3f2;
  border: 1px solid #fecdca;
  padding: 10px 12px;
  border-radius: 10px;
  font-weight: 600;
}

.muted {
  color: var(--muted);
}

.tiny {
  font-size: 0.75rem;
}

@media (max-width: 980px) {
  .grid {
    grid-template-columns: 1fr;
  }

  .actions {
    grid-template-columns: 1fr;
  }

  .topbar,
  .service-hero {
    flex-direction: column;
    align-items: flex-start;
  }

  .cred-grid {
    grid-template-columns: 1fr;
  }

  .range-row {
    grid-template-columns: 1fr;
  }
}
</style>
