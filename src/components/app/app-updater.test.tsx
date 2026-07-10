import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AppUpdater } from "@/components/app/app-updater";
import type { AppUpdaterController } from "@/hooks/use-app-updater";
import "@/i18n";

const availableUpdate = {
  currentVersion: "0.1.3",
  date: "2026-07-10T12:00:00Z",
  notes: "Security and reliability improvements.",
  version: "0.1.4",
};

afterEach(() => {
  cleanup();
});

function availableController(installAndRestart = vi.fn()) {
  return {
    checkForUpdate: vi.fn(),
    installAndRestart,
    isUpdating: false,
    restart: vi.fn(),
    retry: vi.fn(),
    state: {
      error: null,
      failure: null,
      progress: null,
      status: "available",
      update: availableUpdate,
    },
  } satisfies AppUpdaterController;
}

describe("AppUpdater", () => {
  it("defers installation and disables consent while setup is mutating", () => {
    const installAndRestart = vi.fn();
    render(
      <AppUpdater
        controller={availableController(installAndRestart)}
        updateBlocked
      />,
    );

    const consent = screen.getByRole("button", { name: "Update and restart" });
    expect((consent as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(consent);
    expect(installAndRestart).not.toHaveBeenCalled();
  });

  it("collapses an available update without installing it", () => {
    const installAndRestart = vi.fn();
    render(
      <AppUpdater
        controller={availableController(installAndRestart)}
        updateBlocked={false}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Later" }));

    expect(screen.getByText("App update v0.1.4 is available.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Review" })).toBeTruthy();
    expect(installAndRestart).not.toHaveBeenCalled();
  });
});
