export type DockerImageSummary = {
  repository: string;
  tag: string;
  id: string;
  createdSince: string;
  size: string;
};

export type DockerContainerSummary = {
  name: string;
  image: string;
  status: string;
  ports: string;
  id: string;
};

export type NixPackageSummary = {
  name: string;
  version: string;
  source: string;
};

export type MinikubeSetupOptions = {
  installMinikube?: boolean;
  ensureDocker?: boolean;
  autoStart?: boolean;
  profile?: string;
  driver?: "docker" | "none";
  cpus?: number;
  memoryMb?: number;
  kubernetesVersion?: string;
};

export type DockerSetupOptions = {
  installDocker?: boolean;
  dockerfileContent?: string;
  dockerfileTargetPath?: string;
};

export type NixSetupOptions = {
  installNix?: boolean;
  enableFlakes?: boolean;
  runGarbageCollect?: boolean;
  packages?: string;
};

export type DockerfileTemplateOptions = {
  baseImage?: string;
  workdir?: string;
  exposePort?: string;
  startCommand?: string;
};

export function shellSingleQuote(value: unknown): string;

export const DEFAULT_PLATFORM_DOCKERFILE_PATH: string;

export function buildDockerSetupCommand(options?: DockerSetupOptions): string;
export function buildDockerImagesCommand(): string;
export function buildDockerContainersCommand(): string;
export function parseDockerImages(stdout: string): DockerImageSummary[];
export function parseDockerContainers(stdout: string): DockerContainerSummary[];
export function buildDockerfileTemplate(options?: DockerfileTemplateOptions): string;
export function parseDockerfileTemplate(content: string): {
  baseImage?: string;
  workdir?: string;
  exposePort?: string;
  startCommand?: string;
};

export function buildMinikubeSetupCommand(options?: MinikubeSetupOptions): string;
export function buildMinikubeStatusCommand(profile?: string): string;
export function buildMinikubeNodesCommand(profile?: string): string;
export function buildMinikubePodsCommand(profile?: string): string;
export function buildNixSetupCommand(options?: NixSetupOptions): string;
export function buildNixVersionCommand(): string;
export function buildNixPackagesCommand(): string;
export function buildNixStoreUsageCommand(): string;
export function parseNixPackages(output: string): NixPackageSummary[];
