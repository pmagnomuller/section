import { defineConfig } from "vite";

export default defineConfig({
  base: process.env.SECTION_BASE ?? "/",
});
