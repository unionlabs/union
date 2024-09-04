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
  supabaseProject: string;
  bucket: string;
  jwt: string;
  apiKey: string;
  contributorId: string;
  payloadId: string;
}
