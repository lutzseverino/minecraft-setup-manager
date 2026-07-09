import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { AppTooltipProvider } from "@/components/app/app-tooltip";
import { Stepper, StepperStep } from "@/components/app/stepper";
import {
  fallbackDetections,
  wizardSteps,
} from "@/config/setup-options";
import "@/i18n";
import {
  detectLaunchers,
  exportDiagnostics,
  getInstallPlan,
  listSavedServers,
  resolveServerManifest,
  startInstall,
  validateInstallation,
} from "@/lib/tauri";
import type {
  DiagnosticBundle,
  InstallPlan,
  InstallProgress,
  LauncherDetection,
  LauncherKind,
  PerformanceProfileId,
  ResolvedServerManifest,
  SavedServerEntry,
  ValidationResult,
  WizardStepId,
} from "@/lib/types";
import { DiagnosticsScreen } from "@/screens/diagnostics-screen";
import { DoneScreen } from "@/screens/done-screen";
import { InstallScreen } from "@/screens/install-screen";
import { LauncherScreen } from "@/screens/launcher-screen";
import { ProfileScreen } from "@/screens/profile-screen";
import { ServerScreen } from "@/screens/server-screen";

function stepIndex(step: WizardStepId) {
  return wizardSteps.findIndex((item) => item.id === step);
}

