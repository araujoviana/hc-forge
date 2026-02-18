<script setup lang="ts">
import { computed } from "vue";
import ReloadIconButton from "../ReloadIconButton.vue";
import type { FlavorGroup, FlavorOption, ImageOption, SubnetOption, VpcOption } from "../../types/ecs";

const props = defineProps<{
  region: string;
  regions: readonly string[];
  canCreate: boolean;
  creating: boolean;
  useCustomName: boolean;
  name: string;
  autoUpdateVmOnStartup: boolean;
  setupGuiRdpOnStartup: boolean;
  filteredImages: ImageOption[];
  images: ImageOption[];
  canListImages: boolean;
  loadingImages: boolean;
  imageSearch: string;
  imageId: string;
  filteredFlavors: FlavorOption[];
  flavors: FlavorOption[];
  canListFlavors: boolean;
  loadingFlavors: boolean;
  flavorSearch: string;
  flavorArchFilter: string;
  flavorVcpuFilter: string;
  flavorId: string;
  flavorArchitectureOptions: string[];
  flavorGroups: FlavorGroup[];
  formatFlavorLabel: (flavor: FlavorOption) => string;
  imageMinRam: number;
  passwordSectionOpen: boolean;
  passwordCopyFeedback: string | null;
  useGeneratedPassword: boolean;
  generatedPassword: string;
  showAdminPassword: boolean;
  customPassword: string;
  passwordError: string | null;
  passwordMinLength: number;
  passwordMaxLength: number;
  storageSectionOpen: boolean;
  rootVolumeType: string;
  allocateEip: boolean;
  eipBandwidthSize: number;
  eipBandwidthMin: number;
  eipBandwidthMax: number;
  rootVolumeSize: number;
  imageMinDisk: number;
  includeDataDisk: boolean;
  dataDiskType: string;
  dataDiskSize: number;
  dataDiskCount: number;
  dataDiskMultiattach: boolean;
  dataDiskHwPassthrough: boolean;
  dataDiskDefaultSize: number;
  dataDiskMin: number;
  dataDiskMax: number;
  dataDiskMinCount: number;
  dataDiskMaxCount: number;
  imageFilterSectionOpen: boolean;
  imageVisibility: string;
  imageType: string;
  networkSectionOpen: boolean;
  loadingVpcs: boolean;
  loadingSubnets: boolean;
  canLoadSubnets: boolean;
  selectedVpc: string;
  selectedSubnet: string;
  vpcs: VpcOption[];
  subnets: SubnetOption[];
  loadingAll: boolean;
  cacheAge: {
    images: string;
    flavors: string;
    vpcs: string;
    subnets: string;
    evss: string;
  };
  evssLength: number;
}>();

const emit = defineEmits<{
  (e: "update:region", value: string): void;
  (e: "update:useCustomName", value: boolean): void;
  (e: "update:name", value: string): void;
  (e: "update:autoUpdateVmOnStartup", value: boolean): void;
  (e: "update:setupGuiRdpOnStartup", value: boolean): void;
  (e: "update:imageSearch", value: string): void;
  (e: "update:imageId", value: string): void;
  (e: "update:flavorSearch", value: string): void;
  (e: "update:flavorArchFilter", value: string): void;
  (e: "update:flavorVcpuFilter", value: string): void;
  (e: "update:flavorId", value: string): void;
  (e: "update:passwordSectionOpen", value: boolean): void;
  (e: "update:useGeneratedPassword", value: boolean): void;
  (e: "update:customPassword", value: string): void;
  (e: "update:showAdminPassword", value: boolean): void;
  (e: "update:storageSectionOpen", value: boolean): void;
  (e: "update:rootVolumeType", value: string): void;
  (e: "update:allocateEip", value: boolean): void;
  (e: "update:eipBandwidthSize", value: number): void;
  (e: "update:rootVolumeSize", value: number): void;
  (e: "update:includeDataDisk", value: boolean): void;
  (e: "update:dataDiskType", value: string): void;
  (e: "update:dataDiskSize", value: number): void;
  (e: "update:dataDiskCount", value: number): void;
  (e: "update:dataDiskMultiattach", value: boolean): void;
  (e: "update:dataDiskHwPassthrough", value: boolean): void;
  (e: "update:imageFilterSectionOpen", value: boolean): void;
  (e: "update:imageVisibility", value: string): void;
  (e: "update:imageType", value: string): void;
  (e: "update:networkSectionOpen", value: boolean): void;
  (e: "update:selectedVpc", value: string): void;
  (e: "update:selectedSubnet", value: string): void;
  (e: "create"): void;
  (e: "load-images"): void;
  (e: "load-flavors"): void;
  (e: "regenerate-password"): void;
  (e: "copy-current-password"): void;
  (e: "load-vpcs"): void;
  (e: "load-subnets"): void;
  (e: "load-all"): void;
}>();

