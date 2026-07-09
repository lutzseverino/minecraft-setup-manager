import { Gamepad2Icon, HardDriveIcon, ShieldCheckIcon } from "lucide-react";
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
import type { ServerConfig } from "@/config/server-catalog";

type WelcomeScreenProps = Readonly<{
  onContinue: () => void;
  server: ServerConfig;
}>;

const setupHighlights = [
  {
    Icon: HardDriveIcon,
    labelKey: "welcome.highlights.folder.label",
    detailKey: "welcome.highlights.folder.detail",
  },
  {
    Icon: Gamepad2Icon,
    labelKey: "welcome.highlights.launcher.label",
    detailKey: "welcome.highlights.launcher.detail",
  },
  {
    Icon: ShieldCheckIcon,
    labelKey: "welcome.highlights.check.label",
    detailKey: "welcome.highlights.check.detail",
  },
];

export function WelcomeScreen({ onContinue, server }: WelcomeScreenProps) {
  const { t } = useTranslation();

  return (
    <ScreenShell
      actions={
        <AppButton onClick={onContinue} size="lg">
          {t("welcome.start")}
        </AppButton>
      }
      eyebrow={t("welcome.eyebrow")}
      lead={t("welcome.lead", { server: server.displayName })}
      title={t("welcome.title", { server: server.displayName })}
    >
      <div className="grid gap-3">
        <StatusRow
          detail={t("welcome.status.folder.detail", {
            server: server.displayName,
          })}
          label={t("welcome.status.folder.label")}
          tone="success"
        />
        <StatusRow
          detail={t("welcome.status.launcher.detail")}
          label={t("welcome.status.launcher.label")}
          tone="info"
        />
        <StatusRow
          detail={t("welcome.status.local.detail")}
          label={t("welcome.status.local.label")}
          tone="success"
        />
        <AppCard>
          <AppCardHeader>
            <AppCardTitle>{t("welcome.highlights.title")}</AppCardTitle>
          </AppCardHeader>
          <AppCardContent className="grid gap-3 sm:grid-cols-3">
            {setupHighlights.map(({ Icon, detailKey, labelKey }) => (
              <div className="mc-slot bg-[var(--slot)] p-3" key={labelKey}>
                <Icon className="size-4 text-primary" />
                <div className="mt-3 text-sm font-medium">{t(labelKey)}</div>
                <div className="mt-1 text-sm text-muted-foreground">
                  {t(detailKey, { server: server.displayName })}
                </div>
              </div>
            ))}
          </AppCardContent>
        </AppCard>
      </div>
    </ScreenShell>
  );
}
