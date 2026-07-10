import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAppUpdater } from "@/hooks/use-app-updater";

const tauri = vi.hoisted(() => ({
  checkForAppUpdate: vi.fn(),
  downloadAndInstallAppUpdate: vi.fn(),
  relaunchApplication: vi.fn(),
}));

vi.mock("@/lib/tauri", () => tauri);

const availableUpdate = {
  currentVersion: "0.1.3",
  date: "2026-07-10T12:00:00Z",
  notes: "Security and reliability improvements.",
  version: "0.1.4",
};

beforeEach(() => {
  vi.clearAllMocks();
  tauri.checkForAppUpdate.mockResolvedValue(undefined);
  tauri.downloadAndInstallAppUpdate.mockResolvedValue(undefined);
  tauri.relaunchApplication.mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
});

describe("useAppUpdater", () => {
  it("checks once on startup without downloading", async () => {
    tauri.checkForAppUpdate.mockResolvedValue(availableUpdate);

    const { result } = renderHook(() => useAppUpdater());

    await waitFor(() => expect(result.current.state.status).toBe("available"));
    expect(tauri.checkForAppUpdate).toHaveBeenCalledTimes(1);
    expect(tauri.downloadAndInstallAppUpdate).not.toHaveBeenCalled();
    expect(tauri.relaunchApplication).not.toHaveBeenCalled();
  });

  it("reports progress and relaunches only after installation", async () => {
    tauri.checkForAppUpdate.mockResolvedValue(availableUpdate);
    tauri.downloadAndInstallAppUpdate.mockImplementation(async (onProgress) => {
      onProgress({
        downloadComplete: false,
        downloadedBytes: 50,
        percent: 50,
        totalBytes: 100,
      });
      onProgress({
        downloadComplete: true,
        downloadedBytes: 100,
        percent: 100,
        totalBytes: 100,
      });
    });
    const { result } = renderHook(() => useAppUpdater());
    await waitFor(() => expect(result.current.state.status).toBe("available"));

    await act(async () => {
      await result.current.installAndRestart();
    });

    expect(tauri.downloadAndInstallAppUpdate).toHaveBeenCalledTimes(1);
    expect(tauri.relaunchApplication).toHaveBeenCalledTimes(1);
    expect(result.current.state.progress?.percent).toBe(100);
  });

  it("keeps an install failure retryable", async () => {
    tauri.checkForAppUpdate.mockResolvedValue(availableUpdate);
    tauri.downloadAndInstallAppUpdate.mockRejectedValueOnce(
      new Error("signature rejected"),
    );
    const { result } = renderHook(() => useAppUpdater());
    await waitFor(() => expect(result.current.state.status).toBe("available"));

    await act(async () => {
      await result.current.installAndRestart();
    });
    expect(result.current.state.status).toBe("error");
    expect(result.current.state.failure).toBe("install");

    await act(async () => {
      await result.current.retry();
    });
    expect(tauri.downloadAndInstallAppUpdate).toHaveBeenCalledTimes(2);
    expect(tauri.relaunchApplication).toHaveBeenCalledTimes(1);
  });
});
