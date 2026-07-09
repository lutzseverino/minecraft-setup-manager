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
  isInstalling: boolean;
  isValidating: boolean;
  onContinue: () => void;
  onInstall: () => void;
  plan: InstallPlan | null;
}>;

export function InstallScreen({
  installProgress,
  isInstalling,
  isValidating,
  onContinue,
  onInstall,
  plan,
}: InstallScreenProps) {
  const { t } = useTranslation();
  const log = installProgress?.log ?? [t("install.emptyLog")];
  const unsupportedActions =
    plan?.actions.filter((action) => action.status === "not_implemented") ?? [];
  const installSucceeded = installProgress?.phase === "complete";

  return (
    <ScreenShell
      actions={
        <>
          <AppButton
            disabled={!plan || unsupportedActions.length > 0 || isInstalling}
            onClick={onInstall}
            variant="outline"
          >
            <PlayIcon data-icon="inline-start" />
            {isInstalling ? t("install.running") : t("install.run")}
          </AppButton>
          <AppButton
            disabled={!installSucceeded || isValidating}
            onClick={onContinue}
          >
            {isValidating ? t("install.checking") : t("install.check")}
          </AppButton>
        </>
      }
      eyebrow={t("install.eyebrow")}
      lead={t("install.lead")}
      title={t("install.title")}
    >
      <div className="grid gap-4">
        {unsupportedActions.length > 0 ? (
          <StatusRow
            detail={t("install.unsupported.detail")}
            label={t("install.unsupported.label")}
            tone="warning"
          />
        ) : null}
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
                detail={actionDetail(action, plan, t)}
                key={action.id}
                label={actionLabel(action, t)}
                meta={t(actionMetaKey(action))}
                tone={actionTone(action, installSucceeded)}
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

  if (action.intent === "remove") {
    return "install.actionStatus.remove";
  }

  return "install.actionStatus.add";
}

function actionLabel(action: SetupActionPreview, t: TFunction) {
  return t(`install.actions.${action.kind}.label`, {
    subject: action.subject,
  });
}

function actionDetail(
  action: SetupActionPreview,
  plan: InstallPlan,
  t: TFunction,
) {
  const detail = t(`install.actions.${action.kind}.detail`, {
    subject: action.subject,
    target: action.target ? t(`install.targets.${action.target}`) : undefined,
  });
  const resource = plan.resources.find((item) => item.id === action.resourceId);

  if (action.kind !== "sync_resource" || !resource) {
    return detail;
  }

  if (resource.source.kind === "modrinth") {
    return `${detail} ${t("install.resourceSource.modrinth", {
      project: resource.source.project,
      version: resource.source.version,
    })}`;
  }

  return `${detail} ${t("install.resourceSource.direct", {
    hash: hashLabel(resource.hashes),
    host: new URL(resource.source.url).host,
  })}`;
}

function hashLabel(hashes: InstallPlan["resources"][number]["hashes"]) {
  const hash = hashes.sha512 ?? hashes.sha256;
  const algorithm = hashes.sha512 ? "SHA-512" : "SHA-256";

  return hash ? `${algorithm} ${hash.slice(0, 12)}...` : algorithm;
}
