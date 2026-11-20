/** Simulated web bundle fragment for fixture REA live scans. */
import { createStore } from "./store.js";
import { apiClient } from "./api.js";

export function bootstrapApp(root) {
  const store = createStore();
  root.addEventListener("click", async (ev) => {
    if (ev.target?.dataset?.action === "load") {
      const data = await fetch("/api/items").then((r) => r.json());
      store.setItems(data);
    }
  });
  return { store, apiClient };
}

export default bootstrapApp;
