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
import type { PerformanceProfileOption } from "@/config/setup-options";
import type { PerformanceProfileId, ResolvedServerManifest } from "@/lib/types";

type ProfileScreenProps = Readonly<{
  onContinue: () => void;
  onProfileChange: (profile: PerformanceProfileId) => void;
  profile: PerformanceProfileId;
  profiles: PerformanceProfileOption[];
  server: ResolvedServerManifest | null;
}>;

export function ProfileScreen({
  onContinue,
  onProfileChange,
  profile,
  profiles,
  server,
}: ProfileScreenProps) {
  const { t } = useTranslation();
  const serverName = server?.manifest.displayName ?? t("server.unknownName");

  return (
    <ScreenShell
      actions={<AppButton onClick={onContinue}>{t("profile.continue")}</AppButton>}
      eyebrow={t("profile.eyebrow")}
      lead={t("profile.lead", { server: serverName })}
      title={t("profile.title")}
    >
      <div className="grid gap-4">
        <AppToggleGroup
          className="grid w-full auto-rows-fr items-stretch gap-3 sm:grid-cols-3"
          onValueChange={(value) => {
            if (value) {
              onProfileChange(value as PerformanceProfileId);
            }
          }}
          type="single"
          value={profile}
        >
          {profiles.map((item) => {
            const Icon = item.Icon;

            return (
              <AppToggleGroupItem
                className="h-full! min-h-40 items-stretch!"
                key={item.id}
                treatment="choice"
                value={item.id}
              >
                <span className="flex h-full w-full flex-col items-start gap-3">
                  <Icon className="size-5" />
                  <span className="text-sm font-semibold">
                    {t(item.labelKey)}
                  </span>
                  <span
                    className="text-sm font-normal text-muted-foreground"
                    data-slot="choice-copy"
                  >
                    {t(item.detailKey)}
                  </span>
                  <span
                    className="mt-auto pt-1 font-mono text-[0.68rem] leading-none tracking-[0.16em] text-muted-foreground uppercase"
                    data-slot="choice-meta"
                  >
                    {t("profile.memory", { memory: item.recommendedMemoryMb })}
                  </span>
                </span>
              </AppToggleGroupItem>
            );
          })}
        </AppToggleGroup>
        <AppCard>
          <AppCardHeader>
            <AppCardTitle className="text-sm">
              {t("profile.serverCardTitle")}
            </AppCardTitle>
          </AppCardHeader>
          <AppCardContent className="grid gap-1 text-sm">
            <div className="font-medium">{serverName}</div>
            <div className="text-muted-foreground">
              {server?.manifest.server.address}
            </div>
            <div className="font-mono text-xs text-muted-foreground">
              {t("profile.manifestVersion", {
                version: server?.manifest.manifestVersion ?? "",
              })}
            </div>
          </AppCardContent>
        </AppCard>
      </div>
    </ScreenShell>
  );
}
