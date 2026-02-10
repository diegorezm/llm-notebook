import { invoke } from "@tauri-apps/api/core";
import { type Result, ok, err } from "./utils";

export type MessageRole = "user" | "assistant" | "system";

export interface Notebook {
  id: string;
  title: string;
  created_at: number;
  last_accessed: number;
}

export interface ChatEntry {
  id: string;
  notebook_id: string;
  role: MessageRole;
  message: string;
  timestamp: number;
}

export interface Attachment {
  id: string;
  notebookId: string;
  file_name: string;
  file_path: string;
  file_size: number;
  file_type: string;
  created_at: number;
}

export interface AppError {
  reason: string;
}

// Caller
async function call<T>(cmd: string, args?: any): Promise<Result<T, AppError>> {
  try {
    const res = await invoke<T>(cmd, args);
    return ok(res);
  } catch (e) {
    // Ensuring error follows the { reason: string } constraint
    const reason =
      typeof e === "string" ? e : ((e as any)?.reason ?? "Unknown error");
    return err({ reason });
  }
}

// Tauri commands
export async function createNotebook(
  title: string,
): Promise<Result<Notebook, AppError>> {
  return call<Notebook>("create_notebook", { title });
}

export async function getNotebooks(): Promise<Result<Notebook[], AppError>> {
  return call<Notebook[]>("get_notebooks");
}

export async function sendMessage(
  notebookId: string,
  message: string,
): Promise<Result<ChatEntry, AppError>> {
  return call<ChatEntry>("send_message", { notebookId, message });
}

export async function getChatHistory(
  notebookId: string,
): Promise<Result<ChatEntry[], AppError>> {
  return call<ChatEntry[]>("get_chat_history", { notebookId });
}

export async function deleteNotebook(
  notebookId: string,
): Promise<Result<null, AppError>> {
  return call<null>("delete_notebook", { notebookId });
}

export async function getAttachments(
  notebookId: string,
): Promise<Result<Attachment[], AppError>> {
  return call<Attachment[]>("get_attachments", { notebookId });
}

export async function uploadFile(
  notebookId: string,
): Promise<Result<Attachment, AppError>> {
  return call<Attachment>("upload_file", { notebookId });
}

export async function deleteAttachment(
  id: string,
): Promise<Result<null, AppError>> {
  return call<null>("delete_attachment", { id });
}
