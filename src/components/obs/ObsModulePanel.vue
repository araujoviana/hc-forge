<script setup lang="ts">
import { computed } from "vue";
import ReloadIconButton from "../ReloadIconButton.vue";
import TrashIconButton from "../TrashIconButton.vue";
import type { ObsBucket, ObsObject, ObsOperationResult } from "../../types/obs";

const props = defineProps<{
  region: string;
  regions: readonly string[];
  bucketName: string;
  bucketNameError: string | null;
  defaultStorageClass: string;
  bucketAcl: string;
  storageClasses: readonly string[];
  bucketAcls: readonly string[];
  canCreateBucket: boolean;
  creatingBucket: boolean;
  selectedBucket: string;
  selectedBucketRecord: ObsBucket | null;
  buckets: ObsBucket[];
  loadingBuckets: boolean;
  deletingBucket: string | null;
  uploadObjectKey: string;
  uploadContentType: string;
  resolvedUploadContentType: string;
  singlePutLimitLabel: string;
  canUploadObject: boolean;
  uploadingObject: boolean;
  uploadProgress: number | null;
  objectPrefix: string;
  objectMarker: string;
  objectMaxKeys: number;
  maxKeysMin: number;
  maxKeysMax: number;
  canLoadObjects: boolean;
  loadingObjects: boolean;
  objects: ObsObject[];
  bucketTotalSizeBytes: number | null;
  bucketTotalObjectCount: number | null;
  loadingBucketTotals: boolean;
  bucketTotalsError: string | null;
  deletingObject: string | null;
  downloadingObject: string | null;
  downloadProgress: number | null;
  lastResult: ObsOperationResult | null;
  errorMsg: string;
  quickCopyFeedback: string | null;
  formatObsObjectSize: (size: number | null | undefined) => string;
}>();

const emit = defineEmits<{
  (e: "update:region", value: string): void;
  (e: "update:bucketName", value: string): void;
  (e: "update:defaultStorageClass", value: string): void;
  (e: "update:bucketAcl", value: string): void;
  (e: "update:uploadObjectKey", value: string): void;
  (e: "update:uploadContentType", value: string): void;
  (e: "update:objectPrefix", value: string): void;
  (e: "update:objectMarker", value: string): void;
  (e: "update:objectMaxKeys", value: number): void;
  (e: "create-bucket"): void;
  (e: "reload-buckets"): void;
  (e: "select-bucket", value: string): void;
  (e: "copy-bucket-name", value: string): void;
  (e: "delete-bucket", value: ObsBucket): void;
  (e: "upload-file", value: File | null): void;
  (e: "upload-object"): void;
  (e: "reload-objects"): void;
  (e: "search-objects"): void;
  (e: "download-object", value: string): void;
  (e: "copy-object-key", value: string): void;
  (e: "delete-object", value: string): void;
}>();

const regionModel = computed({
  get: () => props.region,
  set: (value: string) => emit("update:region", value),
});
const bucketNameModel = computed({
  get: () => props.bucketName,
  set: (value: string) => emit("update:bucketName", value),
});
const defaultStorageClassModel = computed({
  get: () => props.defaultStorageClass,
  set: (value: string) => emit("update:defaultStorageClass", value),
});
const bucketAclModel = computed({
  get: () => props.bucketAcl,
  set: (value: string) => emit("update:bucketAcl", value),
});
const uploadObjectKeyModel = computed({
  get: () => props.uploadObjectKey,
  set: (value: string) => emit("update:uploadObjectKey", value),
});
const uploadContentTypeModel = computed({
  get: () => props.uploadContentType,
  set: (value: string) => emit("update:uploadContentType", value),
});
const objectPrefixModel = computed({
  get: () => props.objectPrefix,
  set: (value: string) => emit("update:objectPrefix", value),
});
const objectMarkerModel = computed({
  get: () => props.objectMarker,
  set: (value: string) => emit("update:objectMarker", value),
});
const objectMaxKeysModel = computed({
  get: () => props.objectMaxKeys,
  set: (value: number) => emit("update:objectMaxKeys", value),
});

