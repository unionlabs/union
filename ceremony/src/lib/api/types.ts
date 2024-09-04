export interface Status {
  status:
    | 'idle'
    | 'initializing'
    | 'contributionStarted'
    | 'contributionEnded'
    | 'successful';
  downloadStarted?: string;
  downloading?: {
    file: string;
    progress: number;
  };
  downloadEnded?: string;
  uploadStarted?: string;
  uploadEnded?: string;
  failed?: string;
}

export interface ContributeBody {
  supabase_project: string;
  bucket: string;
  jwt: string;
  api_key: string;
  contributor_id: string;
  payload_id: string;
}
