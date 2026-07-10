import { useTranslation } from "react-i18next";

import { AppTooltipProvider } from "@/components/app/app-tooltip";
import { AppUpdater } from "@/components/app/app-updater";
import { Stepper, StepperStep } from "@/components/app/stepper";
import { wizardSteps } from "@/config/setup-options";
import { useSetupWizard } from "@/hooks/use-setup-wizard";
import { useAppUpdater } from "@/hooks/use-app-updater";
import "@/i18n";
import type { WizardStepId } from "@/lib/types";
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
  const wizard = useSetupWizard();
  const updater = useAppUpdater();
  const currentStepIndex = stepIndex(wizard.step);
  const setupIsMutating = wizard.isInstalling || wizard.isValidating;

  return (
    <AppTooltipProvider>
      <div className="min-h-screen">
        <main className="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-8 px-4 py-6 sm:px-6 lg:px-8">
          <AppUpdater
            controller={updater}
            updateBlocked={setupIsMutating}
          />
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
                      wizard.setStep(item.id);
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

          {wizard.step === "server" ? (
            <ServerScreen
              address={wizard.serverAddress}
              error={wizard.resolveError}
              isResolving={wizard.isResolvingServer}
              onAddressChange={wizard.changeAddress}
              onSetupCodeChange={wizard.changeSetupCode}
              onContinue={() => wizard.setStep("launcher")}
              onResolve={() => void wizard.resolveServer()}
              onSelectSavedServer={(server) => {
                void wizard.resolveServer(server.address);
              }}
              resolved={wizard.resolvedServer}
              savedServers={wizard.savedServers}
              setupCode={wizard.setupCode}
            />
          ) : null}
          {wizard.step === "launcher" ? (
            <LauncherScreen
              detections={wizard.detections}
              error={wizard.launcherError}
              launcher={wizard.launcher}
              onContinue={() => wizard.setStep("profile")}
              onRefresh={wizard.refreshDetections}
              onSelect={wizard.setLauncher}
            />
          ) : null}
          {wizard.step === "profile" ? (
            <ProfileScreen
              error={wizard.planError}
              isBuilding={wizard.isBuildingPlan}
              onContinue={wizard.buildPlan}
              onProfileChange={wizard.setProfile}
              profile={wizard.profile}
              profiles={wizard.resolvedServer?.manifest.profiles ?? []}
              server={wizard.resolvedServer}
            />
          ) : null}
          {wizard.step === "install" ? (
            <InstallScreen
              installProgress={wizard.installProgress}
              isAppUpdating={updater.isUpdating}
              isInstalling={wizard.isInstalling}
              isValidating={wizard.isValidating}
              onContinue={wizard.runValidation}
              onInstall={wizard.runInstall}
              plan={wizard.plan}
            />
          ) : null}
          {wizard.step === "validate" ? (
            <DiagnosticsScreen
              onContinue={() => wizard.setStep("done")}
              result={wizard.validationResult}
            />
          ) : null}
          {wizard.step === "done" ? (
            <DoneScreen
              diagnostics={wizard.diagnostics}
              onExportDiagnostics={wizard.exportSetupDiagnostics}
              onRestart={wizard.restart}
              plan={wizard.plan}
            />
          ) : null}
        </main>
      </div>
    </AppTooltipProvider>
  );
}