const regionModel = computed({
  get: () => props.region,
  set: (value: string) => emit("update:region", value),
});
const useCustomNameModel = computed({
  get: () => props.useCustomName,
  set: (value: boolean) => emit("update:useCustomName", value),
});
const nameModel = computed({
  get: () => props.name,
  set: (value: string) => emit("update:name", value),
});
const autoUpdateVmOnStartupModel = computed({
  get: () => props.autoUpdateVmOnStartup,
  set: (value: boolean) => emit("update:autoUpdateVmOnStartup", value),
});
const setupGuiRdpOnStartupModel = computed({
  get: () => props.setupGuiRdpOnStartup,
  set: (value: boolean) => emit("update:setupGuiRdpOnStartup", value),
});
const imageSearchModel = computed({
  get: () => props.imageSearch,
  set: (value: string) => emit("update:imageSearch", value),
});
const imageIdModel = computed({
  get: () => props.imageId,
  set: (value: string) => emit("update:imageId", value),
});
const flavorSearchModel = computed({
  get: () => props.flavorSearch,
  set: (value: string) => emit("update:flavorSearch", value),
});
const flavorArchFilterModel = computed({
  get: () => props.flavorArchFilter,
  set: (value: string) => emit("update:flavorArchFilter", value),
});
const flavorVcpuFilterModel = computed({
  get: () => props.flavorVcpuFilter,
  set: (value: string) => emit("update:flavorVcpuFilter", value),
});
const flavorIdModel = computed({
  get: () => props.flavorId,
  set: (value: string) => emit("update:flavorId", value),
});
const passwordSectionOpenModel = computed({
  get: () => props.passwordSectionOpen,
  set: (value: boolean) => emit("update:passwordSectionOpen", value),
});
const useGeneratedPasswordModel = computed({
  get: () => props.useGeneratedPassword,
  set: (value: boolean) => emit("update:useGeneratedPassword", value),
});
const customPasswordModel = computed({
  get: () => props.customPassword,
  set: (value: string) => emit("update:customPassword", value),
});
const showAdminPasswordModel = computed({
  get: () => props.showAdminPassword,
  set: (value: boolean) => emit("update:showAdminPassword", value),
});
const storageSectionOpenModel = computed({
  get: () => props.storageSectionOpen,
  set: (value: boolean) => emit("update:storageSectionOpen", value),
});
const rootVolumeTypeModel = computed({
  get: () => props.rootVolumeType,
  set: (value: string) => emit("update:rootVolumeType", value),
});
const allocateEipModel = computed({
  get: () => props.allocateEip,
  set: (value: boolean) => emit("update:allocateEip", value),
});
const eipBandwidthSizeModel = computed({
  get: () => props.eipBandwidthSize,
  set: (value: number) => emit("update:eipBandwidthSize", value),
});
const rootVolumeSizeModel = computed({
  get: () => props.rootVolumeSize,
  set: (value: number) => emit("update:rootVolumeSize", value),
});
const includeDataDiskModel = computed({
  get: () => props.includeDataDisk,
  set: (value: boolean) => emit("update:includeDataDisk", value),
});
const dataDiskTypeModel = computed({
  get: () => props.dataDiskType,
  set: (value: string) => emit("update:dataDiskType", value),
});
const dataDiskSizeModel = computed({
  get: () => props.dataDiskSize,
  set: (value: number) => emit("update:dataDiskSize", value),
});
const dataDiskCountModel = computed({
  get: () => props.dataDiskCount,
  set: (value: number) => emit("update:dataDiskCount", value),
});
const dataDiskMultiattachModel = computed({
  get: () => props.dataDiskMultiattach,
  set: (value: boolean) => emit("update:dataDiskMultiattach", value),
});
const dataDiskHwPassthroughModel = computed({
  get: () => props.dataDiskHwPassthrough,
  set: (value: boolean) => emit("update:dataDiskHwPassthrough", value),
});
const imageFilterSectionOpenModel = computed({
  get: () => props.imageFilterSectionOpen,
  set: (value: boolean) => emit("update:imageFilterSectionOpen", value),
});
const imageVisibilityModel = computed({
  get: () => props.imageVisibility,
  set: (value: string) => emit("update:imageVisibility", value),
});
const imageTypeModel = computed({
  get: () => props.imageType,
  set: (value: string) => emit("update:imageType", value),
});
const networkSectionOpenModel = computed({
  get: () => props.networkSectionOpen,
  set: (value: boolean) => emit("update:networkSectionOpen", value),
});
const selectedVpcModel = computed({
  get: () => props.selectedVpc,
  set: (value: string) => emit("update:selectedVpc", value),
});
const selectedSubnetModel = computed({
  get: () => props.selectedSubnet,
  set: (value: string) => emit("update:selectedSubnet", value),
});
</script>

