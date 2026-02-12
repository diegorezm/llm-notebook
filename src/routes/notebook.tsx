import { useParams } from "@solidjs/router";
import { ResizableSidebar } from "../components/notebook/file-sidebar";
import { ChatMessage } from "../components/notebook/chat-message";
import { ChatInput } from "../components/notebook/chat-input";
import { createResource, createSignal, For, Show } from "solid-js";
import { chat, getChatHistory } from "../lib/commands"; // Ensure this path matches your bindings
import { showToast } from "../lib/toast";

export function NotebookRoute() {
  const params = useParams<{ id: string }>();
  const [isThinking, setIsThinking] = createSignal(false);
  const [optimisticMessages, setOptimisticMessages] = createSignal<
    { role: "user" | "assistant"; message: string }[]
  >([]);

  const [chatHistory, { refetch }] = createResource(
    () => params.id,
    async (id) => {
      const [err, data] = await getChatHistory(id);
      if (err) throw err;
      return data;
    },
  );

  // Combine real history + optimistic messages
  const allMessages = () => [...(chatHistory() ?? []), ...optimisticMessages()];

  async function handleChat(msg: string) {
    if (msg.length === 0 || isThinking()) return;

    // 1. Immediately show the user message
    setOptimisticMessages((prev) => [...prev, { role: "user", message: msg }]);
    setIsThinking(true);

    const [err, response] = await chat(params.id, msg);

    setIsThinking(false);
    setOptimisticMessages([]);

    if (err) {
      showToast({ message: err.reason, type: "error" });
      return;
    }

    refetch(); // Now fetch the real history with both messages persisted
  }

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
          <Show when={chatHistory.error}>
            <div class="alert alert-error bg-red-950/20 border-red-900 text-red-200 rounded-sm text-xs">
              <span>Error loading history: {chatHistory.error.reason}</span>
            </div>
          </Show>

          <Show when={chatHistory.loading}>
            <div class="space-y-6 animate-pulse">
              <div class="h-4 bg-zinc-900 rounded w-3/4"></div>
              <div class="h-4 bg-zinc-900 rounded w-1/2"></div>
            </div>
          </Show>

          <Show when={!chatHistory.loading}>
            {/* Default welcome if brand new */}
            <Show when={allMessages().length === 0}>
              <ChatMessage
                role="assistant"
                content="Hello! I've indexed your files. Ask anything about your documents."
              />
            </Show>

            <For each={allMessages()}>
              {(msg) => <ChatMessage role={msg.role} content={msg.message} />}
            </For>

            {/* Thinking indicator */}
            <Show when={isThinking()}>
              <div class="flex items-center gap-2 text-zinc-500">
                <div class="flex gap-1">
                  <span class="w-1.5 h-1.5 bg-zinc-500 rounded-full animate-bounce [animation-delay:0ms]" />
                  <span class="w-1.5 h-1.5 bg-zinc-500 rounded-full animate-bounce [animation-delay:150ms]" />
                  <span class="w-1.5 h-1.5 bg-zinc-500 rounded-full animate-bounce [animation-delay:300ms]" />
                </div>
                <span class="text-xs">Thinking...</span>
              </div>
            </Show>
          </Show>
        </div>

        <ChatInput disabled={isThinking()} onSend={handleChat} />
      </main>
    </div>
  );
}
