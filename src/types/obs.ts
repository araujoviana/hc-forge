export type ObsBucket = {
  name: string;
  creation_date?: string | null;
  location?: string | null;
  bucket_type?: string | null;
};

export type ObsListBucketsResponse = {
  buckets?: ObsBucket[];
};

export type ObsObject = {
  key: string;
  last_modified?: string | null;
  etag?: string | null;
  size?: number | null;
  storage_class?: string | null;
};

export type ObsListObjectsResponse = {
  bucket?: string | null;
  prefix?: string | null;
  marker?: string | null;
  next_marker?: string | null;
  is_truncated?: boolean | null;
  objects?: ObsObject[];
};

export type ObsOperationResult = {
  status: string;
  status_code: number;
  body: string;
};

export type ObsGetObjectResult = {
  status: string;
  status_code: number;
  content_base64?: string | null;
  content_type?: string | null;
  body?: string | null;
};
