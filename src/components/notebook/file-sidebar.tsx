import { Component, createSignal } from "solid-js";
import { ArrowLeft, Plus, FileText, CloudUpload } from "lucide-solid";
import { A } from "@solidjs/router";

export const ResizableSidebar: Component = () => {
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
    // Set boundaries (min 200px, max 600px)
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
        <FileSidebar />
      </div>

      <div
        onMouseDown={startResizing}
        class="w-1 bg-transparent hover:bg-zinc-700 cursor-col-resize transition-colors z-50 h-full -ml-0.5"
      />
    </div>
  );
};

const FileSidebar: Component = () => {
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
        <button class="btn btn-ghost btn-xs text-zinc-400 hover:text-white">
          <Plus size={16} />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-2 space-y-1">
        <FileItem name="physics_notes.pdf" />
      </div>

      <div class="p-4 border-t border-zinc-800">
        <button class="btn btn-outline border-zinc-700 hover:border-zinc-400 hover:bg-transparent text-zinc-400 hover:text-white w-full btn-sm rounded-sm text-[10px] tracking-widest uppercase">
          <CloudUpload class="size-4" />
          Upload File
        </button>
      </div>
    </aside>
  );
};

const FileItem: Component<{ name: string }> = (props) => (
  <div class="flex items-center gap-3 p-2 rounded-sm hover:bg-zinc-800/50 cursor-pointer group transition-colors">
    <FileText size={14} class="text-zinc-600 group-hover:text-zinc-300" />
    <span class="text-xs text-zinc-400 group-hover:text-zinc-200 truncate">
      {props.name}
    </span>
  </div>
);
