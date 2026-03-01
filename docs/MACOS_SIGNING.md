# macOS Code Signing & Notarization

CORE-FFX uses Tauri v2's built-in macOS signing and notarization support. Without these, downloaded `.app` bundles trigger **"CORE-FFX.app is damaged and can't be opened"** from macOS Gatekeeper.

## Prerequisites

- **Apple Developer Program** membership ($99/year) — [developer.apple.com](https://developer.apple.com/programs/)
- A **Developer ID Application** certificate (not Mac App Store)
- An **App Store Connect API key** (for notarization)

## 1. Create the Developer ID Certificate

1. Open **Keychain Access** → Certificate Assistant → Request a Certificate from a Certificate Authority
2. Go to [developer.apple.com/account/resources/certificates](https://developer.apple.com/account/resources/certificates)
3. Click **+** → **Developer ID Application** → upload your CSR
4. Download and install the `.cer` file (double-click to add to Keychain)
5. Export as `.p12`:
   ```bash
   # In Keychain Access: right-click the certificate → Export
   # Choose .p12 format, set a password
   ```
6. Base64-encode the `.p12`:
   ```bash
   base64 -i Certificates.p12 | pbcopy
   ```

## 2. Create the App Store Connect API Key

1. Go to [appstoreconnect.apple.com/access/integrations/api](https://appstoreconnect.apple.com/access/integrations/api)
2. Click **Generate API Key**
3. Name: `CORE-FFX Notarization`, Access: **Developer**
4. Download the `.p8` file (only available once!)
5. Note the **Key ID** and **Issuer ID** shown on the page

## 3. Configure GitHub Secrets

Go to your repo → **Settings** → **Secrets and variables** → **Actions** → **New repository secret**:

| Secret Name | Value | Description |
|---|---|---|
| `APPLE_CERTIFICATE` | Base64 of `.p12` file | `base64 -i Certificates.p12 \| pbcopy` |
| `APPLE_CERTIFICATE_PASSWORD` | Password set during `.p12` export | Plain text |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Name (TEAMID)` | Exact identity string from Keychain |
| `APPLE_API_ISSUER` | UUID from App Store Connect API page | e.g., `69a6de7e-...` |
| `APPLE_API_KEY` | Key ID from App Store Connect API page | e.g., `2X9R4HXF34` |
| `APPLE_API_KEY_CONTENT` | Full contents of the `.p8` file | Paste entire file including `-----BEGIN PRIVATE KEY-----` |

### Finding Your Signing Identity

```bash
security find-identity -v -p codesigning
```
Look for: `"Developer ID Application: Your Name (XXXXXXXXXX)"`

## 4. How It Works

When these secrets are configured, the release workflow automatically:

1. **Writes** the `.p8` API key to a temp file
2. **Imports** the `.p12` certificate to a temporary keychain (Tauri handles this)
3. **Signs** the `.app` with `codesign --force --options runtime` (hardened runtime)
4. **Notarizes** via `notarytool submit` using the API key
5. **Staples** the notarization ticket via `stapler staple`

The resulting `.app` and `.dmg` will pass Gatekeeper on any Mac without warnings.

## 5. Entitlements

The entitlements file at `src-tauri/Entitlements.plist` grants:

| Entitlement | Purpose |
|---|---|
| `com.apple.security.cs.allow-jit` | WKWebView JIT compilation |
| `com.apple.security.cs.allow-unsigned-executable-memory` | WKWebView runtime |
| `com.apple.security.cs.disable-library-validation` | WebView plugins / dylibs |
| `com.apple.security.network.client` | Updater, AI assistant API calls |
| `com.apple.security.files.user-selected.read-write` | File dialog access |
| `com.apple.security.files.downloads.read-only` | Evidence scanning |

## 6. Without Signing (Workaround)

If Apple Developer credentials are not configured, users can bypass Gatekeeper manually:

```bash
# Remove quarantine flag from downloaded .app
xattr -cr /Applications/CORE-FFX.app
```

Or: **System Settings** → **Privacy & Security** → scroll to "CORE-FFX.app was blocked" → **Open Anyway**.

## 7. Verifying a Signed Build

```bash
# Check code signature
codesign -dv --verbose=4 /Applications/CORE-FFX.app

# Verify notarization
spctl --assess --type exec --verbose /Applications/CORE-FFX.app

# Check stapled ticket
stapler validate /Applications/CORE-FFX.app
```

## Troubleshooting

| Error | Cause | Fix |
|---|---|---|
| "damaged and can't be opened" | No code signing | Configure `APPLE_CERTIFICATE` + `APPLE_SIGNING_IDENTITY` secrets |
| "not notarized" / "unidentified developer" | Signed but not notarized | Configure `APPLE_API_*` secrets |
| "certificate not found" | Wrong signing identity string | Run `security find-identity -v -p codesigning` on the CI runner |
| Notarization timeout | Large app / Apple server slow | Retry the workflow; notarization can take 5-15 min |
| "invalid signature" after extract | `.tar.gz` extraction broke signature | Use the `.dmg` instead |
