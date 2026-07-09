export type LauncherKind = "official" | "sklauncher" | "manual";

export type PerformanceProfileId = "low_end" | "balanced" | "shaders";

export type WizardStepId =
  | "welcome"
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

export type InstallPlanStepId =
  | "game_directory"
  | "fabric_version"
  | "launcher_profile"
  | "mods_directory"
  | "setup_receipt"
  | "validation";

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

export type InstallPlan = Readonly<{
  serverId: string;
  minecraftVersion: string;
  fabricLoaderVersion: string;
  gameDirectoryName: string;
  serverName: string;
  serverAddress: string;
  launcher: LauncherKind;
  profile: PerformanceProfileId;
  steps: InstallPlanStepId[];
  requiredMods: string[];
  optionalMods: string[];
  warnings: string[];
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
