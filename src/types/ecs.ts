export type VpcOption = { id: string; name: string };
export type SubnetOption = {
  id: string;
  name: string;
  cidr: string;
  availability_zone?: string | null;
};
export type ImageOption = {
  id: string;
  name: string;
  min_disk?: number | null;
  min_ram?: number | null;
};
export type FlavorOption = {
  id: string;
  name: string;
  vcpus?: number | null;
  ram?: number | null;
  disk?: number | null;
  os_extra_specs?: Record<string, string>;
};
export type EipVnic = {
  private_ip_address?: string | null;
  device_id?: string | null;
  vpc_id?: string | null;
  port_id?: string | null;
  instance_id?: string | null;
};
export type EipBandwidth = {
  size?: number | null;
  share_type?: string | null;
  charge_mode?: string | null;
};
export type EipRecord = {
  id?: string | null;
  public_ip_address?: string | null;
  status?: string | null;
  associate_instance_id?: string | null;
  associate_instance_type?: string | null;
  publicip_pool_name?: string | null;
  vnic?: EipVnic | null;
  bandwidth?: EipBandwidth | null;
};
export type EipListResponse = {
  publicips?: EipRecord[];
  total_count?: number | null;
};
export type EvsAttachment = {
  id?: string | null;
  server_id?: string | null;
  device?: string | null;
  attached_at?: string | null;
};
export type EvsVolume = {
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
export type EvsListResponse = {
  volumes?: EvsVolume[];
  count?: number | null;
};
export type EcsFlavorInfo = {
  name?: string | null;
  id?: string | null;
  vcpus?: number | null;
  ram?: number | null;
};
export type EcsServer = {
  id?: string | null;
  name?: string | null;
  status?: string | null;
  availability_zone?: string | null;
  flavor?: EcsFlavorInfo | null;
  created?: string | null;
};
export type EcsListResponse = {
  servers?: EcsServer[];
};
export type CreateEcsResult = { status: string; status_code: number; body: string };
export type ServiceModule = "ecs" | "obs" | "cce";
export type CredentialsPayload = { accessKey: string; secretKey: string };
export type DeleteOperationResult = {
  status: string;
  status_code?: number | null;
  body: string;
};
export type DeleteEcsResult = {
  ecs: DeleteOperationResult;
  eip?: DeleteOperationResult | null;
};
export type StopEcsResult = {
  ecs: DeleteOperationResult;
};
export type SshConnectResult = {
  sessionId: string;
  host: string;
  port: number;
  username: string;
  connectedAt: string;
};
export type SshExecResult = {
  sessionId: string;
  command: string;
  stdout: string;
  stderr: string;
  exitStatus?: number | null;
};
export type SshDisconnectResult = {
  sessionId: string;
  disconnected: boolean;
};
export type SshResizeResult = {
  sessionId: string;
  cols: number;
  rows: number;
};
export type SshSendControlResult = {
  sessionId: string;
  control: string;
  sent: boolean;
};
export type SshExecOneShotResult = {
  sessionId: string;
  host: string;
  port: number;
  username: string;
  command: string;
  stdout: string;
  stderr: string;
  exitStatus?: number | null;
};
export type SshStreamEventPayload = {
  sessionId: string;
  kind: "meta" | "stdout" | "stderr";
  text: string;
  at: string;
};
export type SshSessionInfo = SshConnectResult & {
  serverId: string;
  serverName: string;
};
export type SshTerminalEntry = {
  id: number;
  at: string;
  kind: "meta" | "command" | "stdout" | "stderr";
  text: string;
};
export type PlatformOpsTab = "docker" | "minikube" | "nix";
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
export type CachedEntry<T> = {
  updatedAt: string;
  data: T;
};
export type CachedResource =
  | "images"
  | "flavors"
  | "vpcs"
  | "subnets"
  | "eips"
  | "evss"
  | "ecses";
export type FlavorGroup = {
  key: string;
  label: string;
  flavors: FlavorOption[];
};
export type LogSource = "app" | "backend" | "runtime";
export type LogLevelName = "trace" | "debug" | "info" | "warn" | "error";
export type AppLogEntry = {
  id: number;
  at: string;
  source: LogSource;
  level: LogLevelName;
  message: string;
};
export type ConfirmDialogKind = "info" | "warning" | "error";
export type ConfirmDialogState = {
  open: boolean;
  title: string;
  message: string;
  kind: ConfirmDialogKind;
  okLabel: string;
  cancelLabel: string;
};
export type AutoUpdateProgressInfo = {
  sessionId: string | null;
  startedAt: string | null;
  finishedAt: string | null;
  percent: number | null;
  lastLine: string | null;
};
export type StartupTaskConfig = {
  region: string;
  autoUpdate: boolean;
  setupGuiRdp: boolean;
  rdpUsername: string | null;
  lastStatus: "pending" | "done" | "failed";
  createdAt: string;
  updatedAt: string;
};
export type PendingStartupTaskCreate = {
  config: StartupTaskConfig;
  password: string;
};
export type StoredServerPassword = {
  version: 1;
  saltB64: string;
  ivB64: string;
  cipherB64: string;
  updatedAt: string;
};
