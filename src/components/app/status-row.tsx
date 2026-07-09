import {
  AlertTriangleIcon,
  CheckCircle2Icon,
  CircleDashedIcon,
  InfoIcon,
  LoaderCircleIcon,
  XCircleIcon,
} from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type StatusTone = "idle" | "working" | "success" | "warning" | "error" | "info";

const toneIcon = {
  idle: CircleDashedIcon,
  working: LoaderCircleIcon,
  success: CheckCircle2Icon,
  warning: AlertTriangleIcon,
  error: XCircleIcon,
  info: InfoIcon,
} satisfies Record<StatusTone, typeof CircleDashedIcon>;

const toneLabel = {
  idle: "status.pending",
  working: "status.working",
  success: "status.ready",
  warning: "status.review",
  error: "status.blocked",
  info: "status.info",
} satisfies Record<StatusTone, string>;

const toneBadge = {
  idle: "outline",
  working: "info",
  success: "success",
  warning: "warning",
  error: "destructive",
  info: "info",
} as const;

type StatusRowProps = Readonly<{
  detail?: ReactNode;
  label: ReactNode;
  meta?: ReactNode;
  tone: StatusTone;
}>;

export function StatusRow({ detail, label, meta, tone }: StatusRowProps) {
  const { t } = useTranslation();
  const Icon = toneIcon[tone];

  return (
    <div className="mc-inset grid grid-cols-[auto_1fr_auto] items-start gap-3 bg-[var(--slot)] p-3">
      <Icon
        className={cn(
          "mt-0.5 size-4",
          tone === "working" && "animate-spin text-info",
          tone === "success" && "text-success",
          tone === "warning" && "text-warning",
          tone === "error" && "text-destructive",
          tone === "info" && "text-info",
          tone === "idle" && "text-muted-foreground",
        )}
      />
      <div className="min-w-0">
        <div className="text-sm font-medium">{label}</div>
        {detail ? (
          <div className="mt-1 text-sm text-muted-foreground">{detail}</div>
        ) : null}
      </div>
      <div className="flex flex-col items-end gap-1">
        <Badge
          className="rounded-[var(--radius)] px-1.5 font-mono text-[0.6rem] tracking-[0.1em] uppercase"
          style={{ borderRadius: "var(--radius)" }}
          variant={toneBadge[tone]}
        >
          {t(toneLabel[tone])}
        </Badge>
        {meta ? (
          <div className="font-mono text-xs text-muted-foreground">{meta}</div>
        ) : null}
      </div>
    </div>
  );
}
