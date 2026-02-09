import { useParams } from "@solidjs/router";
import { ResizableSidebar } from "../components/notebook/file-sidebar";
import { ChatMessage } from "../components/notebook/chat-message";
import { ChatInput } from "../components/notebook/chat-input";

export function NotebookRoute() {
  const params = useParams<{ id: string }>();
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
          <ChatMessage
            role="assistant"
            content="Hello! I've indexed your files."
          />
        </div>

        <ChatInput onSend={(msg) => console.log("Sending:", msg)} />
      </main>
    </div>
  );
}
