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
import { profileIcon } from "@/config/setup-options";
import type {
  ManifestPerformanceProfile,
  PerformanceProfileId,
  ResolvedServerManifest,
} from "@/lib/types";

type ProfileScreenProps = Readonly<{
  error: string | null;
  isBuilding: boolean;
  onContinue: () => void;
  onProfileChange: (profile: PerformanceProfileId) => void;
  profile: PerformanceProfileId;
  profiles: ManifestPerformanceProfile[];
  server: ResolvedServerManifest | null;
}>;

export function ProfileScreen({
  error,
  isBuilding,
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
      actions={
        <AppButton disabled={!profile || isBuilding} onClick={onContinue}>
          {isBuilding ? t("profile.building") : t("profile.continue")}
        </AppButton>
      }
      eyebrow={t("profile.eyebrow")}
      lead={t("profile.lead", { server: serverName })}
      title={t("profile.title")}
    >
      <div className="grid gap-4">
        {error ? (
          <StatusRow detail={error} label={t("profile.errorLabel")} tone="error" />
        ) : null}
        <AppToggleGroup
          className="grid w-full auto-rows-fr items-stretch gap-3 sm:grid-cols-[repeat(auto-fit,minmax(12rem,1fr))]"
          onValueChange={(value) => {
            if (value) {
              onProfileChange(value as PerformanceProfileId);
            }
          }}
          type="single"
          value={profile}
        >
          {profiles.map((item) => {
            const Icon = profileIcon(item);

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
                    {item.label}
                  </span>
                  <span
                    className="text-sm font-normal text-muted-foreground"
                    data-slot="choice-copy"
                  >
                    {t(profileDetailKey(item))}
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

function profileDetailKey(profile: ManifestPerformanceProfile) {
  if (profile.includesShaders) {
    return "profiles.dynamic.shaders";
  }

  if (profile.recommendedMemoryMb <= 3072) {
    return "profiles.dynamic.light";
  }

  return "profiles.dynamic.standard";
}
