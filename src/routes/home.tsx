import { FileUp } from "lucide-solid";
import { createSignal } from "solid-js";
import { NotebooksGrid } from "../components/notebooks/grid";

export function HomeRoute() {
  const [isDragging, setIsDragging] = createSignal(false);

  const onDragOver = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const onDragLeave = () => setIsDragging(false);

  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      console.log("Files dropped:", files[0].name);
    }
  };

  return (
    <div class="min-h-screen bg-black text-white p-8 ">
      <div class="max-w-6xl mx-auto">
        {/* Unified Input Hub */}
        <section class="mb-16 space-y-4">
          {/* Drag & Drop Zone */}
          <div
            onDragOver={onDragOver}
            onDragLeave={onDragLeave}
            onDrop={onDrop}
            class={`
              relative h-48 rounded-2xl border border-dashed transition-all duration-300
              flex flex-col items-center justify-center gap-3 cursor-pointer
              ${
                isDragging()
                  ? "border-white bg-zinc-900/50 scale-[1.01]"
                  : "border-zinc-800 bg-zinc-900/10 hover:border-zinc-700"
              }
            `}
          >
            <div
              class={`p-3 rounded-full border transition-colors ${isDragging() ? "bg-white text-black border-white" : "border-zinc-700 text-zinc-500"}`}
            >
              <FileUp size={24} />
            </div>
            <div class="text-center">
              <p class="text-sm font-medium tracking-wide">
                {isDragging() ? "DROP TO IMPORT" : "DRAG CONTEXT HERE"}
              </p>
              <p class="text-xs text-zinc-500 mt-1 uppercase tracking-widest">
                Supports PDF, TXT, or Code Files
              </p>
            </div>
          </div>
        </section>

        <NotebooksGrid />
      </div>
    </div>
  );
}