function handleUploadFileChange(event: Event) {
  const input = event.target as HTMLInputElement | null;
  emit("upload-file", input?.files?.[0] ?? null);
}
</script>

<template>
  <div class="obsx-shell">
    <aside class="obsx-sidebar">
      <section class="obsx-card">
        <div class="obsx-row-between">
          <h3>Create Bucket</h3>
          <button class="primary obsx-top-cta" :disabled="!canCreateBucket" @click="emit('create-bucket')">
            {{ creatingBucket ? "Creating..." : "Create Bucket" }}
          </button>
        </div>

        <div class="obsx-grid-2">
          <label class="obsx-input-group">
            <span>Region</span>
            <select v-model="regionModel" class="obsx-input">
              <option v-for="item in regions" :key="item" :value="item">
                {{ item }}
              </option>
            </select>
          </label>

          <label class="obsx-input-group">
            <span>Bucket Name</span>
            <input v-model="bucketNameModel" class="obsx-input" spellcheck="false" placeholder="my-obs-bucket" />
          </label>

          <label class="obsx-input-group">
            <span>Default Storage Class</span>
            <select v-model="defaultStorageClassModel" class="obsx-input">
              <option v-for="storageClass in storageClasses" :key="storageClass" :value="storageClass">
                {{ storageClass }}
              </option>
            </select>
          </label>

          <label class="obsx-input-group">
            <span>Bucket ACL</span>
            <select v-model="bucketAclModel" class="obsx-input">
              <option v-for="acl in bucketAcls" :key="acl" :value="acl">
                {{ acl }}
              </option>
            </select>
          </label>
        </div>
        <p v-if="bucketNameError" class="obsx-error-text">{{ bucketNameError }}</p>
        <p v-else class="obsx-hint">3-63 chars, lowercase letters, numbers, dots, and dashes.</p>
      </section>

      <section class="obsx-card">
        <div class="obsx-row-between">
          <h3>Upload Object</h3>
          <span class="obsx-limit">Max single PUT: {{ singlePutLimitLabel }}</span>
        </div>
        <p class="obsx-hint">
          Target:
          <span class="mono obsx-target-value">{{ selectedBucketRecord?.name ?? "none selected" }}</span>
        </p>
        <div class="obsx-grid-1">
          <label class="obsx-input-group">
            <span>File</span>
            <input class="obsx-input" type="file" @change="handleUploadFileChange" />
          </label>
          <label class="obsx-input-group">
            <span>Object Key</span>
            <input v-model="uploadObjectKeyModel" class="obsx-input" placeholder="folder/file.txt" />
          </label>
          <label class="obsx-input-group">
            <span>MIME Type (optional)</span>
            <input v-model="uploadContentTypeModel" class="obsx-input" placeholder="auto from file if empty" />
          </label>
        </div>
        <p class="obsx-hint">
          Effective MIME:
          <span class="mono">{{ resolvedUploadContentType }}</span>
        </p>
        <button class="primary obsx-full-btn" :disabled="!canUploadObject" @click="emit('upload-object')">
          {{ uploadingObject ? "Uploading..." : "Upload Object" }}
        </button>
        <div v-if="uploadProgress !== null" class="obsx-progress-wrap">
          <div class="obsx-progress-label">Upload progress: {{ uploadProgress }}%</div>
          <div class="obsx-progress" aria-hidden="true">
            <div class="obsx-progress-bar" :style="{ width: `${uploadProgress}%` }"></div>
          </div>
        </div>
      </section>

      <section class="obsx-card">
        <h3>Last OBS Action</h3>
        <div v-if="lastResult" class="obsx-last-action">
          <div class="obsx-status-row">
            <span class="badge">{{ lastResult.status_code }}</span>
            <span>{{ lastResult.status }}</span>
          </div>
          <details class="obsx-raw-details">
            <summary>Raw response</summary>
            <pre class="obsx-raw">{{ lastResult.body }}</pre>
          </details>
        </div>
        <p v-else class="obsx-hint">No OBS action yet.</p>
      </section>
    </aside>

    <section class="obsx-main">
      <div v-if="errorMsg" class="obsx-alert obsx-alert-error">{{ errorMsg }}</div>
      <div v-else-if="quickCopyFeedback" class="obsx-alert obsx-alert-info">{{ quickCopyFeedback }}</div>

      <div class="obsx-main-grid">
        <section class="obsx-card obsx-buckets-card">
          <div class="obsx-row-between">
            <h3>Buckets</h3>
            <ReloadIconButton
              :disabled="loadingBuckets"
              :loading="loadingBuckets"
              :title="loadingBuckets ? 'Reloading OBS buckets...' : 'Reload buckets'"
              @click="emit('reload-buckets')"
            />
          </div>
          <div class="obsx-subtitle">{{ buckets.length }} total (account-wide, globally named)</div>

          <div v-if="buckets.length" class="obsx-list">
            <article
              v-for="bucket in buckets"
              :key="bucket.name"
              class="obsx-list-item"
              :class="{ selected: bucket.name === selectedBucket }"
            >
              <div class="obsx-row-between obsx-item-head">
                <div class="obsx-title-row">
                  <span class="obsx-item-title mono">{{ bucket.name }}</span>
                  <button class="ghost minor obsx-mini-btn" @click="emit('copy-bucket-name', bucket.name)">
                    Copy
                  </button>
                </div>
                <span class="status-pill status-neutral">{{ bucket.bucket_type ?? "OBJECT" }}</span>
              </div>
              <div class="obsx-meta">
                {{ bucket.location ?? region }}
                <template v-if="bucket.creation_date"> • {{ bucket.creation_date }}</template>
              </div>
              <div class="obsx-actions">
                <button
                  class="ghost minor"
                  :disabled="loadingObjects && selectedBucket === bucket.name"
                  @click="emit('select-bucket', bucket.name)"
                >
                  {{ selectedBucket === bucket.name ? "Managing" : "Manage" }}
                </button>
                <TrashIconButton
                  :disabled="deletingBucket === bucket.name"
                  :loading="deletingBucket === bucket.name"
                  :title="deletingBucket === bucket.name ? 'Deleting bucket...' : 'Delete bucket'"
                  @click="emit('delete-bucket', bucket)"
                />
              </div>
            </article>
          </div>
          <p v-else class="obsx-hint">No OBS buckets found for this account.</p>
        </section>

        <section class="obsx-card obsx-objects-card">
          <div class="obsx-row-between">
            <h3>Objects</h3>
            <ReloadIconButton
              :disabled="!canLoadObjects"
              :loading="loadingObjects"
              :title="loadingObjects ? 'Reloading objects...' : 'Reload objects'"
              @click="emit('reload-objects')"
            />
          </div>

          <div class="obsx-search-panel">
            <div class="obsx-row-between obsx-search-head">
              <strong>Search Objects</strong>
              <span class="obsx-subtitle">
                Bucket:
                <span class="mono">{{ selectedBucketRecord?.name ?? "none selected" }}</span>
              </span>
            </div>
            <p
              v-if="selectedBucket"
              class="obsx-subtitle obsx-usage-summary"
              :class="{ 'obsx-usage-summary-error': !!bucketTotalsError }"
            >
              <template v-if="loadingBucketTotals">Calculating total stored usage...</template>
              <template v-else-if="bucketTotalsError">{{ bucketTotalsError }}</template>
              <template v-else-if="bucketTotalSizeBytes !== null">
                Total stored:
                <span class="mono">{{ formatObsObjectSize(bucketTotalSizeBytes) }}</span>
                • {{ bucketTotalObjectCount ?? 0 }} object(s)
              </template>
              <template v-else>Usage unavailable for this bucket.</template>
            </p>
            <div class="obsx-search-grid">
              <input v-model="objectPrefixModel" class="obsx-input" placeholder="Prefix filter (optional)" />
              <input v-model="objectMarkerModel" class="obsx-input" placeholder="Marker (optional)" />
              <input
                v-model.number="objectMaxKeysModel"
                class="obsx-input"
                type="number"
                :min="maxKeysMin"
                :max="maxKeysMax"
                step="1"
              />
              <button class="primary obsx-search-btn" :disabled="!canLoadObjects" @click="emit('search-objects')">
                Search!
              </button>
            </div>
          </div>

          <div v-if="selectedBucket && objects.length" class="obsx-list">
            <article v-for="object in objects" :key="object.key" class="obsx-list-item">
              <div class="obsx-row-between obsx-item-head">
                <div>
                  <div class="obsx-item-title mono">{{ object.key }}</div>
                  <div class="obsx-meta">
                    {{ object.last_modified ?? "unknown timestamp" }} •
                    {{ object.storage_class ?? "STANDARD" }}
                  </div>
                </div>
                <span class="status-pill status-muted">{{ formatObsObjectSize(object.size) }}</span>
              </div>

              <div class="obsx-actions">
                <button
                  class="ghost minor"
                  :disabled="downloadingObject === object.key"
                  @click="emit('download-object', object.key)"
                >
                  {{ downloadingObject === object.key ? "Downloading..." : "Download" }}
                </button>
                <button class="ghost minor" @click="emit('copy-object-key', object.key)">Copy Key</button>
                <TrashIconButton
                  :disabled="deletingObject === object.key"
                  :loading="deletingObject === object.key"
                  :title="deletingObject === object.key ? 'Deleting object...' : 'Delete object'"
                  @click="emit('delete-object', object.key)"
                />
              </div>

              <div v-if="downloadingObject === object.key && downloadProgress !== null" class="obsx-progress-wrap">
                <div class="obsx-progress-label">Download progress: {{ downloadProgress }}%</div>
                <div class="obsx-progress" aria-hidden="true">
                  <div class="obsx-progress-bar" :style="{ width: `${downloadProgress}%` }"></div>
                </div>
              </div>
            </article>
          </div>
          <p v-else-if="selectedBucket" class="obsx-hint">No objects found for this bucket and current filters.</p>
          <p v-else class="obsx-hint">Select a bucket and press Search! to list objects.</p>
        </section>
      </div>
    </section>
  </div>
