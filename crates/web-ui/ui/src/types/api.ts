export type Priority = "Lowest" | "Low" | "Normal" | "High" | "Highest";
export type TaskState = "Pending" | "Downloading" | "Paused" | "Canceled" | "Completed" | "Failed";

export interface ProgressChunk {
  start: number;
  end: number;
  received: number;
  is_completed: boolean;
}

export interface TaskStatus {
  id: number;
  file_path: string;
  content_length: number;
  chunks: ProgressChunk[];
  concurrent_number: number;
  state: TaskState;
  priority: Priority;
  creation_timestamp: string;
}

export interface CreateTaskRequest {
  uri: string;
  priority: Priority;
  creation_timestamp: string;
  concurrent_number?: number;
  filename?: string;
  output_directory?: string;
}
