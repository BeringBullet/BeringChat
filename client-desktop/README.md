# BeringShare Desktop (minimal scaffold)

This folder contains a minimal Tauri + React scaffold for a desktop client that talks to the federated server.

Quick start (dev):

```bash
cd client-desktop
npm install
npm run dev
# In another shell: from repo root, run `cargo tauri dev` if you have Tauri installed
```

Notes:
- This is a scaffold only — adjust `API_BASE` in `src/services/api.ts` to point to your server.
- The Tauri backend lives under `src-tauri/`.
