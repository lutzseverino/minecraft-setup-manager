import { useCallback, useEffect, useRef, useState } from "react";

import {
  checkForAppUpdate,
  downloadAndInstallAppUpdate,
  relaunchApplication,
} from "@/lib/tauri";
import type {
  AppUpdateDownloadProgress,
  AppUpdateInfo,
} from "@/lib/types";

type AppUpdateStatus =
  | "unsupported"
  | "checking"
  | "current"
  | "available"
  | "downloading"
  | "installing"
  | "restarting"
  | "ready_to_restart"
  | "error";

type AppUpdateFailure = "check" | "install" | "restart";

export type AppUpdaterState = Readonly<{
  error: string | null;
  failure: AppUpdateFailure | null;
  progress: AppUpdateDownloadProgress | null;
  status: AppUpdateStatus;
  update: AppUpdateInfo | null;
}>;

const initialState: AppUpdaterState = {
  error: null,
  failure: null,
  progress: null,
  status: "unsupported",
  update: null,
};

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function useAppUpdater() {
  const checkedOnMount = useRef(false);
  const [state, setState] = useState<AppUpdaterState>(initialState);

  const checkForUpdate = useCallback(async () => {
    setState((current) => ({
      ...current,
      error: null,
      failure: null,
      progress: null,
      status: "checking",
      update: null,
    }));

    try {
      const update = await checkForAppUpdate();
      setState({
        error: null,
        failure: null,
        progress: null,
        status:
          update === undefined
            ? "unsupported"
            : update === null
              ? "current"
              : "available",
        update: update ?? null,
      });
    } catch (error) {
      setState({
        error: errorMessage(error),
        failure: "check",
        progress: null,
        status: "error",
        update: null,
      });
    }
  }, []);

  const restart = useCallback(async () => {
    setState((current) => ({
      ...current,
      error: null,
      failure: null,
      status: "restarting",
    }));

    try {
      await relaunchApplication();
    } catch (error) {
      setState((current) => ({
        ...current,
        error: errorMessage(error),
        failure: "restart",
        status: "ready_to_restart",
      }));
    }
  }, []);

  const installAndRestart = useCallback(async () => {
    setState((current) => ({
      ...current,
      error: null,
      failure: null,
      progress: null,
      status: "downloading",
    }));

    try {
      await downloadAndInstallAppUpdate((progress) => {
        setState((current) => ({
          ...current,
          progress,
          status: progress.downloadComplete ? "installing" : "downloading",
        }));
      });
    } catch (error) {
      setState((current) => ({
        ...current,
        error: errorMessage(error),
        failure: "install",
        status: "error",
      }));
      return;
    }

    await restart();
  }, [restart]);

  const retry = useCallback(async () => {
    if (state.failure === "install") {
      await installAndRestart();
    } else if (state.failure === "restart") {
      await restart();
    } else {
      await checkForUpdate();
    }
  }, [checkForUpdate, installAndRestart, restart, state.failure]);

  useEffect(() => {
    if (checkedOnMount.current) {
      return;
    }

    checkedOnMount.current = true;
    void checkForUpdate();
  }, [checkForUpdate]);

  return {
    checkForUpdate,
    installAndRestart,
    isUpdating: ["downloading", "installing", "restarting"].includes(
      state.status,
    ),
    restart,
    retry,
    state,
  };
}

export type AppUpdaterController = ReturnType<typeof useAppUpdater>;
