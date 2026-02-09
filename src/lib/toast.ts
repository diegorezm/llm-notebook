import { createStore } from "solid-js/store";

export type ToastType = "success" | "error" | "info" | "warning";

interface Toast {
  id: number;
  message: string;
  type: ToastType;
}

const [toasts, setToasts] = createStore<Toast[]>([]);

export { toasts };

type ShowToastProps = {
  message: string;
  type?: ToastType;
  duration?: number;
};

export const showToast = ({
  message,
  type = "info",
  duration = 3000,
}: ShowToastProps) => {
  const id = Date.now();
  setToasts((prev) => [...prev, { id, message, type }]);

  setTimeout(() => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, duration);
};
