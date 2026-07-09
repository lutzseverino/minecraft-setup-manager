export type ServerId = "maresme-mc";

export type ServerConfig = Readonly<{
  id: ServerId;
  displayName: string;
  defaultAddress: string;
  gameDirectoryName: string;
  minecraftVersion: string;
  fabricLoaderVersion: string;
  requiredMods: string[];
  balancedExtras: string[];
  shadersExtras: string[];
}>;

export const serverCatalog = [
  {
    id: "maresme-mc",
    displayName: "Maresme MC",
    defaultAddress: "localhost",
    gameDirectoryName: "Maresme MC",
    minecraftVersion: "26.1.2",
    fabricLoaderVersion: "0.19.3",
    requiredMods: [
      "Fabric API",
      "Simple Voice Chat",
      "Sodium",
      "Lithium",
      "ImmediatelyFast",
    ],
    balancedExtras: [
      "Sodium Extra",
      "Dynamic FPS",
      "Entity Culling",
      "FerriteCore",
      "Mod Menu",
    ],
    shadersExtras: ["Iris", "Reese's Sodium Options"],
  },
] satisfies ServerConfig[];

export const defaultServer = serverCatalog[0];

export function getServerConfig(serverId: string) {
  return (
    serverCatalog.find((server) => server.id === serverId) ?? defaultServer
  );
}
