import { Component } from "solid-js";
import { MessageRole } from "../../lib/commands";

interface ChatMessageProps {
  role: MessageRole;
  content: string;
}

export const ChatMessage: Component<ChatMessageProps> = (props) => {
  return (
    <div class="flex gap-4">
      <div
        class={`w-6 h-6 rounded-sm flex items-center justify-center shrink-0 border ${
          props.role === "assistant"
            ? "bg-zinc-800 border-zinc-700"
            : "bg-zinc-100 border-white"
        }`}
      >
        <span
          class={`text-[10px] ${props.role === "assistant" ? "text-zinc-400" : "text-black"}`}
        >
          {props.role === "assistant" ? "AI" : "U"}
        </span>
      </div>
      <div class="space-y-2">
        <p class="text-sm leading-relaxed text-zinc-300">{props.content}</p>
      </div>
    </div>
  );
};
