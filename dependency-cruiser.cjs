module.exports = {
  forbidden: [
    {
      name: "no-circular",
      severity: "error",
      from: {},
      to: {
        circular: true,
      },
    },
    {
      name: "no-orphans",
      severity: "warn",
      from: {
        orphan: true,
        pathNot: [
          "^src/main[.]tsx$",
          "^src/vite-env[.]d[.]ts$",
          "^src/lib/types[.]ts$",
          "^src/i18n/locales/",
        ],
      },
      to: {},
    },
    {
      name: "ui-components-stay-foundational",
      severity: "error",
      comment:
        "Low-level shadcn UI components must not import app-specific composition.",
      from: { path: "^src/components/ui/" },
      to: { path: "^src/(components/app|screens|lib/tauri)" },
    },
    {
      name: "app-components-do-not-import-screens",
      severity: "error",
      comment:
        "Reusable app components should not depend on screen-level wizard composition.",
      from: { path: "^src/components/app/" },
      to: { path: "^src/screens/" },
    },
    {
      name: "hooks-own-state-not-rendering",
      severity: "error",
      comment:
        "Wizard hooks own orchestration state and must not depend on screen or component rendering.",
      from: { path: "^src/hooks/" },
      to: { path: "^src/(components|screens)/" },
    },
    {
      name: "config-stays-pure",
      severity: "error",
      comment:
        "Setup configuration should not depend on screen composition or command execution.",
      from: { path: "^src/config/" },
      to: { path: "^src/(screens|lib/tauri)" },
    },
    {
      name: "screens-use-typed-tauri-wrapper",
      severity: "error",
      comment:
        "Screens call typed command wrappers instead of importing Tauri APIs directly.",
      from: { path: "^src/screens/" },
      to: { path: "^node_modules/@tauri-apps/api/" },
    },
    {
      name: "tauri-wrapper-is-only-tauri-api-consumer",
      severity: "error",
      comment:
        "Keep all Tauri guest APIs behind src/lib/tauri.ts so native contracts stay typed.",
      from: { path: "^src/(?!lib/tauri[.]ts)" },
      to: { path: "^node_modules/@tauri-apps/" },
    },
  ],
  options: {
    doNotFollow: {
      path: "node_modules",
    },
    enhancedResolveOptions: {
      exportsFields: ["exports"],
      conditionNames: ["import", "require", "node", "default"],
    },
    tsConfig: {
      fileName: "tsconfig.depcruise.json",
    },
  },
};
