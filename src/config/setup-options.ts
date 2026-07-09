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
  ManifestPerformanceProfile,
  WizardStepId,
} from "@/lib/types";

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

export const fallbackDetections = [
  {
    kind: "official",
    status: "manual",
    setupSupported: true,
    detail: "Detection has not run yet.",
    confidence: 0,
  },
  {
    kind: "sklauncher",
    status: "manual",
    setupSupported: false,
    detail: "Detection has not run yet.",
    confidence: 0,
  },
  {
    kind: "manual",
    status: "manual",
    setupSupported: false,
    detail: "Manual setup is not available yet.",
    confidence: 1,
  },
] satisfies LauncherDetection[];

export function profileIcon(profile: ManifestPerformanceProfile) {
  if (profile.includesShaders) {
    return SparklesIcon;
  }

  if (profile.recommendedMemoryMb <= 3072) {
    return GaugeIcon;
  }

  return MonitorIcon;
}
