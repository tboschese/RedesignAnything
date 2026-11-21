import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The design system is linked from ../../design-system, which has no node_modules of its own:
// React is taken from this app.
export default defineConfig({
  plugins: [react()],
  resolve: { dedupe: ["react", "react-dom"], preserveSymlinks: true },
});
