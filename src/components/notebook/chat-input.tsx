import { Component, createSignal } from "solid-js";
import { Send, Paperclip } from "lucide-solid";

export const ChatInput: Component<{
  onSend: (msg: string) => void;
  disabled: boolean;
}> = (props) => {
  const [text, setText] = createSignal("");

  const submit = () => {
    if (!text().trim()) return;
    props.onSend(text());
    setText("");
  };

  return (
    <div class="p-6 border-t border-zinc-900 bg-black">
      <div class="max-w-4xl mx-auto relative group">
        <textarea
          value={text()}
          onInput={(e) => setText(e.currentTarget.value)}
          onKeyDown={(e) =>
            e.key === "Enter" && !e.shiftKey && (e.preventDefault(), submit())
          }
          placeholder="Ask anything..."
          class="w-full bg-zinc-900/50 border border-zinc-800 rounded-sm p-4 pr-12 text-sm focus:outline-none focus:border-zinc-600 transition-colors resize-none h-24"
        />
        <div class="absolute bottom-4 right-4 flex items-center gap-2">
          <button class="p-1.5 text-zinc-600 hover:text-zinc-300 transition-colors">
            <Paperclip size={18} />
          </button>
          <button
            onClick={submit}
            class="p-1.5 bg-zinc-100 text-black rounded-sm hover:bg-white transition-colors"
            disabled={props.disabled}
          >
            <Send size={18} />
          </button>
        </div>
      </div>
    </div>
  );
};
