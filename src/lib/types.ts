export type LauncherKind = "official" | "sklauncher" | "manual";

export type PerformanceProfileId = "low_end" | "balanced" | "shaders";

export type WizardStepId =
  | "server"
  | "launcher"
  | "profile"
  | "install"
  | "validate"
  | "done";

export type InstallPhase =
  | "idle"
  | "planning"
  | "preparing"
  | "installing"
  | "validating"
  | "complete"
  | "failed";

export type LauncherDetectionStatus = "detected" | "not_found" | "manual";

export type LauncherDetection = Readonly<{
  kind: LauncherKind;
  status: LauncherDetectionStatus;
  detail: string;
  confidence: number;
}>;

export type PerformanceProfile = Readonly<{
  id: PerformanceProfileId;
  labelKey: string;
  detailKey: string;
  recommendedMemoryMb: number;
  includesShaders: boolean;
}>;

export type InstallPlanRequest = Readonly<{
  serverId: string;
  launcher: LauncherKind;
  profile: PerformanceProfileId;
  serverAddress: string;
}>;

export type ManifestLoaderKind = "none" | "fabric";

export type SetupManifest = Readonly<{
  schemaVersion: number;
  manifestVersion: string;
  id: string;
  displayName: string;
  server: {
    name: string;
    address: string;
  };
  minecraft: {
    version: string;
    loader: {
      kind: ManifestLoaderKind;
      version?: string | null;
    };
  };
  install: {
    gameDirectoryName: string;
    launcherProfileName: string;
  };
  profiles: Array<{
    id: string;
    label: string;
    recommendedMemoryMb: number;
    includesShaders?: boolean;
  }>;
  resources: Array<{
    id: string;
    name: string;
    resourceType: "mod" | "resource_pack" | "shader_pack" | "config";
    target: "mods" | "resourcepacks" | "shaderpacks" | "config";
    required: boolean;
    source: Record<string, unknown>;
    hashes?: Record<string, string>;
  }>;
  serverEntry?: {
    name: string;
    address: string;
  } | null;
}>;

export type SavedServerEntry = Readonly<{
  id: string;
  address: string;
  manifestUrl: string;
  displayName: string;
  lastCheckedAt: string;
  lastInstalledAt: string | null;
  selectedLauncher: LauncherKind;
  selectedProfile: PerformanceProfileId;
  installedManifestVersion: string | null;
  installedManifestFingerprint: string | null;
}>;

export type ServerUpdateStatus =
  | "new_setup"
  | "up_to_date"
  | "update_available";

export type ResolveServerManifestRequest = Readonly<{
  address: string;
}>;

export type ResolvedServerManifest = Readonly<{
  server: SavedServerEntry;
  manifest: SetupManifest;
  manifestFingerprint: string;
  updateStatus: ServerUpdateStatus;
}>;

export type InstallPlan = Readonly<{
  serverId: string;
  updateStatus: ServerUpdateStatus;
  minecraftVersion: string;
  fabricLoaderVersion: string;
  gameDirectoryName: string;
  serverName: string;
  serverAddress: string;
  launcher: LauncherKind;
  profile: PerformanceProfileId;
  actions: SetupActionPreview[];
  requiredMods: string[];
  optionalMods: string[];
  warnings: string[];
}>;

export type SetupActionKind =
  | "verify_loader"
  | "install_loader"
  | "ensure_game_directory"
  | "ensure_launcher_profile"
  | "sync_resource"
  | "remove_resource"
  | "write_server_entry"
  | "write_setup_receipt"
  | "validate_setup";

export type SetupActionIntent = "add" | "update" | "remove" | "verify";

export type SetupActionStatus = "ready" | "not_implemented";

export type SetupActionTarget =
  | "mods"
  | "resourcepacks"
  | "shaderpacks"
  | "config";

export type SetupActionPreview = Readonly<{
  id: string;
  kind: SetupActionKind;
  intent: SetupActionIntent;
  status: SetupActionStatus;
  required: boolean;
  resourceId: string | null;
  subject: string | null;
  target: SetupActionTarget | null;
}>;

export type StartInstallRequest = InstallPlanRequest;

export type InstallProgress = Readonly<{
  phase: InstallPhase;
  percent: number;
  log: string[];
  plan: InstallPlan;
}>;

export type ValidationStatus = "pass" | "warning" | "fail";

export type ValidationCheck = Readonly<{
  id: string;
  label: string;
  detail: string;
  status: ValidationStatus;
}>;

export type ValidationResult = Readonly<{
  overall: ValidationStatus;
  checks: ValidationCheck[];
}>;

export type DiagnosticBundle = Readonly<{
  path: string;
  summary: string;
}>;
