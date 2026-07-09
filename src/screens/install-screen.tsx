import { HammerIcon, PlayIcon } from "lucide-react";
import type { TFunction } from "i18next";
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
import type {
  InstallPlan,
  InstallProgress,
  SetupActionPreview,
} from "@/lib/types";

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
            {plan?.actions.map((action) => (
              <StatusRow
                detail={actionDetail(action, t)}
                key={action.id}
                label={actionLabel(action, t)}
                meta={t(actionMetaKey(action))}
                tone={actionTone(action, Boolean(installProgress))}
              />
            ))}
          </AppCardContent>
        </AppCard>
        <ProgressLog entries={log} />
      </div>
    </ScreenShell>
  );
}

function actionTone(action: SetupActionPreview, hasRun: boolean) {
  if (action.status === "not_implemented") {
    return "warning";
  }

  return hasRun ? "success" : "idle";
}

function actionMetaKey(action: SetupActionPreview) {
  if (action.status === "not_implemented" && action.intent === "verify") {
    return "install.actionStatus.notImplemented";
  }

  if (action.intent === "verify") {
    return "install.actionStatus.verify";
  }

  if (action.intent === "update") {
    return "install.actionStatus.update";
  }

  return "install.actionStatus.add";
}

function actionLabel(action: SetupActionPreview, t: TFunction) {
  return t(`install.actions.${action.kind}.label`, {
    subject: action.subject,
  });
}

function actionDetail(action: SetupActionPreview, t: TFunction) {
  return t(`install.actions.${action.kind}.detail`, {
    subject: action.subject,
    target: action.target ? t(`install.targets.${action.target}`) : undefined,
  });
}
