import { ClipboardCheckIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { AppButton } from "@/components/app/app-button";
import { ScreenShell } from "@/components/app/screen-shell";
import { StatusRow } from "@/components/app/status-row";
import type { ValidationResult } from "@/lib/types";

type DiagnosticsScreenProps = Readonly<{
  onContinue: () => void;
  result: ValidationResult | null;
}>;

function toneForStatus(status: "pass" | "warning" | "fail") {
  if (status === "pass") {
    return "success";
  }

  if (status === "warning") {
    return "warning";
  }

  return "error";
}

export function DiagnosticsScreen({
  onContinue,
  result,
}: DiagnosticsScreenProps) {
  const { t } = useTranslation();
  const canFinish = result !== null && result.overall !== "fail";

  return (
    <ScreenShell
      actions={
        <AppButton disabled={!canFinish} onClick={onContinue}>
          {t("common.done")}
          <ClipboardCheckIcon data-icon="inline-end" />
        </AppButton>
      }
      eyebrow={t("diagnostics.eyebrow")}
      lead={t("diagnostics.lead")}
      title={t("diagnostics.title")}
    >
      <div className="grid gap-2">
        {result?.checks.map((check) => (
          <StatusRow
            detail={t(`diagnostics.checks.${check.id}.detail`, {
              defaultValue: check.detail,
            })}
            key={check.id}
            label={t(`diagnostics.checks.${check.id}.label`, {
              defaultValue: check.label,
            })}
            tone={toneForStatus(check.status)}
          />
        ))}
      </div>
    </ScreenShell>
  );
}