<template>
  <section class="panel">
    <div class="panel-head">
      <h2>Server Inputs</h2>
      <button class="primary quick-create" :disabled="!canCreate" @click="emit('create')">
        {{ creating ? "Creating..." : "Create" }}
      </button>
    </div>
    <div class="grid inputs-grid">
      <label class="field region-field">
        <span>Region</span>
        <select v-model="regionModel">
          <option v-for="item in regions" :key="item" :value="item">
            {{ item }}
          </option>
        </select>
      </label>

      <div class="field">
        <span>Name</span>
        <div class="toggle-inline">
          <input id="custom-name" v-model="useCustomNameModel" type="checkbox" />
          <label for="custom-name">Use custom name</label>
        </div>
        <input
          v-model="nameModel"
          :disabled="!useCustomNameModel"
          placeholder="my-ecs-prod"
        />
      </div>

      <div class="field span-2 startup-update-field">
        <span>Startup Tasks (new VM only)</span>
        <div class="startup-task-toggles">
          <div class="toggle-inline">
            <input id="auto-update-vm" v-model="autoUpdateVmOnStartupModel" type="checkbox" />
            <label for="auto-update-vm">Update VM on startup</label>
          </div>
          <div class="toggle-inline">
            <input id="setup-gui-rdp" v-model="setupGuiRdpOnStartupModel" type="checkbox" />
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
            @click="emit('load-images')"
          />
        </div>
        <div class="combo">
          <input v-model="imageSearchModel" placeholder="Search images..." />
          <select v-model="imageIdModel">
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
            @click="emit('load-flavors')"
          />
        </div>
        <div class="combo">
          <input v-model="flavorSearchModel" placeholder="Search flavors..." />
          <div class="inline-pairs">
            <label class="mini-field">
              <span>Architecture</span>
              <select v-model="flavorArchFilterModel">
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
              <select v-model="flavorVcpuFilterModel">
                <option value="all">All</option>
                <option value="1-2">1-2</option>
                <option value="4-8">4-8</option>
                <option value="16+">16+</option>
              </select>
            </label>
          </div>
          <select v-model="flavorIdModel">
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
            @click="passwordSectionOpenModel = !passwordSectionOpenModel"
          >
            <span>Administrator Access</span>
            <span class="fold-state">{{ passwordSectionOpenModel ? "Hide" : "Show" }}</span>
          </button>
          <span v-if="passwordCopyFeedback" class="copy-feedback tiny">
            {{ passwordCopyFeedback }}
          </span>
          <button class="ghost minor action-chip fold-copy" type="button" @click="emit('copy-current-password')">
            Copy Password
          </button>
        </div>
        <transition name="fold">
          <div v-if="passwordSectionOpenModel" class="fold-body">
            <div class="field password-field">
              <div class="field-head">
                <span>Administrator Password</span>
                <button
                  class="ghost minor"
                  type="button"
                  :disabled="!useGeneratedPasswordModel"
                  @click="emit('regenerate-password')"
                >
                  Regenerate
                </button>
              </div>
              <div class="toggle-inline">
                <input
                  id="generated-password"
                  v-model="useGeneratedPasswordModel"
                  type="checkbox"
                />
                <label for="generated-password">
                  Use generated password (recommended)
                </label>
              </div>
              <div class="password-input-row">
                <input
                  v-if="useGeneratedPasswordModel"
                  :value="generatedPassword"
                  :type="showAdminPasswordModel ? 'text' : 'password'"
                  readonly
                  spellcheck="false"
                />
                <input
                  v-else
                  v-model="customPasswordModel"
                  :type="showAdminPasswordModel ? 'text' : 'password'"
                  placeholder="Enter your own admin password"
                  spellcheck="false"
                />
                <button
                  class="ghost minor eye-toggle"
                  type="button"
                  :aria-label="showAdminPasswordModel ? 'Hide password' : 'Show password'"
                  @click="showAdminPasswordModel = !showAdminPasswordModel"
                >
                  {{ showAdminPasswordModel ? "🙈" : "👁️" }}
                </button>
              </div>
              <div class="password-actions">
                <button class="ghost minor action-chip" type="button" @click="emit('copy-current-password')">
                  Copy Password
                </button>
                <span class="muted tiny">
                  {{
                    passwordCopyFeedback ??
                    `Must be ${passwordMinLength}-${passwordMaxLength} chars with upper/lower/number/symbol.`
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
          @click="storageSectionOpenModel = !storageSectionOpenModel"
        >
          <span>Storage And Public Network</span>
          <span class="fold-state">{{ storageSectionOpenModel ? "Hide" : "Show" }}</span>
        </button>
        <transition name="fold">
          <div v-if="storageSectionOpenModel" class="fold-body">
            <div class="grid">
              <label class="field">
                <span>Root Volume Type</span>
                <select v-model="rootVolumeTypeModel" :disabled="!imageIdModel">
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
                  <input id="eip" v-model="allocateEipModel" type="checkbox" />
                  <label for="eip">Allocate public EIP</label>
                </div>
                <label class="mini-field">
                  <span>EIP Bandwidth (Mbit/s)</span>
                  <input
                    v-model.number="eipBandwidthSizeModel"
                    type="number"
                    :min="eipBandwidthMin"
                    :max="eipBandwidthMax"
                    step="1"
                    :disabled="!allocateEipModel"
                  />
                </label>
                <span class="muted tiny">
                  Charge mode is fixed to traffic. Huawei ECS API allows
                  {{ eipBandwidthMin }}-{{ eipBandwidthMax }} Mbit/s.
                </span>
              </div>

              <div class="field span-2">
                <span>Root Volume Size (GB)</span>
                <div class="range-row">
                  <input
                    v-model.number="rootVolumeSizeModel"
                    type="range"
                    :min="imageMinDisk"
                    max="1024"
                    step="1"
                  />
                  <input
                    v-model.number="rootVolumeSizeModel"
                    type="number"
                    :min="imageMinDisk"
                    max="1024"
                  />
                </div>
                <div class="range-meta">
                  <span>{{ rootVolumeSizeModel }} GB</span>
                  <span class="muted">Min {{ imageMinDisk }} GB</span>
                </div>
              </div>

              <div class="field span-2">
                <span>EVS Data Disk (optional)</span>
                <div class="toggle-inline">
                  <input id="include-data-disk" v-model="includeDataDiskModel" type="checkbox" />
                  <label for="include-data-disk">Attach EVS data disk on create</label>
                </div>
                <div class="inline-pairs">
                  <label class="mini-field">
                    <span>Volume Type</span>
                    <select v-model="dataDiskTypeModel" :disabled="!includeDataDiskModel">
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
                      v-model.number="dataDiskSizeModel"
                      type="number"
                      :min="dataDiskMin"
                      :max="dataDiskMax"
                      :disabled="!includeDataDiskModel"
                    />
                  </label>
                  <label class="mini-field">
                    <span>Count</span>
                    <input
                      v-model.number="dataDiskCountModel"
                      type="number"
                      :min="dataDiskMinCount"
                      :max="dataDiskMaxCount"
                      :disabled="!includeDataDiskModel"
                    />
                  </label>
                  <div class="mini-field">
                    <span>Flags</span>
                    <div class="toggle-inline">
                      <input
                        id="data-disk-multiattach"
                        v-model="dataDiskMultiattachModel"
                        type="checkbox"
                        :disabled="!includeDataDiskModel"
                      />
                      <label for="data-disk-multiattach">Shareable (multiattach)</label>
                    </div>
                    <div class="toggle-inline">
                      <input
                        id="data-disk-scsi"
                        v-model="dataDiskHwPassthroughModel"
                        type="checkbox"
                        :disabled="!includeDataDiskModel"
                      />
                      <label for="data-disk-scsi">SCSI passthrough</label>
                    </div>
                  </div>
                </div>
                <span class="muted tiny">
                  Defaults: no data disk attached; when enabled uses {{ dataDiskDefaultSize }} GB
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
        @click="imageFilterSectionOpenModel = !imageFilterSectionOpenModel"
      >
        <span>Image Filters</span>
        <span class="fold-state">{{ imageFilterSectionOpenModel ? "Hide" : "Show" }}</span>
      </button>
      <transition name="fold">
        <div v-if="imageFilterSectionOpenModel" class="fold-body">
          <div class="advanced">
            <div class="advanced-header">
              <span>Image Filters (optional)</span>
              <span class="muted tiny">Usually keep defaults.</span>
            </div>
            <div class="grid minor-grid">
              <label class="field">
                <span>Visibility</span>
                <select v-model="imageVisibilityModel">
                  <option value="all">All</option>
                  <option value="public">Public</option>
                  <option value="private">Private</option>
                  <option value="shared">Shared</option>
                </select>
              </label>

              <label class="field">
                <span>Image Type</span>
                <select v-model="imageTypeModel">
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
        @click="networkSectionOpenModel = !networkSectionOpenModel"
      >
        <span>Network</span>
        <span class="fold-state">{{ networkSectionOpenModel ? "Hide" : "Show" }}</span>
      </button>
      <transition name="fold">
        <div v-if="networkSectionOpenModel" class="fold-body">
          <div class="grid">
            <label class="field">
              <div class="field-title-row">
                <span>VPC</span>
                <ReloadIconButton
                  :disabled="loadingVpcs"
                  :loading="loadingVpcs"
                  :title="loadingVpcs ? 'Reloading VPCs...' : 'Reload VPCs'"
                  @click="emit('load-vpcs')"
                />
              </div>
              <select v-model="selectedVpcModel">
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
                  @click="emit('load-subnets')"
                />
              </div>
              <select v-model="selectedSubnetModel">
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
      <button class="ghost minor" :disabled="loadingAll" @click="emit('load-all')">
        {{ loadingAll ? "Reloading All..." : "Reload All" }}
      </button>
    </div>

    <div class="bottom-create-row">
      <button class="primary cta bottom-create" :disabled="!canCreate" @click="emit('create')">
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
      {{ evssLength }} ({{ cacheAge.evss }})
    </p>
  </section>
</template>
