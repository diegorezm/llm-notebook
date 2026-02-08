import { ChevronRight, Plus, FileUp, MessageSquare } from "lucide-solid";
import { createSignal, For } from "solid-js";

export function HomeRoute() {
  const [isDragging, setIsDragging] = createSignal(false);
  const [message, setMessage] = createSignal("");
  const [notebooks, setNotebooks] = createSignal([
    {
      id: 1,
      title: "Next.js Architecture",
      date: "Feb 08",
      snippet: "Discussing server components...",
    },
    {
      id: 2,
      title: "Rust Backend Logic",
      date: "Feb 07",
      snippet: "Context regarding Mutex patterns...",
    },
  ]);

  // Handlers for the Drop Zone
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
      // Logic to create a new notebook with this file's content
    }
  };

  const handleSend = () => {
    if (!message().trim()) return;
    console.log("New session with message:", message());
    setMessage("");
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

          {/* Message Input */}
          <div class="relative group">
            <input
              type="text"
              value={message()}
              onInput={(e) => setMessage(e.currentTarget.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSend()}
              placeholder="Or start a conversation with a question..."
              class="w-full bg-zinc-900/50 border border-zinc-800 rounded-xl py-4 pl-12 pr-4 focus:outline-none focus:border-zinc-500 transition-colors placeholder:text-zinc-600 text-sm"
            />
            <MessageSquare
              class="absolute left-4 top-1/2 -translate-y-1/2 text-zinc-600 group-focus-within:text-zinc-400 transition-colors"
              size={18}
            />
            <div class="absolute right-3 top-1/2 -translate-y-1/2">
              <button
                onClick={handleSend}
                class="btn btn-ghost btn-xs text-zinc-500 hover:text-white"
              >
                Press ↵
              </button>
            </div>
          </div>
        </section>

        {/* Header */}
        <header class="mb-8 flex justify-between items-end">
          <div>
            <h2 class="text-2xl font-light tracking-tighter">
              RECENT NOTEBOOKS
            </h2>
          </div>
          <div class="text-zinc-600 text-[10px] font-mono uppercase tracking-widest leading-none">
            {notebooks().length} Persistent Units
          </div>
        </header>

        {/* Grid */}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {/* Create New Card (Simplified as requested) */}
          <button
            class="card border border-zinc-800 hover:border-zinc-400 bg-zinc-900/30 transition-all duration-300 group min-h-[180px]"
            onClick={() => console.log("New Session")}
          >
            <div class="card-body items-center justify-center text-center">
              <div class="w-10 h-10 rounded-full border border-zinc-700 flex items-center justify-center group-hover:bg-white group-hover:text-black transition-all">
                <Plus size={20} />
              </div>
              <p class="text-zinc-500 group-hover:text-zinc-200 mt-3 text-[10px] font-medium uppercase tracking-widest">
                Blank Notebook
              </p>
            </div>
          </button>

          <For each={notebooks()}>
            {(nb) => (
              <div class="card border border-zinc-800 bg-black hover:bg-zinc-900/20 transition-all duration-300 group cursor-pointer">
                <div class="card-body p-6">
                  <div class="flex justify-between items-start">
                    <h2 class="card-title text-zinc-100 font-normal text-base group-hover:text-white">
                      {nb.title}
                    </h2>
                    <span class="text-[9px] text-zinc-600 font-mono">
                      {nb.date}
                    </span>
                  </div>
                  <p class="text-zinc-500 text-xs line-clamp-2 mt-2 leading-relaxed italic">
                    {nb.snippet}
                  </p>
                  <div class="card-actions justify-end mt-4">
                    <button class="btn btn-ghost btn-xs text-zinc-600 hover:text-white no-animation p-0 text-[10px] tracking-widest">
                      OPEN <ChevronRight class="size-3" />
                    </button>
                  </div>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
