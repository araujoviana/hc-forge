<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

type VpcOption = { id: string; name: string };
type SubnetOption = { id: string; name: string; cidr: string };
type CreateEcsResult = { status: string; status_code: number; body: string };
type CredentialsPayload = { accessKey: string; secretKey: string };

const region = ref("sa-brazil-1");
const name = ref("ecs-<RANDOM-VALUE>");
const imageId = ref("");
const flavorId = ref("");
const rootVolumeType = ref("SATA");
const rootVolumeSize = ref(40);
const allocateEip = ref(false);
const accessKey = ref("");
const secretKey = ref("");

const vpcs = ref<VpcOption[]>([]);
const subnets = ref<SubnetOption[]>([]);
const selectedVpc = ref("");
const selectedSubnet = ref("");

const loadingVpcs = ref(false);
const loadingSubnets = ref(false);
const creating = ref(false);

const errorMsg = ref("");
const result = ref<CreateEcsResult | null>(null);

const canLoadSubnets = computed(() => !!selectedVpc.value && !loadingSubnets.value);
const canCreate = computed(
  () =>
    !!imageId.value &&
    !!flavorId.value &&
    !!selectedVpc.value &&
    !!selectedSubnet.value &&
    !creating.value
);

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
      name: name.value,
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
    <header class="hero">
      <div>
        <p class="eyebrow">HC Forge</p>
        <h1>Elastic Cloud Server</h1>
        <p class="subtitle">
          Provide AK/SK
          below or leave them blank to use `HWC_AK` and `HWC_SK` from the
          environment.
        </p>
      </div>
      <div class="chip">Region-driven, API-backed</div>
    </header>

    <div class="layout">
      <section class="panel">
        <h2>Credentials</h2>
        <p class="hint">
          These are only used for the current request and are not stored.
        </p>
        <div class="grid">
          <label class="field">
            <span>Access Key</span>
            <input
              v-model="accessKey"
              autocomplete="off"
              spellcheck="false"
              placeholder="AK..."
            />
          </label>

          <label class="field">
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

        <div class="divider"></div>

        <h2>Server Inputs</h2>
        <div class="grid">
          <label class="field">
            <span>Region</span>
            <input v-model="region" placeholder="sa-brazil-1" />
          </label>

          <label class="field">
            <span>Name</span>
            <input v-model="name" placeholder="ecs-&lt;RANDOM-VALUE&gt;" />
          </label>

          <label class="field">
            <span>Image ID</span>
            <input v-model="imageId" placeholder="Image UUID" />
          </label>

          <label class="field">
            <span>Flavor ID</span>
            <input v-model="flavorId" placeholder="Flavor UUID" />
          </label>

          <label class="field">
            <span>Root Volume Type</span>
            <input v-model="rootVolumeType" placeholder="SATA" />
          </label>

          <label class="field">
            <span>Root Volume Size (GB)</span>
            <input v-model.number="rootVolumeSize" type="number" min="1" />
          </label>
        </div>

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
          <button class="ghost" :disabled="loadingVpcs" @click="loadVpcs">
            {{ loadingVpcs ? "Loading VPCs..." : "Load VPCs" }}
          </button>
          <button
            class="ghost"
            :disabled="!canLoadSubnets"
            @click="loadSubnets"
          >
            {{ loadingSubnets ? "Loading Subnets..." : "Load Subnets" }}
          </button>
          <button class="primary" :disabled="!canCreate" @click="createEcs">
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
        <p v-else class="muted">No response yet. Load VPCs to begin.</p>

        <pre v-if="result" class="body">{{ result.body }}</pre>
      </section>
    </div>
  </main>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=Space+Grotesk:wght@500;600;700&display=swap");

:root {
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: #1c1f24;
  background: #f3efe7;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  background: radial-gradient(circle at top, #fff7e6 0%, #f3efe7 45%, #e9f0f5 100%);
  min-height: 100vh;
}

#app {
  min-height: 100vh;
}

.page {
  max-width: 1100px;
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
  padding: 24px 28px;
  border-radius: 20px;
  background: linear-gradient(135deg, #0f766e 0%, #1f8a70 45%, #ef6f48 100%);
  color: #fff;
  box-shadow: 0 24px 40px rgba(15, 23, 42, 0.2);
}

.eyebrow {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.2em;
  margin: 0 0 8px;
  font-weight: 600;
}

.hero h1 {
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 2.4rem;
  margin: 0 0 8px;
}

.subtitle {
  margin: 0;
  max-width: 520px;
  color: rgba(255, 255, 255, 0.85);
}

.chip {
  padding: 10px 16px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.18);
  border: 1px solid rgba(255, 255, 255, 0.4);
  font-weight: 600;
  text-align: center;
}

.layout {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 24px;
}

.panel {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 18px;
  padding: 24px;
  box-shadow: 0 20px 40px rgba(15, 23, 42, 0.08);
  border: 1px solid rgba(15, 118, 110, 0.12);
}

.panel h2 {
  margin: 0 0 16px;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
  font-size: 1.2rem;
}

.hint {
  margin: -6px 0 16px;
  color: #6b7280;
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
  border-color: #1f8a70;
  box-shadow: 0 0 0 3px rgba(31, 138, 112, 0.18);
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

.actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin-top: 16px;
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
  background: linear-gradient(135deg, #0f766e 0%, #1f8a70 100%);
  color: #fff;
  box-shadow: 0 16px 24px rgba(15, 118, 110, 0.25);
}

.ghost {
  background: #f3efe7;
  border: 1px solid #d4d0c7;
  color: #1c1f24;
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
  max-height: 420px;
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
  color: #6b7280;
}

@media (max-width: 980px) {
  .layout {
    grid-template-columns: 1fr;
  }

  .grid {
    grid-template-columns: 1fr;
  }

  .actions {
    grid-template-columns: 1fr;
  }

  .hero {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
