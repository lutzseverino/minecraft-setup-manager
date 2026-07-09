import type { ComponentProps } from "react";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

type AppToggleGroupProps = Readonly<ComponentProps<typeof ToggleGroup>>;
type AppToggleGroupItemProps = Readonly<
  ComponentProps<typeof ToggleGroupItem> & {
    treatment?: "default" | "choice";
  }
>;

export function AppToggleGroup(props: AppToggleGroupProps) {
  return <ToggleGroup {...props} />;
}

export function AppToggleGroupItem({
  className,
  treatment = "default",
  variant,
  ...props
}: AppToggleGroupItemProps) {
  return (
    <ToggleGroupItem
      className={cn(
        treatment === "choice" &&
          "mc-panel h-auto min-h-28 w-full justify-start overflow-hidden whitespace-normal bg-card! p-4 text-left transition-[filter] hover:brightness-[1.04] disabled:opacity-55 disabled:hover:brightness-100 [&_span[data-slot=choice-copy]]:w-full data-[state=on]:bg-primary! data-[state=on]:text-primary-foreground data-[state=on]:[outline:2px_solid_var(--primary-foreground)] data-[state=on]:[outline-offset:-5px] data-[state=on]:[&_span[data-slot=choice-copy]]:text-primary-foreground/85 data-[state=on]:[&_[data-slot=choice-meta]]:text-primary-foreground!",
        className,
      )}
      variant={treatment === "choice" ? "outline" : variant}
      {...props}
    />
  );
}
