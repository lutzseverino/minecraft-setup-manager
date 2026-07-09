import { invoke } from "@tauri-apps/api/core";

import type {
  DiagnosticBundle,
  InstallPlan,
  InstallPlanRequest,
  InstallProgress,
  LauncherDetection,
  ResolvedServerManifest,
  ResolveServerManifestRequest,
  SavedServerEntry,
  StartInstallRequest,
  SetupManifest,
  ValidationResult,
} from "@/lib/types";

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invokeOrFallback<T>(
  command: string,
  args: Record<string, unknown>,
  fallback: T,
) {
  if (!isTauriRuntime()) {
    return fallback;
  }

  return invoke<T>(command, args);
}

const demoManifest = {
  schemaVersion: 1,
  manifestVersion: "demo.1",
  id: "example-server",
  displayName: "Example Server",
  server: {
    name: "Example Server",
    address: "play.example.com",
  },
  minecraft: {
    version: "1.21.6",
    loader: {
      kind: "fabric",
      version: "0.16.14",
    },
  },
  install: {
    gameDirectoryName: "Example Server",
    launcherProfileName: "Example Server",
  },
  profiles: [
    {
      id: "balanced",
      label: "Recommended",
      recommendedMemoryMb: 4096,
    },
  ],
  resources: [
    {
      id: "fabric-api",
      name: "Fabric API",
      resourceType: "mod",
      target: "mods",
      required: true,
      fileName: "fabric-api.jar",
      source: { kind: "modrinth", project: "fabric-api", version: "demo" },
      hashes: {},
    },
  ],
  serverEntry: {
    name: "Example Server",
    address: "play.example.com",
  },
} satisfies SetupManifest;

export async function detectLaunchers() {
  const fallback: LauncherDetection[] = [
    {
      kind: "official",
      status: "detected",
      detail: "Likely installed in the usual place.",
      confidence: 0.74,
    },
    {
      kind: "sklauncher",
      status: "not_found",
      detail: "Not found on this computer.",
      confidence: 0.22,
    },
    {
      kind: "manual",
      status: "manual",
      detail: "Use this if your launcher is not listed.",
      confidence: 1,
    },
  ];

  return invokeOrFallback<LauncherDetection[]>("detect_launchers", {}, fallback);
}

export async function listSavedServers() {
  return invokeOrFallback<SavedServerEntry[]>("list_saved_servers", {}, []);
}

export async function resolveServerManifest(
  request: ResolveServerManifestRequest,
) {
  const address = request.address.trim() || demoManifest.server.address;
  const fallbackServer = {
    id: `${demoManifest.id}@${address}`,
    address,
    manifestUrl: `https://${address}/.well-known/minecraft-setup-manager/manifest.json`,
    displayName: demoManifest.displayName,
    lastCheckedAt: new Date().toISOString(),
    lastInstalledAt: null,
    selectedLauncher: "official",
    selectedProfile: "balanced",
    installedManifestVersion: null,
    installedManifestFingerprint: null,
  } satisfies SavedServerEntry;

  return invokeOrFallback<ResolvedServerManifest>(
    "resolve_server_manifest",
    { request },
    {
      server: fallbackServer,
      manifest: {
        ...demoManifest,
        server: { ...demoManifest.server, address },
      },
      manifestFingerprint: "sha256:browser-demo",
      updateStatus: "new_setup",
    },
  );
}

