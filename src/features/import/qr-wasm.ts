import { prepareZXingModule, type ZXingModuleOverrides } from "zxing-wasm/reader";
import zxingWasmUrl from "zxing-wasm/reader/zxing_reader.wasm?url";

let isConfigured = false;

/**
 * By default the library downloads its WebAssembly file from a public CDN. That
 * would send a request out of the app and fail offline, so the file is bundled
 * with the app and loaded from there. Tests pass their own overrides.
 */
export function configureQrDecoder(overrides: ZXingModuleOverrides): void {
  prepareZXingModule({ overrides });
  isConfigured = true;
}

export function ensureQrDecoderConfigured(): void {
  if (isConfigured) return;
  configureQrDecoder({
    locateFile: (path: string, prefix: string) => (path.endsWith(".wasm") ? zxingWasmUrl : prefix + path),
  });
}
