import { FileDownIcon, RotateCcwIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { AppButton } from "@/components/app/app-button";
import {
  AppCard,
  AppCardContent,
  AppCardHeader,
  AppCardTitle,
} from "@/components/app/app-card";
import { ScreenShell } from "@/components/app/screen-shell";
import { StatusRow } from "@/components/app/status-row";
import {
  launcherOptions,
  performanceProfiles,
} from "@/config/setup-options";
import type { DiagnosticBundle, InstallPlan } from "@/lib/types";

type DoneScreenProps = Readonly<{
  diagnostics: DiagnosticBundle | null;
  onExportDiagnostics: () => void;
  onRestart: () => void;
  plan: InstallPlan | null;
}>;

export function DoneScreen({
  diagnostics,
  onExportDiagnostics,
  onRestart,
  plan,
}: DoneScreenProps) {
  const { t } = useTranslation();
  const selectedProfile = performanceProfiles.find(
    (item) => item.id === (plan?.profile ?? "balanced"),
  );
  const selectedLauncher = launcherOptions[plan?.launcher ?? "official"];

  return (
    <ScreenShell
      actions={
        <>
          <AppButton onClick={onExportDiagnostics} variant="outline">
            <FileDownIcon data-icon="inline-start" />
            {t("done.export")}
          </AppButton>
          <AppButton onClick={onRestart} variant="secondary">
            <RotateCcwIcon data-icon="inline-start" />
            {t("done.startOver")}
          </AppButton>
        </>
      }
      eyebrow={t("done.eyebrow")}
      lead={t("done.lead")}
      title={t("done.title")}
    >
      <div className="grid gap-4">
        <StatusRow
          detail={t("done.summary.detail", {
            profile: selectedProfile ? t(selectedProfile.labelKey) : "",
            launcher: t(selectedLauncher.labelKey),
          })}
          label={t("done.summary.label")}
          tone="success"
        />
        {diagnostics ? (
          <AppCard>
            <AppCardHeader>
              <AppCardTitle className="flex items-center gap-2 text-sm">
                <FileDownIcon className="size-4" />
                {t("done.reportTitle")}
              </AppCardTitle>
            </AppCardHeader>
            <AppCardContent className="grid gap-2 text-sm text-muted-foreground">
              <p className="mc-inset bg-[var(--slot)] p-3">
                {diagnostics.summary}
                <br />
                <span className="font-mono text-xs">{diagnostics.path}</span>
              </p>
            </AppCardContent>
          </AppCard>
        ) : null}
      </div>
    </ScreenShell>
  );
}
