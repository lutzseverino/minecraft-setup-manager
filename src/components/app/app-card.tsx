import type { ComponentProps } from "react";

import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";

type AppCardProps = Readonly<ComponentProps<typeof Card>>;
type AppCardHeaderProps = Readonly<ComponentProps<typeof CardHeader>>;
type AppCardTitleProps = Readonly<ComponentProps<typeof CardTitle>>;
type AppCardDescriptionProps = Readonly<ComponentProps<typeof CardDescription>>;
type AppCardActionProps = Readonly<ComponentProps<typeof CardAction>>;
type AppCardContentProps = Readonly<ComponentProps<typeof CardContent>>;
type AppCardFooterProps = Readonly<ComponentProps<typeof CardFooter>>;

export function AppCard({ className, ...props }: AppCardProps) {
  return (
    <Card className={cn("mc-panel gap-0 bg-card py-0", className)} {...props} />
  );
}

export function AppCardHeader({ className, ...props }: AppCardHeaderProps) {
  return (
    <CardHeader
      className={cn(
        "rounded-t-[1px] border-b-2 border-b-[var(--bevel-line)] bg-muted/70 px-4 py-2.5",
        className,
      )}
      {...props}
    />
  );
}

export function AppCardTitle({ className, ...props }: AppCardTitleProps) {
  return (
    <CardTitle
      className={cn(
        "font-display text-[0.78rem] leading-tight font-normal tracking-wide",
        className,
      )}
      {...props}
    />
  );
}

export function AppCardDescription(props: AppCardDescriptionProps) {
  return <CardDescription {...props} />;
}

export function AppCardAction(props: AppCardActionProps) {
  return <CardAction {...props} />;
}

export function AppCardContent({ className, ...props }: AppCardContentProps) {
  return <CardContent className={cn("p-4", className)} {...props} />;
}

export function AppCardFooter(props: AppCardFooterProps) {
  return <CardFooter {...props} />;
}
