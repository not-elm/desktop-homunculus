import { host } from './host';

/**
 * Assets API namespace for querying available mod assets.
 *
 * Provides access to the asset registry, which contains the engine's built-in
 * assets, assets declared by installed mods, and files imported by the user.
 * Assets are referenced by their globally unique ID (e.g., `"vrm:elmer"`,
 * `"my-mod:click"`).
 *
 * @example
 * ```typescript
 * // List all available assets
 * const all = await assets.list();
 *
 * // Filter by type
 * const vrms = await assets.list({ type: "vrm" });
 *
 * // Filter by mod
 * const myModAssets = await assets.list({ mod: "my-mod" });
 *
 * // Built-in assets shipped with the engine
 * const builtin = await assets.list({ mod: "builtin" });
 * ```
 */
export namespace assets {
  /** The type of an asset. */
  export type AssetType = 'vrm' | 'vrma' | 'sound' | 'image' | 'html';

  /** Information about a registered asset. */
  export interface AssetInfo {
    /** Globally unique asset ID. */
    id: string;
    /** The asset type. */
    type: AssetType;
    /**
     * Where this asset comes from: `"builtin"` for assets shipped with the
     * engine, `"local"` for files imported by the user, otherwise the name of
     * the mod that declares it.
     */
    mod: string;
    /** Optional description of the asset. */
    description?: string;
  }

  /** Filter options for listing assets. */
  export interface AssetFilter {
    /** Filter by asset type. */
    type?: AssetType;
    /** Filter by origin: a mod name, `"builtin"`, or `"local"`. */
    mod?: string;
  }

  /**
   * Lists available assets, optionally filtered by type and/or mod.
   *
   * @param filter - Optional filter criteria
   * @returns Array of matching asset info objects
   *
   * @example
   * ```typescript
   * // Get all assets
   * const all = await assets.list();
   *
   * // Get only VRM models
   * const vrms = await assets.list({ type: "vrm" });
   *
   * // Get assets from a specific mod
   * const modAssets = await assets.list({ mod: "my-mod" });
   *
   * // Get the engine's built-in assets
   * const builtin = await assets.list({ mod: "builtin" });
   *
   * // Combine filters
   * const sounds = await assets.list({ type: "sound", mod: "my-mod" });
   * ```
   */
  export async function list(filter?: AssetFilter): Promise<AssetInfo[]> {
    const response = await host.get(host.createUrl('assets', filter));
    return (await response.json()) as AssetInfo[];
  }

  /** Parameters for importing an asset file. */
  export interface ImportAssetParams {
    /** Absolute path to the source file */
    sourcePath: string;
    /** Asset ID to register (e.g., "vrm:local:my-persona") */
    assetId: string;
    /** Asset type */
    assetType: AssetType;
    /** Optional description */
    description?: string;
  }

  /** Result of an asset import. */
  export interface ImportAssetResult {
    /** The registered asset ID */
    assetId: string;
  }

  /**
   * Imports a file as a managed asset.
   *
   * Copies the source file to the managed assets directory, registers it in the
   * asset registry, and persists the registration to the database.
   *
   * @returns The registered asset info
   *
   * @example
   * ```typescript
   * const result = await assets.importAsset({
   *   sourcePath: "/Users/me/Downloads/model.vrm",
   *   assetId: "vrm:local:alice",
   *   assetType: "vrm",
   *   description: "Alice's custom model",
   * });
   * console.log("Imported:", result.assetId);
   * ```
   */
  export async function importAsset(params: ImportAssetParams): Promise<ImportAssetResult> {
    const response = await host.post(host.createUrl('assets/import'), params);
    return (await response.json()) as ImportAssetResult;
  }

  /**
   * Fetches the raw file content of an asset by ID.
   *
   * The returned `Blob` carries both the byte payload and the MIME type
   * (inferred server-side from the file extension). Unknown extensions
   * fall back to `application/octet-stream`.
   *
   * @param id - Asset ID (e.g. `"@hmcs/elmer:thumbnail"`)
   * @returns A Blob containing the asset bytes. `blob.type` holds the MIME type.
   * @throws {HomunculusApiError} 400 if the id is missing, 404 if the asset
   *         or underlying file is missing, 500 on IO error.
   *
   * @example
   * ```typescript
   * const blob = await assets.fetchFile("@hmcs/elmer:thumbnail");
   * console.log(blob.type);  // "image/png"
   * console.log(blob.size);  // e.g. 12345
   *
   * // Read as bytes
   * const bytes = new Uint8Array(await blob.arrayBuffer());
   *
   * // Or use directly as an <img src> in a browser/CEF context
   * const url = URL.createObjectURL(blob);
   * ```
   */
  export async function fetchFile(id: string): Promise<Blob> {
    const response = await host.get(host.createUrl('assets/file', { id }));
    return await response.blob();
  }
}
