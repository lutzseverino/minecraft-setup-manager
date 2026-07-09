import { invoke } from "@tauri-apps/api/core";

import { getServerConfig } from "@/config/server-catalog";
import { getOptionalModNames } from "@/config/setup-options";
import type {
  DiagnosticBundle,
  InstallPlan,
  InstallPlanRequest,
  InstallProgress,
  LauncherDetection,
  StartInstallRequest,
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

export async function getInstallPlan(request: InstallPlanRequest) {
  const server = getServerConfig(request.serverId);
  const optionalMods = getOptionalModNames(
    request.profile,
    server.balancedExtras,
    server.shadersExtras,
  );

  return invokeOrFallback<InstallPlan>(
    "get_install_plan",
    { request },
    {
      serverId: server.id,
      minecraftVersion: server.minecraftVersion,
      fabricLoaderVersion: server.fabricLoaderVersion,
      gameDirectoryName: server.gameDirectoryName,
      serverName: server.displayName,
      serverAddress: request.serverAddress || server.defaultAddress,
      launcher: request.launcher,
      profile: request.profile,
      steps: [
        "fabric_version",
        "game_directory",
        "launcher_profile",
        "mods_directory",
        "setup_receipt",
        "validation",
      ],
      requiredMods: server.requiredMods,
      optionalMods,
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
      path: "~/Desktop/maresme-mc-check-report.json",
      summary: "Desktop app saves this report on your Desktop.",
    },
  );
}
