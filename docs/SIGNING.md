# Keys and signing — the two things only the owner can do

Written 2026-09-12. Until these are done, builds are for the owner and
testers. Nothing here is needed for testers; all of it is needed before the
download link goes on the landing page.

---

## 1. Back up the updater key (do this now — five minutes)

This key signs every release. Installed apps refuse an update that is not
signed with it. **If it is lost, every installed app is orphaned**: it can
never update again and each user has to reinstall by hand. GitHub holds a copy
as a secret, but secrets cannot be read back — so a copy you can read must
exist somewhere other than this PC.

1. Open **Notepad**. File → Open, paste this path in the filename box:
   `C:\Users\Giof1\.tauri\kronos-desktop.key` → Open.
   (Set the file type dropdown to *All files* if it does not appear.)
2. Ctrl+A, Ctrl+C. It is two lines: an `untrusted comment:` line and a long
   line of letters and digits. Copy both.
3. Open your password manager (iCloud Passwords / whatever you use). Create a
   **secure note**, not a login. Title: `KRONOS desktop updater key`.
   Paste. Add one line of context: *Tauri updater private key for
   Commongio/kronos-desktop. No password. Losing this orphans every install.*
4. Do the same for the public key, `kronos-desktop.key.pub`, in the same note.
   It is already in `tauri.conf.json`, so this is convenience, not safety.
5. Close Notepad **without saving**.

Never paste the private key into a chat, an issue, a commit or a screenshot.
If that ever happens: generate a new pair (`npx tauri signer generate`), put
the new public key in `tauri.conf.json`, update the `TAURI_SIGNING_PRIVATE_KEY`
secret, and ship one release signed with the *old* key that carries the new
public key — installed apps then trust the new one. Do that before any
install has been made with a leaked key, and nothing is lost.

---

## 2. Code signing (before public use — a few hours, plus waiting)

Two different systems, two different vendors, no overlap.

### What unsigned looks like to a tester today

- **Windows** — SmartScreen: *"Windows protected your PC"*. They click
  **More info → Run anyway**. If their PC has **Smart App Control** on
  (Windows 11 default on new machines), it refuses with no override.
- **macOS** — *"KRONOS is damaged and can't be opened"* or *"unidentified
  developer"*. They **right-click the app → Open**, then confirm. Some macOS
  versions need **System Settings → Privacy & Security → Open Anyway**.

Tell testers this up front, in the message with the link. It is normal for
an early build; it is not acceptable for a customer.

### macOS — Apple Developer Program, $99/year

1. **Enrol** at developer.apple.com/programs using your Apple ID. Individual
   enrolment is fine. Takes 1–2 days to approve.
2. **Create a Developer ID Application certificate.** On a Mac (this needs
   Keychain Access — you or a tester with a Mac):
   Keychain Access → Certificate Assistant → *Request a Certificate From a
   Certificate Authority* → save to disk. Then at
   developer.apple.com → Certificates → **+** → *Developer ID Application*
   → upload the request → download the `.cer` → double-click to install.
3. **Export it** as a `.p12`: Keychain Access → My Certificates → right-click
   the Developer ID cert → Export → set a password. Then base64 it:
   `base64 -i cert.p12 | pbcopy`.
4. **App-specific password** for notarization: appleid.apple.com → Sign-In
   and Security → App-Specific Passwords → generate one.
5. **Team ID**: developer.apple.com → Membership details.
6. **Add six GitHub secrets** (`gh secret set NAME` or Settings → Secrets):

   | Secret | Value |
   |---|---|
   | `APPLE_CERTIFICATE` | the base64 from step 3 |
   | `APPLE_CERTIFICATE_PASSWORD` | the `.p12` password |
   | `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Name (TEAMID)` |
   | `APPLE_ID` | your Apple ID email |
   | `APPLE_PASSWORD` | the app-specific password from step 4 |
   | `APPLE_TEAM_ID` | from step 5 |

7. **Workflow**: pass those six as `env:` on the `tauri-apps/tauri-action`
   step, next to `TAURI_SIGNING_PRIVATE_KEY`. Tauri signs and notarizes
   automatically when they are present. Nothing in `tauri.conf.json` changes.
8. Tag a release. Gatekeeper opens the `.dmg` with no warning.

### Windows — Azure Trusted Signing, about $10/month

Cheapest route that gets a real, SmartScreen-trusted signature. (An
OV/EV certificate from DigiCert or Sectigo also works — $200–400/year and a
hardware token — but Trusted Signing is the modern path.)

1. **Azure account** at portal.azure.com (free to create; the signing
   service is the ~$10/month "Basic" tier).
2. **Identity validation**: create a *Trusted Signing account* → *Identity
   validation* → **Individual**. Government ID and a few days' wait.
3. **Certificate profile**: in the account, *Certificate profiles* → **+** →
   *Public Trust* → name it `kronos-desktop`.
4. **Service principal** so CI can sign: Azure → App registrations → New →
   note the **Tenant ID**, **Client ID**; Certificates & secrets → New client
   secret → note the **Client secret**. Then give that app the *Trusted
   Signing Certificate Profile Signer* role on the signing account (Access
   control (IAM) → Add role assignment).
5. **Three GitHub secrets**: `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`,
   `AZURE_CLIENT_SECRET`.
6. **Workflow**: before the tauri-action step on `windows-latest`, install the
   signing tool and set `signCommand`:

   ```yaml
   - name: Trusted Signing
     if: matrix.os == 'windows-latest'
     run: |
       dotnet tool install --global --version 0.2.4 Microsoft.Trusted.Signing.Client
       echo "TAURI_WINDOWS_SIGNTOOL_ARGS=..." >> $env:GITHUB_ENV
   ```

   Tauri's `bundle.windows.signCommand` takes a command with `%1` for the file;
   the exact invocation is in Tauri's docs under *Windows Code Signing → Azure
   Code Signing*. Copy it from there when you do this — the tool's flags have
   changed between versions and a stale copy here would be worse than none.
7. Tag a release. SmartScreen shows the publisher name. **Smart App Control
   starts allowing it** once the signature has some install history — this is
   not instant, but it is the only route to it at all.

### Order to do them in

Apple first if you have to pick one: the Mac failure mode is worse (no
"Run anyway" on newer macOS without digging into System Settings), and the
Apple side needs no identity-validation wait beyond enrolment.

### What does NOT change

The updater key from section 1 stays exactly as it is. Code signing proves
*who published the binary* to the OS; the updater key proves *this update
came from the same KRONOS* to installed apps. Different question, different
key, and neither replaces the other.
