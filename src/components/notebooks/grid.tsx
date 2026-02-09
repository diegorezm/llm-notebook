import { createResource, For, Show } from "solid-js";
import {
  createNotebook,
  deleteNotebook,
  getNotebooks,
} from "../../lib/commands";
import { ChevronRight, Plus, Trash2 } from "lucide-solid";
import { showToast } from "../../lib/toast";

export function NotebooksGrid() {
  const [notebooks, { refetch }] = createResource(async () => {
    const [err, notebooks] = await getNotebooks();
    if (err) throw new Error(err.reason);
    showToast({
      message: "testing",
    });
    return notebooks;
  });

  const formatDate = (ts: number) => {
    const d = new Date(ts * 1000);
    return d
      .toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: "2-digit",
      })
      .toUpperCase();
  };

  const handleCreate = async () => {
    const title = prompt("Enter notebook title:") || "Untitled Notebook";

    const [err, newNb] = await createNotebook(title);

    if (err) {
      // You could use a toast here later
      showToast({ message: `Failed to create: ${err.reason}`, type: "error" });
      return;
    }

    refetch();
    showToast({ message: `Notebook created`, type: "success" });
  };

  const handleDelete = async (id: string) => {
    const [err] = await deleteNotebook(id);

    if (err) {
      showToast({ message: `Failed to delete: ${err.reason}`, type: "error" });
      return;
    }

    refetch();
  };

  return (
    <>
      <header class="mb-8 flex justify-between items-end">
        <div>
          <h2 class="text-2xl font-light tracking-tighter">RECENT NOTEBOOKS</h2>
        </div>
        <div class="text-zinc-600 text-[10px] font-mono uppercase tracking-widest leading-none">
          {notebooks.length} Persistent Units
        </div>
      </header>
      <div class="p-8">
        {/* Grid */}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {/* Create New Card */}
          <button
            class="card border border-zinc-800 hover:border-zinc-400 bg-zinc-900/30 transition-all duration-300 group min-h-[180px]"
            onClick={handleCreate}
          >
            <div class="card-body items-center justify-center text-center">
              {/* Note: changed rounded-full to rounded-sm per your design constraints */}
              <div class="w-10 h-10 rounded-sm border border-zinc-700 flex items-center justify-center group-hover:bg-white group-hover:text-black transition-all">
                <Plus size={20} />
              </div>
              <p class="text-zinc-500 group-hover:text-zinc-200 mt-3 text-[10px] font-medium uppercase tracking-widest">
                Blank Notebook
              </p>
            </div>
          </button>

          <Show when={!notebooks.loading}>
            <For each={notebooks()}>
              {(nb) => (
                <div class="card border border-zinc-800 bg-black hover:bg-zinc-900/20 transition-all duration-300 group cursor-pointer">
                  <div class="card-body p-6">
                    <div class="flex justify-between items-start">
                      <h2 class="card-title text-zinc-100 font-normal text-base group-hover:text-white">
                        {nb.title}
                      </h2>
                      <span class="text-[9px] text-zinc-600 font-mono">
                        {formatDate(nb.created_at)}
                      </span>
                    </div>
                    <p class="text-zinc-500 text-xs line-clamp-2 mt-2 leading-relaxed italic">
                      {/* Fallback snippet since we aren't storing content in the metadata table yet */}
                      No recent activity or notes available in this workspace.
                    </p>
                    <div class="card-actions justify-end mt-4">
                      <button
                        class="btn btn-error btn-xs text-zinc-500  no-animation p-0 text-[10px] tracking-widest"
                        onClick={() => handleDelete(nb.id)}
                      >
                        DELETE <Trash2 class="size-3" />
                      </button>

                      <button class="btn btn-ghost btn-xs text-zinc-600 hover:text-white no-animation p-0 text-[10px] tracking-widest">
                        OPEN <ChevronRight class="size-3" />
                      </button>
                    </div>
                  </div>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
    </>
  );
}
