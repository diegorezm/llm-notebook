import {
  Component,
  createSignal,
  createResource,
  For,
  Show,
  onMount,
  onCleanup,
} from "solid-js";
import { ArrowLeft, FileText, CloudUpload, Trash2 } from "lucide-solid";
import { A, useParams } from "@solidjs/router";
import {
  getAttachments,
  uploadFile,
  deleteAttachment,
  ProcessingStatus,
} from "../../lib/commands";
import { showToast } from "../../lib/toast";
import { listen } from "@tauri-apps/api/event";

export const ResizableSidebar: Component = () => {
  const params = useParams<{ id: string }>();
  const [width, setWidth] = createSignal(280);
  let isResizing = false;

  const startResizing = (e: MouseEvent) => {
    isResizing = true;
    document.body.style.cursor = "col-resize";
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", stopResizing);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing) return;
    const newWidth = Math.min(Math.max(200, e.clientX), 600);
    setWidth(newWidth);
  };

  const stopResizing = () => {
    isResizing = false;
    document.body.style.cursor = "default";
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", stopResizing);
  };

  return (
    <div class="flex h-full overflow-hidden">
      <div style={{ width: `${width()}px` }} class="shrink-0">
        <FileSidebar notebookId={params.id} />
      </div>
      <div
        onMouseDown={startResizing}
        class="w-1 bg-transparent hover:bg-zinc-700 cursor-col-resize transition-colors z-50 h-full -ml-0.5"
      />
    </div>
  );
};

const FileSidebar: Component<{ notebookId: string }> = (props) => {
  const [processingMap, setProcessingMap] = createSignal<
    Record<string, ProcessingStatus>
  >({});

  // Fetch attachments from SQLite
  const [attachments, { refetch }] = createResource(
    () => props.notebookId,
    async (id) => {
      const [err, data] = await getAttachments(id);
      if (err) throw err;
      console.log(data);
      return data;
    },
  );

  onMount(async () => {
    const unlistenStart = await listen<string>("processing-start", (event) => {
      setProcessingMap((prev) => ({ ...prev, [event.payload]: "pending" }));
    });

    const unlistenSuccess = await listen<string>(
      "processing-success",
      (event) => {
        setProcessingMap((prev) => ({ ...prev, [event.payload]: "ready" }));
        refetch();
      },
    );

    const unlistenError = await listen<{ id: string; reason: string }>(
      "processing-error",
      (event) => {
        setProcessingMap((prev) => ({ ...prev, [event.payload.id]: "error" }));
        showToast({ message: `Failed to process file`, type: "error" });
      },
    );

    onCleanup(() => {
      unlistenStart();
      unlistenSuccess();
      unlistenError();
    });
  });

  const handleUpload = async () => {
    const [err, file] = await uploadFile(props.notebookId);

    if (err) {
      if (err.reason !== "No file selected") {
        showToast({ message: err.reason, type: "error" });
      }
      return;
    }

    setProcessingMap((prev) => ({ ...prev, [file.id]: "pending" }));
    showToast({ message: `Uploaded ${file!.file_name}`, type: "success" });
    refetch();
  };

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(`Delete ${name}?`)) return;
    const [err] = await deleteAttachment(id);
    if (err) {
      showToast({ message: err.reason, type: "error" });
    } else {
      refetch();
    }
  };

  return (
    <aside class="w-full h-full border-r border-zinc-800 flex flex-col bg-zinc-900/10">
      <div class="p-4 border-b border-zinc-800 flex items-center justify-between">
        <A
          href="/"
          class="btn btn-ghost btn-xs p-0 hover:bg-transparent text-zinc-500 hover:text-white"
        >
          <ArrowLeft size={16} />
        </A>
        <span class="text-sm font-medium uppercase tracking-widest text-zinc-500">
          Files
        </span>
        <div></div>
      </div>

      <div class="flex-1 overflow-y-auto p-2 space-y-1">
        <Show
          when={!attachments.loading}
          fallback={<div class="p-4 text-xs text-zinc-600">Loading...</div>}
        >
          <For each={attachments()}>
            {(file) => (
              <FileItem
                name={file.file_name}
                status={processingMap()[file.id] ?? file.status}
                onDelete={() => handleDelete(file.id, file.file_name)}
              />
            )}
          </For>
          <Show when={attachments()?.length === 0}>
            <div class="p-4 text-center text-[10px] text-zinc-600 uppercase tracking-tighter">
              No files yet
            </div>
          </Show>
        </Show>
      </div>

      <div class="p-4 border-t border-zinc-800">
        <button
          onClick={handleUpload}
          class="btn btn-outline border-zinc-700 hover:border-zinc-400 hover:bg-transparent text-zinc-400 hover:text-white w-full btn-sm rounded-sm text-[10px] tracking-widest uppercase"
        >
          <CloudUpload class="size-4" />
          Upload File
        </button>
      </div>
    </aside>
  );
};

const FileItem: Component<{
  name: string;
  status: ProcessingStatus;
  onDelete: () => void;
}> = (props) => (
  <div class="flex items-center gap-3 p-2 rounded-sm hover:bg-zinc-800/50 cursor-pointer group transition-colors">
    <FileText size={14} class="text-zinc-600 group-hover:text-zinc-300" />
    <span class="text-xs text-zinc-400 group-hover:text-zinc-200 truncate flex-1">
      {props.name}
    </span>

    <Show when={props.status === "pending"}>
      <span class="text-[9px] uppercase tracking-widest text-yellow-600 animate-pulse">
        Processing
      </span>
    </Show>
    <Show when={props.status === "error"}>
      <span class="text-[9px] uppercase tracking-widest text-red-500">
        Failed
      </span>
    </Show>

    <button
      onClick={(e) => {
        e.stopPropagation();
        props.onDelete();
      }}
      class="p-1 hover:text-red-400 text-zinc-500 transition-all"
    >
      <Trash2 size={12} />
    </button>
  </div>
);
