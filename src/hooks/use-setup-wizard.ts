import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { fallbackDetections } from "@/config/setup-options";
import {
  detectLaunchers,
  exportDiagnostics,
  getInstallPlan,
  listSavedServers,
  redeemSetupAttestation,
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

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function useSetupWizard() {
  const { t } = useTranslation();
  const [step, setStep] = useState<WizardStepId>("server");
  const [savedServers, setSavedServers] = useState<SavedServerEntry[]>([]);
  const [serverAddress, setServerAddress] = useState("");
  const [setupCode, setSetupCode] = useState("");
  const [resolvedServer, setResolvedServer] =
    useState<ResolvedServerManifest | null>(null);
  const [resolveError, setResolveError] = useState<string | null>(null);
  const [isResolvingServer, setIsResolvingServer] = useState(false);
  const [detections, setDetections] =
    useState<LauncherDetection[]>(fallbackDetections);
  const [launcher, setLauncher] = useState<LauncherKind>("official");
  const [launcherError, setLauncherError] = useState<string | null>(null);
  const [profile, setProfile] = useState<PerformanceProfileId>("");
  const [plan, setPlan] = useState<InstallPlan | null>(null);
  const [planError, setPlanError] = useState<string | null>(null);
  const [isBuildingPlan, setIsBuildingPlan] = useState(false);
  const [installProgress, setInstallProgress] =
    useState<InstallProgress | null>(null);
  const [isInstalling, setIsInstalling] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [validationResult, setValidationResult] =
    useState<ValidationResult | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticBundle | null>(null);

  const installRequest = useMemo(
    () => ({
      serverId: resolvedServer?.server.id ?? "",
      manifestFingerprint: resolvedServer?.manifestFingerprint ?? "",
      launcher,
      profile,
    }),
    [launcher, profile, resolvedServer],
  );

  const refreshSavedServers = useCallback(async () => {
    try {
      setSavedServers(await listSavedServers());
    } catch (error) {
      setResolveError(errorMessage(error));
    }
  }, []);

  const refreshDetections = useCallback(async () => {
    setLauncherError(null);
    try {
      const nextDetections = await detectLaunchers();
      setDetections(nextDetections);
      setLauncher((current) => {
        const keepsCurrent = nextDetections.some(
          (item) =>
            item.kind === current &&
            item.setupSupported &&
            item.status !== "not_found",
        );
        if (keepsCurrent) {
          return current;
        }

        const detected = nextDetections.find(
          (item) => item.setupSupported && item.status === "detected",
        );
        const selectable = nextDetections.find(
          (item) => item.setupSupported && item.status !== "not_found",
        );
        return detected?.kind ?? selectable?.kind ?? current;
      });
    } catch (error) {
      setLauncherError(errorMessage(error));
      setDetections(fallbackDetections);
    }
  }, []);

  useEffect(() => {
    void refreshSavedServers();
  }, [refreshSavedServers]);

  useEffect(() => {
    if (step === "launcher") {
      void refreshDetections();
    }
  }, [refreshDetections, step]);

  function clearWork() {
    setPlan(null);
    setPlanError(null);
    setInstallProgress(null);
    setValidationResult(null);
    setDiagnostics(null);
  }

  function changeAddress(address: string) {
    setServerAddress(address);
    setResolveError(null);
    if (resolvedServer && address.trim() !== resolvedServer.server.address) {
      setResolvedServer(null);
      setProfile("");
      clearWork();
    }
  }

  function changeSetupCode(code: string) {
    setSetupCode(code.toUpperCase());
  }

  async function resolveServer(address = serverAddress) {
    setIsResolvingServer(true);
    setResolveError(null);
    setResolvedServer(null);
    setProfile("");
    clearWork();

    try {
      const nextResolvedServer = await resolveServerManifest({ address });
      const savedProfile = nextResolvedServer.server.selectedProfile;
      const nextProfile = nextResolvedServer.manifest.profiles.some(
        (item) => item.id === savedProfile,
      )
        ? savedProfile
        : (nextResolvedServer.manifest.profiles[0]?.id ?? "");
      setResolvedServer(nextResolvedServer);
      setServerAddress(nextResolvedServer.server.address);
      setLauncher(nextResolvedServer.server.selectedLauncher);
      setProfile(nextProfile);
      await refreshSavedServers();
    } catch (error) {
      setResolveError(errorMessage(error));
    } finally {
      setIsResolvingServer(false);
    }
  }

  async function buildPlan() {
    if (!resolvedServer || !profile) {
      setPlanError(t("profile.missingSelection"));
      return;
    }

    setIsBuildingPlan(true);
    setPlanError(null);
    setInstallProgress(null);
    setValidationResult(null);
    try {
      const nextPlan = await getInstallPlan(installRequest);
      setPlan(nextPlan);
      setStep("install");
    } catch (error) {
      setPlanError(errorMessage(error));
    } finally {
      setIsBuildingPlan(false);
    }
  }

  async function runInstall() {
    if (!plan || isInstalling) {
      return;
    }

    setIsInstalling(true);
    setInstallProgress(null);
    setValidationResult(null);
    try {
      const progress = await startInstall(installRequest);
      setInstallProgress(progress);
      setPlan(progress.plan);
    } catch (error) {
      setInstallProgress({
        phase: "failed",
        percent: 0,
        plan,
        log: [t("install.failedLog", { message: errorMessage(error) })],
      });
    } finally {
      setIsInstalling(false);
    }
  }

  async function runValidation() {
    if (installProgress?.phase !== "complete" || isValidating) {
      return;
    }

    setIsValidating(true);
    try {
      const result = await validateInstallation(installRequest);
      if (result.overall !== "fail" && setupCode.trim()) {
        try {
          await redeemSetupAttestation({
            ...installRequest,
            challenge: setupCode,
          });
          setValidationResult({
            overall: result.overall,
            checks: [
              ...result.checks,
              {
                id: "setup_attestation",
                label: t("diagnostics.checks.setup_attestation.label"),
                detail: t("diagnostics.checks.setup_attestation.detail"),
                status: "pass",
              },
            ],
          });
        } catch (error) {
          setValidationResult({
            overall: "fail",
            checks: [
              ...result.checks,
              {
                id: "setup_attestation",
                label: t("diagnostics.checks.setup_attestation.label"),
                detail: t("diagnostics.checks.setup_attestation.failed", {
                  message: errorMessage(error),
                }),
                status: "fail",
              },
            ],
          });
        }
      } else {
        setValidationResult(result);
      }
    } catch (error) {
      setValidationResult({
        overall: "fail",
        checks: [
          {
            id: "local_setup",
            label: t("diagnostics.checks.local_setup.label"),
            detail: t("diagnostics.checks.local_setup.detail", {
              message: errorMessage(error),
            }),
            status: "fail",
          },
        ],
      });
    } finally {
      setIsValidating(false);
      setStep("validate");
    }
  }

  async function exportSetupDiagnostics() {
    try {
      setDiagnostics(await exportDiagnostics());
    } catch (error) {
      setDiagnostics({
        path: "",
        summary: t("done.reportFailed", { message: errorMessage(error) }),
      });
    }
  }

  function restart() {
    setStep("server");
    setServerAddress("");
    setSetupCode("");
    setResolvedServer(null);
    setProfile("");
    setResolveError(null);
    clearWork();
    void refreshSavedServers();
  }

  return {
    buildPlan,
    changeAddress,
    changeSetupCode,
    detections,
    diagnostics,
    exportSetupDiagnostics,
    installProgress,
    isBuildingPlan,
    isInstalling,
    isResolvingServer,
    isValidating,
    launcher,
    launcherError,
    plan,
    planError,
    profile,
    refreshDetections,
    resolveError,
    resolveServer,
    resolvedServer,
    restart,
    runInstall,
    runValidation,
    savedServers,
    serverAddress,
    setupCode,
    setLauncher,
    setProfile,
    setStep,
    step,
    validationResult,
  };
}
