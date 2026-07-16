import {
  CheckCircle2Icon,
  CircleDashedIcon,
  InfoIcon,
  RefreshCwIcon,
  XCircleIcon,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import { AppButton } from "@/components/app/app-button";
import {
  AppCard,
  AppCardContent,
  AppCardHeader,
  AppCardTitle,
} from "@/components/app/app-card";
import {
  AppToggleGroup,
  AppToggleGroupItem,
} from "@/components/app/app-toggle-group";
import { ScreenShell } from "@/components/app/screen-shell";
import { StatusRow } from "@/components/app/status-row";
import { launcherOptions } from "@/config/setup-options";
import type { LauncherDetection, LauncherKind } from "@/lib/types";
import { cn } from "@/lib/utils";

type LauncherScreenProps = Readonly<{
  detections: LauncherDetection[];
  error: string | null;
  launcher: LauncherKind;
  onContinue: () => void;
  onRefresh: () => void;
  onSelect: (launcher: LauncherKind) => void;
}>;

function detectionMeta(
  detection: LauncherDetection,
  t: (key: string) => string,
) {
  if (detection.status === "detected") {
    return {
      Icon: CheckCircle2Icon,
      text: t("launcher.status.found"),
      className: "text-success",
    };
  }

  if (detection.status === "not_found") {
    return {
      Icon: XCircleIcon,
      text: t("launcher.status.notFound"),
      className: "text-muted-foreground",
    };
  }

  if (detection.kind === "manual") {
    return {
      Icon: InfoIcon,
      text: t("launcher.status.notSupported"),
      className: "text-muted-foreground",
    };
  }

  return {
    Icon: CircleDashedIcon,
    text: t("launcher.status.scanning"),
    className: "text-muted-foreground",
  };
}

export function LauncherScreen({
  detections,
  error,
  launcher,
  onContinue,
  onRefresh,
  onSelect,
}: LauncherScreenProps) {
  const { t } = useTranslation();
  const selectedDetection = detections.find((item) => item.kind === launcher);
  const canContinue =
    selectedDetection?.setupSupported === true &&
    selectedDetection.status !== "not_found";

  return (
    <ScreenShell
      actions={
        <>
          <AppButton onClick={onRefresh} variant="outline">
            <RefreshCwIcon data-icon="inline-start" />
            {t("launcher.scanAgain")}
          </AppButton>
          <AppButton disabled={!canContinue} onClick={onContinue}>
            {t("common.continue")}
            <CheckCircle2Icon data-icon="inline-end" />
          </AppButton>
        </>
      }
      eyebrow={t("launcher.eyebrow")}
      lead={t("launcher.lead")}
      title={t("launcher.title")}
    >
      <div className="grid gap-4">
        {error ? (
          <StatusRow
            detail={error}
            label={t("launcher.errorLabel")}
            tone="error"
          />
        ) : null}
        <AppCard>
          <AppCardHeader>
            <AppCardTitle>{t("launcher.cardTitle")}</AppCardTitle>
          </AppCardHeader>
          <AppCardContent>
            <AppToggleGroup
              className="grid w-full gap-2.5"
              onValueChange={(value) => {
                if (value) {
                  onSelect(value as LauncherKind);
                }
              }}
              type="single"
              value={launcher}
            >
              {detections.map((detection) => {
                const option = launcherOptions[detection.kind];
                const Icon = option.Icon;
                const meta = detectionMeta(detection, t);
                const unavailable =
                  !detection.setupSupported || detection.status === "not_found";

                return (
                  <AppToggleGroupItem
                    className="min-h-0! items-center px-3.5 py-3"
                    disabled={unavailable}
                    key={detection.kind}
                    treatment="choice"
                    value={detection.kind}
                  >
                    <span className="flex w-full items-center gap-3.5">
                      <span className="mc-slot grid size-11 shrink-0 place-items-center bg-[var(--slot)]">
                        <Icon className="size-5 text-foreground" />
                      </span>
                      <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                        <span className="text-sm font-semibold">
                          {t(option.labelKey)}
                        </span>
                        <span
                          className="truncate text-sm font-normal text-muted-foreground"
                          data-slot="choice-copy"
                        >
                          {!detection.setupSupported
                            ? t("launcher.notSupported")
                            : unavailable
                              ? t("launcher.unavailable")
                              : t(option.detailKey)}
                        </span>
                      </span>
                      <span
                        className={cn(
                          "flex shrink-0 items-center gap-1.5 font-mono text-[0.7rem] tracking-wide whitespace-nowrap",
                          meta.className,
                        )}
                        data-slot="choice-meta"
                      >
                        <meta.Icon className="size-3.5" />
                        {meta.text}
                      </span>
                    </span>
                  </AppToggleGroupItem>
                );
              })}
            </AppToggleGroup>
          </AppCardContent>
        </AppCard>
      </div>
    </ScreenShell>
  );
}
