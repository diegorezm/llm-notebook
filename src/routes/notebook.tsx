import { useParams } from "@solidjs/router";
import { ResizableSidebar } from "../components/notebook/file-sidebar";
import { ChatMessage } from "../components/notebook/chat-message";
import { ChatInput } from "../components/notebook/chat-input";
import { createResource, For, Show } from "solid-js";
import { getChatHistory } from "../lib/commands"; // Ensure this path matches your bindings

export function NotebookRoute() {
  const params = useParams<{ id: string }>();

  // 1. Fetch chat history whenever the ID changes
  const [chatHistory, { refetch }] = createResource(
    () => params.id,
    async (id) => {
      const [err, data] = await getChatHistory(id);
      if (err) throw err; // This will trigger the .error state below
      return data;
    },
  );

  return (
    <div class="flex h-screen bg-black text-zinc-100 overflow-hidden">
      <ResizableSidebar />

      <main class="flex-1 flex flex-col relative">
        <header class="h-14 border-b border-zinc-800 flex items-center px-6 justify-between bg-black/50 backdrop-blur-md">
          <h1 class="text-sm font-medium text-zinc-200">
            Notebook{" "}
            <span class="text-zinc-600 ml-1">#{params.id.slice(0, 6)}</span>
          </h1>
          <div class="badge badge-outline border-zinc-800 text-[10px] text-zinc-500 rounded-sm">
            Ollama: Llama 3
          </div>
        </header>

        <div class="flex-1 overflow-y-auto p-6 space-y-8 max-w-4xl mx-auto w-full">
          {/* 2. Error Handling */}
          <Show when={chatHistory.error}>
            <div class="alert alert-error bg-red-950/20 border-red-900 text-red-200 rounded-sm text-xs">
              <span>Error loading history: {chatHistory.error.reason}</span>
            </div>
          </Show>

          {/* 3. Loading State (Skeletons) */}
          <Show when={chatHistory.loading}>
            <div class="space-y-6 animate-pulse">
              <div class="h-4 bg-zinc-900 rounded w-3/4"></div>
              <div class="h-4 bg-zinc-900 rounded w-1/2"></div>
            </div>
          </Show>

          {/* 4. Chat Messages List */}
          <Show when={chatHistory()}>
            <For each={chatHistory()}>
              {(msg) => (
                <ChatMessage
                  role={msg.role as "user" | "assistant"}
                  content={msg.message}
                />
              )}
            </For>

            {/* Default welcome if the notebook is brand new */}
            <Show when={chatHistory()!.length === 0}>
              <ChatMessage
                role="assistant"
                content="Hello! I've indexed your files. Ask anything about your documents."
              />
            </Show>
          </Show>
        </div>

        {/* 5. Input handles sending logic */}
        <ChatInput
          onSend={(msg) => {
            console.log("Sending:", msg);
            // We'll hook up the actual Rust sendMessage command here next!
          }}
        />
      </main>
    </div>
  );
}