export async function getInstallPlan(request: InstallPlanRequest) {
  return invokeOrFallback<InstallPlan>(
    "get_install_plan",
    { request },
    {
      serverId: request.serverId || "example-server",
      updateStatus: "new_setup",
      minecraftVersion: demoManifest.minecraft.version,
      fabricLoaderVersion: demoManifest.minecraft.loader.version ?? "",
      gameDirectoryName: demoManifest.install.gameDirectoryName,
      serverName: demoManifest.displayName,
      serverAddress: request.serverAddress || demoManifest.server.address,
      launcher: request.launcher,
      profile: request.profile,
      actions: [
        {
          id: "fabric_version",
          kind: "verify_loader",
          intent: "verify",
          status: "ready",
          required: true,
          resourceId: null,
          subject: demoManifest.minecraft.loader.version ?? null,
          target: null,
          fileName: null,
        },
        {
          id: "fabric_install",
          kind: "install_loader",
          intent: "add",
          status: "not_implemented",
          required: true,
          resourceId: null,
          subject: demoManifest.minecraft.version,
          target: null,
          fileName: null,
        },
        {
          id: "game_directory",
          kind: "ensure_game_directory",
          intent: "add",
          status: "ready",
          required: true,
          resourceId: null,
          subject: demoManifest.install.gameDirectoryName,
          target: null,
          fileName: null,
        },
        {
          id: "launcher_profile",
          kind: "ensure_launcher_profile",
          intent: "update",
          status: "ready",
          required: true,
          resourceId: null,
          subject: demoManifest.install.launcherProfileName,
          target: null,
          fileName: null,
        },
        ...demoManifest.resources.map((resource) => ({
          id: `resource_${resource.id}`,
          kind: "sync_resource" as const,
          intent: "update" as const,
          status: "not_implemented" as const,
          required: resource.required,
          resourceId: resource.id,
          subject: resource.name,
          target: resource.target,
          fileName: resource.fileName ?? null,
        })),
        {
          id: "setup_receipt",
          kind: "write_setup_receipt",
          intent: "update",
          status: "ready",
          required: true,
          resourceId: null,
          subject: null,
          target: null,
          fileName: null,
        },
        {
          id: "validation",
          kind: "validate_setup",
          intent: "verify",
          status: "ready",
          required: true,
          resourceId: null,
          subject: null,
          target: null,
          fileName: null,
        },
      ],
      requiredMods: demoManifest.resources
        .filter((resource) => resource.required)
        .map((resource) => resource.name),
      optionalMods: [],
      warnings: [
        "Open the desktop app to create folders on this computer.",
      ],
    },
  );
}

export async function startInstall(request: StartInstallRequest) {
  const plan = await getInstallPlan(request);

  return invokeOrFallback<InstallProgress>(
    "start_install",
    { request },
    {
      phase: "complete",
      percent: 100,
      plan,
      log: [
        "[plan] Read the server setup list",
        "[launcher] Checked the launcher choice",
        "[fabric] Desktop app verifies the Fabric version",
        "[folder] Desktop app creates the separate game folder",
        "[profile] Desktop app creates the launcher profile",
        "[check] Desktop app checks the setup files",
      ],
    },
  );
}

export async function validateInstallation(request: InstallPlanRequest) {
  return invokeOrFallback<ValidationResult>(
    "validate_installation",
    { request },
    {
      overall: "pass",
      checks: [
        {
          id: "manifest",
          label: "Setup list loaded",
          detail: "Minecraft, Fabric, server, and mod choices are ready.",
          status: "pass",
        },
        {
          id: "game_directory",
          label: "Separate game folder",
          detail: "The game folder is ready.",
          status: "pass",
        },
        {
          id: "fabric_version",
          label: "Fabric version",
          detail: "Fabric version is installed.",
          status: "pass",
        },
        {
          id: "launcher_profile",
          label: "Launcher profile",
          detail: "Launcher profile is ready.",
          status: "pass",
        },
        {
          id: "mods_directory",
          label: "Mods folder",
          detail: "The mods folder is ready.",
          status: "pass",
        },
        {
          id: "setup_receipt",
          label: "Setup file",
          detail: "The setup file is saved.",
          status: "pass",
        },
      ],
    },
  );
}

export async function exportDiagnostics() {
  return invokeOrFallback<DiagnosticBundle>(
    "export_diagnostics",
    {},
    {
      path: "~/Desktop/minecraft-setup-manager-report.json",
      summary: "Desktop app saves this report on your Desktop.",
    },
  );
}
