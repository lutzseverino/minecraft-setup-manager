export type LauncherKind = "official" | "sklauncher" | "manual";

export type PerformanceProfileId = string;

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
  setupSupported: boolean;
  detail: string;
  confidence: number;
}>;

export type ManifestPerformanceProfile = Readonly<{
  id: string;
  label: string;
  recommendedMemoryMb: number;
  includesShaders?: boolean;
}>;

export type InstallPlanRequest = Readonly<{
  serverId: string;
  manifestFingerprint: string;
  launcher: LauncherKind;
  profile: PerformanceProfileId;
}>;

export type ManifestLoaderKind = "none" | "fabric";

export type ManifestResourceSource =
  | Readonly<{
      kind: "direct";
      url: string;
    }>
  | Readonly<{
      kind: "modrinth";
      project: string;
      version: string;
    }>;

export type ManifestResourceHashes = Readonly<{
  sha256?: string;
  sha512?: string;
}>;

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
      version?: string;
    };
  };
  install: {
    gameDirectoryName: string;
    launcherProfileName: string;
  };
  profiles: ManifestPerformanceProfile[];
  resources: Array<{
    id: string;
    name: string;
    resourceType: "mod" | "resource_pack" | "shader_pack" | "config";
    target: "mods" | "resourcepacks" | "shaderpacks" | "config";
    required: boolean;
    profiles?: string[];
    fileName: string;
    source: ManifestResourceSource;
    hashes?: ManifestResourceHashes;
  }>;
  serverEntry?: {
    name: string;
    address: string;
  };
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
  needsRepair: boolean;
}>;

export type ServerUpdateStatus =
  | "new_setup"
  | "up_to_date"
  | "update_available";

export type ResolveServerManifestRequest = Readonly<{
  address: string;
}>;

export type RedeemSetupAttestationRequest = InstallPlanRequest &
  Readonly<{
    challenge: string;
  }>;

export type SetupAttestationReceipt = Readonly<{
  manifestFingerprint: string;
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
  loaderKind: ManifestLoaderKind;
  loaderVersion: string | null;
  gameDirectoryName: string;
  serverName: string;
  serverAddress: string;
  launcherProfileName: string;
  launcher: LauncherKind;
  profile: PerformanceProfileId;
  profileLabel: string;
  recommendedMemoryMb: number;
  actions: SetupActionPreview[];
  resources: Array<{
    id: string;
    source: ManifestResourceSource;
    hashes: ManifestResourceHashes;
  }>;
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
  fileName: string | null;
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