</template>

<style scoped>
.obsx-shell {
  display: grid;
  grid-template-columns: minmax(320px, 380px) minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.obsx-sidebar,
.obsx-main {
  display: grid;
  gap: 12px;
  min-width: 0;
}

.obsx-main-grid {
  display: grid;
  grid-template-columns: minmax(300px, 0.9fr) minmax(0, 1.4fr);
  gap: 12px;
  align-items: start;
}

.obsx-card {
  border: 1px solid #efc2c7;
  border-radius: 14px;
  background: #fffafb;
  padding: 10px;
  display: grid;
  gap: 8px;
  min-width: 0;
}

.obsx-card h3 {
  margin: 0;
  font-size: 1rem;
  color: #45121d;
  font-family: "Space Grotesk", "IBM Plex Sans", sans-serif;
}

.obsx-row-between {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.obsx-grid-2 {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.obsx-grid-2 > *,
.obsx-grid-1 > *,
.obsx-search-grid > * {
  min-width: 0;
}

.obsx-grid-1 {
  display: grid;
  gap: 8px;
}

.obsx-input-group {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.obsx-input-group > span {
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: #7f1d2d;
  text-transform: uppercase;
}

.obsx-input {
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

.obsx-top-cta {
  min-height: 34px;
  min-width: 130px;
}

.obsx-full-btn {
  min-height: 36px;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  box-sizing: border-box;
}

.obsx-limit {
  font-size: 0.72rem;
  color: #7c4e56;
}

.obsx-hint {
  margin: 0;
  font-size: 0.74rem;
  color: #7c4e56;
}

.obsx-target-value {
  display: block;
  margin-top: 2px;
  max-width: 100%;
  white-space: normal;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.obsx-error-text {
  margin: 0;
  font-size: 0.74rem;
  color: #b42318;
  font-weight: 600;
}

.obsx-alert {
  border-radius: 10px;
  padding: 8px 10px;
  font-size: 0.82rem;
  font-weight: 600;
  border: 1px solid transparent;
}

.obsx-alert-error {
  color: #b42318;
  background: #fef3f2;
  border-color: #fecdca;
}

.obsx-alert-info {
  color: #6b3841;
  background: #fff2f5;
  border-color: #f1c6cd;
}

.obsx-subtitle {
  font-size: 0.76rem;
  color: #7c4e56;
}

.obsx-list {
  display: grid;
  gap: 8px;
}

.obsx-list-item {
  border: 1px solid #efccd1;
  border-radius: 11px;
  background: #fff;
  padding: 8px;
  display: grid;
  gap: 6px;
  min-width: 0;
}

.obsx-list-item.selected {
  border-color: rgba(166, 31, 44, 0.55);
  box-shadow: inset 0 0 0 1px rgba(166, 31, 44, 0.16);
}

.obsx-item-head {
  align-items: flex-start;
  min-width: 0;
}

.obsx-title-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  flex: 1 1 auto;
}

.obsx-item-title {
  display: block;
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.86rem;
  font-weight: 700;
  color: #4f1521;
}

.obsx-mini-btn {
  flex: 0 0 auto;
  min-height: 24px;
  font-size: 0.7rem;
  padding: 0 8px;
}

.obsx-item-head .status-pill {
  flex: 0 0 auto;
}

.obsx-meta {
  font-size: 0.74rem;
  color: #7c4e56;
}

.obsx-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.obsx-actions button {
  flex: 1 1 96px;
}

.obsx-actions :deep(.icon-trash) {
  flex: 0 0 auto;
  margin-left: auto;
}

.obsx-search-panel {
  border: 1px solid rgba(166, 31, 44, 0.34);
  border-radius: 11px;
  background: linear-gradient(180deg, rgba(255, 244, 245, 0.94), rgba(255, 249, 250, 0.95));
  padding: 8px;
  display: grid;
  gap: 8px;
}

.obsx-search-head {
  align-items: flex-start;
}

.obsx-usage-summary {
  margin: 0;
}

.obsx-usage-summary-error {
  color: #b42318;
}

.obsx-search-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr) 110px auto;
  gap: 8px;
  align-items: center;
}

.obsx-search-btn {
  min-height: 34px;
  padding: 0 12px;
}

.obsx-progress-wrap {
  display: grid;
  gap: 4px;
}

.obsx-progress-label {
  font-size: 0.74rem;
  color: #7c4e56;
}

.obsx-progress {
  width: 100%;
  height: 7px;
  border-radius: 999px;
  background: rgba(159, 58, 71, 0.16);
  border: 1px solid rgba(159, 58, 71, 0.24);
  overflow: hidden;
}

.obsx-progress-bar {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #a61f2c, #c32936);
  transition: width 0.2s ease;
}

.obsx-raw-details summary {
  cursor: pointer;
  font-size: 0.78rem;
  font-weight: 600;
}

.obsx-raw {
  margin: 8px 0 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  background: #0f172a;
  color: #e2e8f0;
  border-radius: 10px;
  padding: 10px;
  font-size: 0.78rem;
  max-height: 220px;
  overflow: auto;
}

@media (max-width: 1280px) {
  .obsx-shell {
    grid-template-columns: 1fr;
  }

  .obsx-main-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .obsx-grid-2,
  .obsx-search-grid {
    grid-template-columns: 1fr;
  }

  .obsx-row-between {
    align-items: center;
    flex-direction: row;
    flex-wrap: wrap;
  }

  .obsx-row-between > :last-child {
    margin-left: auto;
  }
}
</style>
