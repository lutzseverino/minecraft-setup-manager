import { HammerIcon, PlayIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { AppButton } from "@/components/app/app-button";
import {
  AppCard,
  AppCardContent,
  AppCardHeader,
  AppCardTitle,
} from "@/components/app/app-card";
import { ProgressLog } from "@/components/app/progress-log";
import { ScreenShell } from "@/components/app/screen-shell";
import { StatusRow } from "@/components/app/status-row";
import type { InstallPlan, InstallProgress } from "@/lib/types";

type InstallScreenProps = Readonly<{
  installProgress: InstallProgress | null;
  onContinue: () => void;
  onInstall: () => void;
  plan: InstallPlan | null;
}>;

export function InstallScreen({
  installProgress,
  onContinue,
  onInstall,
  plan,
}: InstallScreenProps) {
  const { t } = useTranslation();
  const log = installProgress?.log ?? [
    t("install.emptyLog"),
  ];

  return (
    <ScreenShell
      actions={
        <>
          <AppButton onClick={onInstall} variant="outline">
            <PlayIcon data-icon="inline-start" />
            {t("install.run")}
          </AppButton>
          <AppButton disabled={!installProgress} onClick={onContinue}>
            {t("install.check")}
          </AppButton>
        </>
      }
      eyebrow={t("install.eyebrow")}
      lead={t("install.lead")}
      title={t("install.title")}
    >
      <div className="grid gap-4">
        <AppCard>
          <AppCardHeader>
            <AppCardTitle className="flex items-center gap-2 text-sm">
              <HammerIcon className="size-4" />
              {t("install.cardTitle")}
            </AppCardTitle>
          </AppCardHeader>
          <AppCardContent className="grid gap-2">
            {plan?.steps.map((step) => (
              <StatusRow
                detail={
                  step === "game_directory"
                    ? t("install.steps.game_directory.detail")
                    : undefined
                }
                key={step}
                label={t(`install.steps.${step}.label`)}
                tone={installProgress ? "success" : "idle"}
              />
            ))}
          </AppCardContent>
        </AppCard>
        <ProgressLog entries={log} />
      </div>
    </ScreenShell>
  );
}