export default function App() {
  const { t } = useTranslation();
  const [step, setStep] = useState<WizardStepId>("server");
  const [savedServers, setSavedServers] = useState<SavedServerEntry[]>([]);
  const [serverAddress, setServerAddress] = useState("");
  const [resolvedServer, setResolvedServer] =
    useState<ResolvedServerManifest | null>(null);
  const [resolveError, setResolveError] = useState<string | null>(null);
  const [isResolvingServer, setIsResolvingServer] = useState(false);
  const [detections, setDetections] =
    useState<LauncherDetection[]>(fallbackDetections);
  const [launcher, setLauncher] = useState<LauncherKind>("official");
  const [profile, setProfile] = useState<PerformanceProfileId>("");
  const [plan, setPlan] = useState<InstallPlan | null>(null);
  const [installProgress, setInstallProgress] = useState<InstallProgress | null>(
    null,
  );
  const [validationResult, setValidationResult] =
    useState<ValidationResult | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticBundle | null>(null);

  const currentStepIndex = stepIndex(step);
  const installRequest = useMemo(
    () => ({
      serverId: resolvedServer?.server.id ?? "",
      manifestFingerprint: resolvedServer?.manifestFingerprint ?? "",
      launcher,
      profile,
      serverAddress:
        resolvedServer?.manifest.server.address ?? serverAddress.trim(),
    }),
    [launcher, profile, resolvedServer, serverAddress],
  );

  useEffect(() => {
    void refreshSavedServers();
  }, []);

  useEffect(() => {
    if (step === "launcher") {
      void refreshDetections();
    }
  }, [step]);

  async function refreshSavedServers() {
    setSavedServers(await listSavedServers());
  }

  async function refreshDetections() {
    const nextDetections = await detectLaunchers();
    setDetections(nextDetections);
    setLauncher((current) => {
      const keepsCurrent = nextDetections.some(
        (item) => item.kind === current && item.status !== "not_found",
      );
      if (keepsCurrent) {
        return current;
      }

      const detected = nextDetections.find((item) => item.status === "detected");
      if (detected) {
        return detected.kind;
      }

      const selectable = nextDetections.find(
        (item) => item.status !== "not_found",
      );
      return selectable ? selectable.kind : current;
    });
  }

  async function resolveServer(address = serverAddress) {
    setIsResolvingServer(true);
    setResolveError(null);

    try {
      const nextResolvedServer = await resolveServerManifest({ address });
      setResolvedServer(nextResolvedServer);
      setServerAddress(nextResolvedServer.server.address);
      setLauncher(nextResolvedServer.server.selectedLauncher);
      const savedProfile = nextResolvedServer.server.selectedProfile;
      const nextProfile = nextResolvedServer.manifest.profiles.some(
        (item) => item.id === savedProfile,
      )
        ? savedProfile
        : (nextResolvedServer.manifest.profiles[0]?.id ?? "");
      setProfile(nextProfile);
      await refreshSavedServers();
    } catch (error) {
      setResolveError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsResolvingServer(false);
    }
  }

  async function buildPlan() {
    const nextPlan = await getInstallPlan(installRequest);
    setPlan(nextPlan);
    setStep("install");
  }

  async function runInstall() {
    try {
      const progress = await startInstall(installRequest);
      setInstallProgress(progress);
      setPlan(progress.plan);
    } catch (error) {
      const fallbackPlan = plan ?? (await getInstallPlan(installRequest));
      setInstallProgress({
        phase: "failed",
        percent: 0,
        plan: fallbackPlan,
        log: [
          t("install.failedLog", {
            message: error instanceof Error ? error.message : String(error),
          }),
        ],
      });
      setPlan(fallbackPlan);
    }
  }

  async function runValidation() {
    try {
      const result = await validateInstallation(installRequest);
      setValidationResult(result);
      setStep("validate");
    } catch (error) {
      setValidationResult({
        overall: "fail",
        checks: [
          {
            id: "local_setup",
            label: t("diagnostics.checks.local_setup.label"),
            detail: t("diagnostics.checks.local_setup.detail", {
              message: error instanceof Error ? error.message : String(error),
            }),
            status: "fail",
          },
        ],
      });
      setStep("validate");
    }
  }

  async function handleExportDiagnostics() {
    try {
      setDiagnostics(await exportDiagnostics());
    } catch (error) {
      setDiagnostics({
        path: "",
        summary: t("done.reportFailed", {
          message: error instanceof Error ? error.message : String(error),
        }),
      });
    }
  }

  function restart() {
    setStep("server");
    setPlan(null);
    setInstallProgress(null);
    setValidationResult(null);
    setDiagnostics(null);
  }

  return (
    <AppTooltipProvider>
      <div className="min-h-screen">
        <main className="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-8 px-4 py-6 sm:px-6 lg:px-8">
          <header className="border-b-2 border-b-[var(--bevel-line)] pb-6">
            <Stepper aria-label={t("steps.ariaLabel")}>
              {wizardSteps.map((item, index) => (
                <StepperStep
                  disabled={index > currentStepIndex}
                  key={item.id}
                  label={t(item.labelKey)}
                  number={index + 1}
                  onClick={() => {
                    if (index <= currentStepIndex) {
                      setStep(item.id);
                    }
                  }}
                  state={
                    index < currentStepIndex
                      ? "complete"
                      : index === currentStepIndex
                        ? "current"
                        : "upcoming"
                  }
                />
              ))}
            </Stepper>
          </header>

          {step === "server" ? (
            <ServerScreen
              address={serverAddress}
              error={resolveError}
              isResolving={isResolvingServer}
              onAddressChange={setServerAddress}
              onContinue={() => setStep("launcher")}
              onResolve={() => void resolveServer()}
              onSelectSavedServer={(server) => {
                setServerAddress(server.address);
                void resolveServer(server.address);
              }}
              resolved={resolvedServer}
              savedServers={savedServers}
            />
          ) : null}
          {step === "launcher" ? (
            <LauncherScreen
              detections={detections}
              launcher={launcher}
              onContinue={() => setStep("profile")}
              onRefresh={refreshDetections}
              onSelect={setLauncher}
            />
          ) : null}
          {step === "profile" ? (
            <ProfileScreen
              onContinue={buildPlan}
              onProfileChange={setProfile}
              profile={profile}
              profiles={resolvedServer?.manifest.profiles ?? []}
              server={resolvedServer}
            />
          ) : null}
          {step === "install" ? (
            <InstallScreen
              installProgress={installProgress}
              onContinue={runValidation}
              onInstall={runInstall}
              plan={plan}
            />
          ) : null}
          {step === "validate" ? (
            <DiagnosticsScreen
              onContinue={() => setStep("done")}
              result={validationResult}
            />
          ) : null}
          {step === "done" ? (
            <DoneScreen
              diagnostics={diagnostics}
              onExportDiagnostics={handleExportDiagnostics}
              onRestart={restart}
              plan={plan}
            />
          ) : null}
        </main>
      </div>
    </AppTooltipProvider>
  );
}
