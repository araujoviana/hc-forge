export type CcePlainObject = Record<string, unknown>;

export type CceCluster = {
  kind?: string | null;
  apiVersion?: string | null;
  metadata?: CcePlainObject | null;
  spec?: CcePlainObject | null;
  status?: CcePlainObject | null;
};

export type CceClusterListResponse = {
  kind?: string | null;
  apiVersion?: string | null;
  items?: CceCluster[];
};

export type CceNodePool = {
  kind?: string | null;
  apiVersion?: string | null;
  metadata?: CcePlainObject | null;
  spec?: CcePlainObject | null;
  status?: CcePlainObject | null;
};

export type CceNodePoolListResponse = {
  kind?: string | null;
  apiVersion?: string | null;
  items?: CceNodePool[];
};

export type CceNatGateway = {
  id?: string | null;
  name?: string | null;
  description?: string | null;
  spec?: string | null;
  status?: string | null;
  router_id?: string | null;
  internal_network_id?: string | null;
  enterprise_project_id?: string | null;
  created_at?: string | null;
};

export type CceNatGatewayListResponse = {
  nat_gateways?: CceNatGateway[];
};

export type CceOperationResult = {
  status: string;
  status_code: number;
  body: string;
};

export type CceKubeconfigResult = {
  status: string;
  status_code: number;
  body: string;
  kubeconfig?: string | null;
};
