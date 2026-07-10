import type { TFunction } from "i18next";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { AppButton } from "@/components/app/app-button";
import {
  AppCard,
  AppCardAction,
  AppCardContent,
  AppCardFooter,
  AppCardHeader,
  AppCardTitle,
} from "@/components/app/app-card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { AppUpdaterController } from "@/hooks/use-app-updater";

type AppUpdaterProps = Readonly<{
  controller: AppUpdaterController;
  updateBlocked: boolean;
}>;

export function AppUpdater({ controller, updateBlocked }: AppUpdaterProps) {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(true);
  const { state } = controller;

  if (state.status === "unsupported") {
    return null;
  }

  if (
    state.status === "checking" ||
    state.status === "current" ||
    ((state.status === "available" || state.status === "error") && !isExpanded)
  ) {
    return (
      <section
        aria-label={t("applicationUpdate.title")}
        aria-live="polite"
        className="flex flex-wrap items-center justify-end gap-3"
      >
        <p className="text-sm text-muted-foreground">
          {state.status === "checking"
            ? t("applicationUpdate.checking")
            : state.status === "current"
              ? t("applicationUpdate.current")
              : state.status === "available"
                ? t("applicationUpdate.compactAvailable", {
                    version: state.update?.version,
                  })
                : t("applicationUpdate.compactError")}
        </p>
        <AppButton
          disabled={state.status === "checking"}
          onClick={() => {
            if (state.status === "available" || state.status === "error") {
              setIsExpanded(true);
            } else {
              void controller.checkForUpdate();
            }
          }}
          variant="ghost"
        >
          {state.status === "available" || state.status === "error"
            ? t("applicationUpdate.review")
            : t("applicationUpdate.checkAgain")}
        </AppButton>
      </section>
    );
  }

  const isAvailable = state.status === "available";
  const isReadyToRestart = state.status === "ready_to_restart";
  const isError = state.status === "error";
  const version = state.update?.version;

  return (
    <section aria-label={t("applicationUpdate.title")}>
      <AppCard>
        <AppCardHeader>
          <AppCardTitle>{
            isError
              ? t("applicationUpdate.errorTitle")
              : isReadyToRestart
                ? t("applicationUpdate.readyTitle")
                : isAvailable
                  ? t("applicationUpdate.availableTitle")
                  : t("applicationUpdate.updatingTitle")
          }</AppCardTitle>
          {version ? (
            <AppCardAction>
              <Badge variant="warning">v{version}</Badge>
            </AppCardAction>
          ) : null}
        </AppCardHeader>
        <AppCardContent className="flex flex-col gap-3">
          <p
            aria-atomic="true"
            aria-live="polite"
            className="text-sm leading-6 text-muted-foreground"
            role="status"
          >
            {updateDetail(state.status, t)}
          </p>

          {state.status === "downloading" ? (
            <div className="flex flex-col gap-2">
              <Progress
                aria-label={t("applicationUpdate.downloadProgress")}
                value={state.progress?.percent ?? null}
              />
              <p className="font-mono text-xs text-muted-foreground">
                {downloadDetail(state.progress, t)}
              </p>
            </div>
          ) : null}

          {isAvailable ? (
            <>
              <p className="text-sm leading-6">
                {t("applicationUpdate.consent", {
                  currentVersion: state.update?.currentVersion,
                  version,
                })}
              </p>
              <p className="text-sm leading-6 text-muted-foreground">
                {t("applicationUpdate.signatureTrust")}
              </p>
              {updateBlocked ? (
                <p className="text-sm leading-6 text-warning">
                  {t("applicationUpdate.setupBusy")}
                </p>
              ) : null}
              {state.update?.notes ? (
                <details className="text-sm text-muted-foreground">
                  <summary className="cursor-pointer font-medium text-foreground">
                    {t("applicationUpdate.releaseNotes")}
                  </summary>
                  <p className="mt-2 max-h-40 overflow-y-auto whitespace-pre-line">
                    {state.update.notes}
                  </p>
                </details>
              ) : null}
            </>
          ) : null}

          {state.error ? (
            <p className="text-sm leading-6 text-destructive">{state.error}</p>
          ) : null}
        </AppCardContent>
        {isAvailable || isError || isReadyToRestart ? (
          <AppCardFooter className="flex flex-wrap justify-end gap-2 p-4">
            {isAvailable ? (
              <>
                <AppButton onClick={() => setIsExpanded(false)} variant="ghost">
                  {t("applicationUpdate.later")}
                </AppButton>
                <AppButton
                  disabled={updateBlocked}
                  onClick={() => void controller.installAndRestart()}
                >
                  {t("applicationUpdate.updateAndRestart")}
                </AppButton>
              </>
            ) : null}
            {isError ? (
              <>
                <AppButton onClick={() => setIsExpanded(false)} variant="ghost">
                  {t("applicationUpdate.dismiss")}
                </AppButton>
                <AppButton
                  disabled={updateBlocked && state.failure !== "check"}
                  onClick={() => void controller.retry()}
                >
                  {t("applicationUpdate.tryAgain")}
                </AppButton>
              </>
            ) : null}
            {isReadyToRestart ? (
              <AppButton
                disabled={updateBlocked}
                onClick={() => void controller.restart()}
              >
                {t("applicationUpdate.restartNow")}
              </AppButton>
            ) : null}
          </AppCardFooter>
        ) : null}
      </AppCard>
    </section>
  );
}

function updateDetail(
  status: AppUpdaterController["state"]["status"],
  t: TFunction,
) {
  if (status === "downloading") {
    return t("applicationUpdate.downloading");
  }
  if (status === "installing") {
    return t("applicationUpdate.installing");
  }
  if (status === "restarting") {
    return t("applicationUpdate.restarting");
  }
  if (status === "ready_to_restart") {
    return t("applicationUpdate.readyDetail");
  }
  if (status === "error") {
    return t("applicationUpdate.errorDetail");
  }

  return t("applicationUpdate.availableDetail");
}

function downloadDetail(
  progress: AppUpdaterController["state"]["progress"],
  t: TFunction,
) {
  if (!progress) {
    return t("applicationUpdate.preparingDownload");
  }

  if (progress.totalBytes === null) {
    return t("applicationUpdate.downloaded", {
      downloaded: formatBytes(progress.downloadedBytes),
    });
  }

  return t("applicationUpdate.downloadedOf", {
    downloaded: formatBytes(progress.downloadedBytes),
    total: formatBytes(progress.totalBytes),
  });
}

function formatBytes(bytes: number) {
  if (bytes < 1024 * 1024) {
    return `${Math.max(0, Math.round(bytes / 1024))} KiB`;
  }

  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}
