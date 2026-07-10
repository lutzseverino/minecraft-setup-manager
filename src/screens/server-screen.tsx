import {
  DownloadIcon,
  PlusIcon,
  RefreshCwIcon,
  ServerIcon,
} from "lucide-react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type {
  ResolvedServerManifest,
  SavedServerEntry,
  ServerUpdateStatus,
} from "@/lib/types";

type ServerScreenProps = Readonly<{
  address: string;
  error: string | null;
  isResolving: boolean;
  onAddressChange: (address: string) => void;
  onSetupCodeChange: (code: string) => void;
  onContinue: () => void;
  onResolve: () => void;
  onSelectSavedServer: (server: SavedServerEntry) => void;
  resolved: ResolvedServerManifest | null;
  savedServers: SavedServerEntry[];
  setupCode: string;
}>;

export function ServerScreen({
  address,
  error,
  isResolving,
  onAddressChange,
  onSetupCodeChange,
  onContinue,
  onResolve,
  onSelectSavedServer,
  resolved,
  savedServers,
  setupCode,
}: ServerScreenProps) {
  const { t } = useTranslation();

  return (
    <ScreenShell
      actions={
        <>
          <AppButton
            disabled={isResolving || !address.trim()}
            onClick={onResolve}
            variant="outline"
          >
            <DownloadIcon data-icon="inline-start" />
            {isResolving ? t("server.checking") : t("server.fetch")}
          </AppButton>
          <AppButton disabled={!resolved} onClick={onContinue}>
            {t("server.continue")}
          </AppButton>
        </>
      }
      eyebrow={t("server.eyebrow")}
      lead={t("server.lead")}
      title={t("server.title")}
    >
      <div className="grid gap-4">
        <AppCard>
          <AppCardHeader>
            <AppCardTitle className="flex items-center gap-2 text-sm">
              <PlusIcon className="size-4" />
              {t("server.addTitle")}
            </AppCardTitle>
          </AppCardHeader>
          <AppCardContent className="grid gap-2">
            <Label htmlFor="server-address">{t("server.addressLabel")}</Label>
            <Input
              autoComplete="off"
              id="server-address"
              onChange={(event) => onAddressChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && address.trim() && !isResolving) {
                  onResolve();
                }
              }}
              placeholder={t("server.addressPlaceholder")}
              value={address}
            />
            <p className="text-sm text-muted-foreground">{t("server.help")}</p>
            <div className="mt-3 grid gap-2 border-t-2 border-[var(--bevel-line)] pt-4">
              <Label htmlFor="setup-code">{t("server.codeLabel")}</Label>
              <Input
                autoComplete="off"
                id="setup-code"
                maxLength={19}
                onChange={(event) => onSetupCodeChange(event.target.value)}
                placeholder={t("server.codePlaceholder")}
                spellCheck={false}
                value={setupCode}
              />
              <p className="text-sm text-muted-foreground">
                {t("server.codeHelp")}
              </p>
            </div>
          </AppCardContent>
        </AppCard>

        {error ? (
          <StatusRow detail={error} label={t("server.errorLabel")} tone="error" />
        ) : null}

        {resolved ? (
          <StatusRow
            detail={t("server.resolved.detail", {
              minecraft: resolved.manifest.minecraft.version,
              version: resolved.manifest.manifestVersion,
            })}
            label={resolved.manifest.displayName}
            meta={t(statusKey(resolved.updateStatus))}
            tone={statusTone(resolved.updateStatus)}
          />
        ) : null}

        <AppCard>
          <AppCardHeader>
            <AppCardTitle className="flex items-center gap-2 text-sm">
              <ServerIcon className="size-4" />
              {t("server.savedTitle")}
            </AppCardTitle>
          </AppCardHeader>
          <AppCardContent className="grid gap-2">
            {savedServers.length > 0 ? (
              savedServers.map((server) => (
                <button
                  className="mc-inset grid grid-cols-[1fr_auto] items-center gap-3 bg-[var(--slot)] p-3 text-left text-sm"
                  key={server.id}
                  onClick={() => onSelectSavedServer(server)}
                  type="button"
                >
                  <span>
                    <span className="block font-medium">{server.displayName}</span>
                    <span className="mt-1 block text-muted-foreground">
                      {server.address}
                    </span>
                  </span>
                  <RefreshCwIcon className="size-4 text-muted-foreground" />
                </button>
              ))
            ) : (
              <p className="mc-inset bg-[var(--slot)] p-3 text-sm text-muted-foreground">
                {t("server.emptySaved")}
              </p>
            )}
          </AppCardContent>
        </AppCard>
      </div>
    </ScreenShell>
  );
}

function statusKey(status: ServerUpdateStatus) {
  if (status === "up_to_date") {
    return "server.status.upToDate";
  }

  if (status === "update_available") {
    return "server.status.updateAvailable";
  }

  return "server.status.newSetup";
}

function statusTone(status: ServerUpdateStatus) {
  if (status === "up_to_date") {
    return "success";
  }

  if (status === "update_available") {
    return "warning";
  }

  return "info";
}
