import type { ComponentProps } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type AppButtonProps = Readonly<ComponentProps<typeof Button>>;

export function AppButton({ className, ...props }: AppButtonProps) {
  return (
    <Button
      className={cn(
        "mc-button h-10 px-4 font-mono text-xs leading-none tracking-[0.12em] uppercase",
        className,
      )}
      {...props}
    />
  );
}
