import {
  BlocksIcon,
  CompassIcon,
  FileQuestionIcon,
  GaugeIcon,
  MonitorIcon,
  SparklesIcon,
} from "lucide-react";

import type {
  LauncherDetection,
  LauncherKind,
  PerformanceProfileId,
  WizardStepId,
} from "@/lib/types";

export type PerformanceProfileOption = Readonly<{
  id: PerformanceProfileId;
  Icon: typeof GaugeIcon;
  labelKey: string;
  detailKey: string;
  recommendedMemoryMb: number;
  includesShaders: boolean;
}>;

export const wizardSteps = [
  { id: "server", labelKey: "steps.server" },
  { id: "launcher", labelKey: "steps.launcher" },
  { id: "profile", labelKey: "steps.profile" },
  { id: "install", labelKey: "steps.install" },
  { id: "validate", labelKey: "steps.validate" },
  { id: "done", labelKey: "steps.done" },
] satisfies Array<{ id: WizardStepId; labelKey: string }>;

export const launcherOptions = {
  official: {
    Icon: BlocksIcon,
    labelKey: "launchers.official.label",
    detailKey: "launchers.official.detail",
  },
  sklauncher: {
    Icon: CompassIcon,
    labelKey: "launchers.sklauncher.label",
    detailKey: "launchers.sklauncher.detail",
  },
  manual: {
    Icon: FileQuestionIcon,
    labelKey: "launchers.manual.label",
    detailKey: "launchers.manual.detail",
  },
} satisfies Record<
  LauncherKind,
  { Icon: typeof BlocksIcon; labelKey: string; detailKey: string }
>;

export const performanceProfiles = [
  {
    id: "low_end",
    Icon: GaugeIcon,
    labelKey: "profiles.lowEnd.label",
    detailKey: "profiles.lowEnd.detail",
    recommendedMemoryMb: 3072,
    includesShaders: false,
  },
  {
    id: "balanced",
    Icon: MonitorIcon,
    labelKey: "profiles.balanced.label",
    detailKey: "profiles.balanced.detail",
    recommendedMemoryMb: 4096,
    includesShaders: false,
  },
  {
    id: "shaders",
    Icon: SparklesIcon,
    labelKey: "profiles.shaders.label",
    detailKey: "profiles.shaders.detail",
    recommendedMemoryMb: 6144,
    includesShaders: true,
  },
] satisfies PerformanceProfileOption[];

export const fallbackDetections = [
  {
    kind: "official",
    status: "manual",
    detail: "Detection has not run yet.",
    confidence: 0,
  },
  {
    kind: "sklauncher",
    status: "manual",
    detail: "Detection has not run yet.",
    confidence: 0,
  },
  {
    kind: "manual",
    status: "manual",
    detail: "Manual setup is always available.",
    confidence: 1,
  },
] satisfies LauncherDetection[];

export function getOptionalModNames(
  profile: PerformanceProfileId,
  balancedExtras: string[],
  shadersExtras: string[],
) {
  if (profile === "low_end") {
    return ["Dynamic FPS", "Entity Culling", "FerriteCore"];
  }

  if (profile === "shaders") {
    return [...balancedExtras, ...shadersExtras];
  }

  return balancedExtras;
}
