import { Component, For } from "solid-js";
import { toasts } from "../../lib/toast";
import { BugIcon, Check, InfoIcon, TriangleAlert } from "lucide-solid";

const ToastContainer: Component = () => {
  return (
    <div class="toast toast-end toast-bottom z-9999 p-4">
      <For each={toasts}>
        {(toast) => (
          <div
            class={`alert shadow-2xl border border-zinc-800 rounded-sm text-[11px] font-medium tracking-wide min-w-70 transition-all duration-300 ${
              toast.type === "error"
                ? "alert-error bg-red-950/40 text-red-200"
                : toast.type === "success"
                  ? "alert-success bg-zinc-900 text-emerald-400"
                  : "bg-zinc-900 text-zinc-100"
            }`}
          >
            <div class="flex items-center gap-3">
              <div class="shrink-0">
                {toast.type === "info" && <InfoIcon />}
                {toast.type === "warning" && <TriangleAlert />}
                {toast.type === "error" && <BugIcon />}
                {toast.type === "success" && <Check />}
              </div>
              <span>{toast.message}</span>
            </div>
          </div>
        )}
      </For>
    </div>
  );
};

export default ToastContainer;
