import type { ReactNode } from "react";

type ScreenShellProps = Readonly<{
  actions?: ReactNode;
  children: ReactNode;
  eyebrow: string;
  lead: string;
  title: string;
}>;

export function ScreenShell({
  actions,
  children,
  eyebrow,
  lead,
  title,
}: ScreenShellProps) {
  return (
    <section className="grid gap-6 lg:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)] lg:items-start">
      <div className="min-w-0">
        <p className="flex items-center gap-2 font-display text-[0.7rem] tracking-tight text-primary">
          <span
            aria-hidden
            className="mc-panel inline-block size-3 shrink-0"
            style={{
              background:
                "linear-gradient(var(--primary) 0 38%, var(--secondary) 38% 100%)",
            }}
          />
          {eyebrow}
        </p>
        <h1 className="mt-4 max-w-xl text-3xl leading-[1.1] font-semibold tracking-tight text-balance md:text-[2.6rem]">
          {title}
        </h1>
        <p className="mt-4 max-w-xl text-base leading-7 text-muted-foreground">
          {lead}
        </p>
        {actions ? (
          <div className="mt-6 flex flex-wrap gap-2">{actions}</div>
        ) : null}
      </div>
      <div className="min-w-0">{children}</div>
    </section>
  );
}
