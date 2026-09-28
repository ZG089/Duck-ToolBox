/**
 * Contract between the development mock host (`src/dev/mock-host.ts`) and each feature's
 * optional `mock.ts`. Only development and E2E builds load any of this.
 */

export interface MockPackage {
  packageName: string
  label: string
  system: boolean
}

/** Receives the remaining positional arguments, the flags and the stdin JSON. */
export type MockHandler = (
  args: string[],
  flags: Record<string, string | true>,
  input: unknown,
) => unknown

export interface MockModule {
  /** Keyed by dotted command, e.g. `tricky-store.keybox.set-aosp`. */
  handlers: Record<string, MockHandler>
  /** Backend feature id and contract this mock stands in for. */
  feature?: { id: string; contract: number }
}

/** A failure the mock reports as an error envelope, like a real `duckd` error code. */
export class MockFailure extends Error {
  readonly code: string
  readonly details: unknown

  constructor(code: string, message: string, details?: unknown) {
    super(message)
    this.code = code
    this.details = details
  }
}

/** Installed apps shared by every mock, so package lists agree across features. */
export const MOCK_PACKAGES: MockPackage[] = [
  { packageName: "io.github.vvb2060.keyattestation", label: "Key Attestation", system: false },
  { packageName: "io.github.vvb2060.mahoshojo", label: "Mahoshojo", system: false },
  { packageName: "com.google.android.apps.walletnfcrel", label: "Google Wallet", system: false },
  { packageName: "com.paypal.android.p2pmobile", label: "PayPal", system: false },
  { packageName: "com.revolut.revolut", label: "Revolut", system: false },
  { packageName: "com.netflix.mediaclient", label: "Netflix", system: false },
  { packageName: "com.spotify.music", label: "Spotify", system: false },
  { packageName: "org.telegram.messenger", label: "Telegram", system: false },
  { packageName: "com.whatsapp", label: "WhatsApp", system: false },
  { packageName: "com.discord", label: "Discord", system: false },
  { packageName: "me.weishu.kernelsu", label: "KernelSU", system: false },
  { packageName: "org.lsposed.manager", label: "LSPosed", system: false },
  { packageName: "com.example.xposed.hider", label: "Hide My Applist", system: false },
  { packageName: "com.tencent.mm", label: "微信", system: false },
  { packageName: "com.eg.android.AlipayGphone", label: "支付宝", system: false },
  { packageName: "com.google.android.gms", label: "Google Play services", system: true },
  { packageName: "com.android.vending", label: "Google Play Store", system: true },
  { packageName: "com.google.android.gsf", label: "Google Services Framework", system: true },
  { packageName: "com.android.chrome", label: "Chrome", system: true },
  { packageName: "com.oplus.deepthinker", label: "Deep Thinker", system: true },
  { packageName: "com.android.settings", label: "Settings", system: true },
]
