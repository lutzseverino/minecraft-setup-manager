import { TerminalIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  AppCard,
  AppCardContent,
  AppCardHeader,
  AppCardTitle,
} from "./app-card";

type ProgressLogProps = Readonly<{
  entries: string[];
}>;

export function ProgressLog({ entries }: ProgressLogProps) {
  const { t } = useTranslation();

  return (
    <AppCard>
      <AppCardHeader className="grid-cols-[1fr_auto]">
        <AppCardTitle className="flex items-center gap-2">
          <TerminalIcon className="size-4" />
          {t("progressLog.title")}
        </AppCardTitle>
      </AppCardHeader>
      <AppCardContent>
        <div className="mc-console max-h-52 overflow-auto p-3 font-mono text-xs leading-6">
          {entries.map((entry) => (
            <div className="whitespace-pre-wrap" key={entry}>
              {entry}
            </div>
          ))}
        </div>
      </AppCardContent>
    </AppCard>
  );
}
