import type { ComponentProps, ReactNode } from "react";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

type AppTooltipProviderProps = Readonly<ComponentProps<typeof TooltipProvider>>;

type AppTooltipProps = Readonly<{
  children: ReactNode;
  label: string;
}>;

function AppTooltipProvider(props: AppTooltipProviderProps) {
  return <TooltipProvider {...props} />;
}

function AppTooltip({ children, label }: AppTooltipProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>{children}</TooltipTrigger>
      <TooltipContent side="bottom">{label}</TooltipContent>
    </Tooltip>
  );
}

export { AppTooltip, AppTooltipProvider };
