import { useNavigate } from "@solidjs/router";
import { Ghost } from "lucide-solid";

export function NotFoundRoute() {
  const navigate = useNavigate();

  return (
    <div class="min-h-screen bg-black text-white flex flex-col items-center justify-center p-8 gap-8">
      <div class="flex flex-col items-center gap-4">
        <div class="p-5 rounded-full border border-zinc-800 bg-zinc-900/30 text-zinc-500">
          <Ghost size={48} />
        </div>

        <div class="text-center space-y-2">
          <p class="text-xs text-zinc-600 uppercase tracking-[0.3em] font-medium">
            Error 404
          </p>
          <h1 class="text-4xl font-bold tracking-tight">Page not found</h1>
          <p class="text-sm text-zinc-500 max-w-xs leading-relaxed">
            The page you're looking for doesn't exist or has been moved.
          </p>
        </div>
      </div>

      <button
        onClick={() => navigate("/")}
        class="px-6 py-2.5 rounded-xl border border-zinc-700 bg-zinc-900/50 text-sm font-medium tracking-wide uppercase hover:border-zinc-500 hover:bg-zinc-800 transition-all duration-200"
      >
        Go home
      </button>
    </div>
  );
}
