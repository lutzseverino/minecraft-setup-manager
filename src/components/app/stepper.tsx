import { CheckIcon } from "lucide-react";
import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/utils";

type StepperStepState = "complete" | "current" | "upcoming";

type StepperProps = Readonly<ComponentProps<"ol">>;

type StepperStepProps = Readonly<
  Omit<ComponentProps<"button">, "children" | "type"> & {
    label: ReactNode;
    number: number;
    state: StepperStepState;
  }
>;

export function Stepper({ className, ...props }: StepperProps) {
  return (
    <ol
      className={cn("flex items-start gap-1.5 sm:gap-2", className)}
      {...props}
    />
  );
}

export function StepperStep({
  className,
  disabled,
  label,
  number,
  state,
  ...props
}: StepperStepProps) {
  return (
    <li className="flex min-w-0 flex-1 flex-col items-center gap-1.5">
      <button
        {...props}
        aria-current={state === "current" ? "step" : undefined}
        className={cn(
          "mc-slot grid size-11 place-items-center font-display text-sm transition-[filter]",
          state === "complete" && "text-primary-foreground",
          state === "current" && "text-foreground",
          state === "upcoming" && "text-muted-foreground/70",
          !disabled && state !== "current" && "hover:brightness-105",
          disabled && "cursor-default",
          className,
        )}
        data-step={state}
        disabled={disabled}
        title={typeof label === "string" ? label : undefined}
        type="button"
      >
        {state === "complete" ? (
          <CheckIcon className="size-5" strokeWidth={3} />
        ) : (
          <span aria-hidden>{String(number).padStart(2, "0")}</span>
        )}
        <span className="sr-only">{label}</span>
      </button>
      <span
        className={cn(
          "type-label hidden max-w-full truncate sm:block",
          state === "current" ? "text-foreground" : "text-muted-foreground",
        )}
      >
        {label}
      </span>
    </li>
  );
}
