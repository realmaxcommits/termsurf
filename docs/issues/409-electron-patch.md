# Issue 409: Apply Electron's Chromium Patch Set

## Goal

Apply Electron's full 147-patch set to our `termsurf-chromium` submodule so that
Two Profiles (and future TermSurf browser panes) render at 60fps. Track the same
Chromium version as Electron (146.0.7650.0) and apply the exact same patches
with no modifications.

## Background

Issue 407 proved that multiple `BrowserContext` instances coexist in one
Chromium process with full profile isolation, but rendering was throttled to
2-3fps. Issue 408 traced the problem to three independent throttling systems in
Chromium's rendering pipeline and discovered that Electron solves this with a
well-tested set of patches. Rather than cherry-picking a subset, we adopt the
full patch set — it's simpler, tested, and future-proof.

## Approach

Our `termsurf-chromium` submodule (at `ts4/termsurf-chromium/src/`) is a
Chromium fork. The layering is:

```
Chromium 146.0.7650.0 (vanilla)
  + Electron's 147 patches (from electron/patches/chromium/)
  + TermSurf's modifications (content/two_profiles/, BUILD.gn change)
```

Electron does not maintain a Chromium fork. It applies patches at build time
using `git am` or equivalent. We take a different approach — we apply the
patches as permanent commits in our fork. This means our fork's git history
contains the vanilla Chromium history, then the Electron patches as commits,
then our own modifications on top.

## Steps

### Step 1: Verify Chromium version

Check what version our fork is currently on and whether it matches Electron's
target (146.0.7650.0). If not, we need to check out the matching version first.

```bash
cd ts4/termsurf-chromium/src
git log --oneline -1  # Check current HEAD
```

Electron's DEPS file pins `chromium_version: '146.0.7650.0'`. Our fork must be
on the same version for the patches to apply cleanly.

### Step 2: Save our modifications

Our fork currently has modifications from Issue 407:

- `content/two_profiles/` directory (new files)
- `BUILD.gn` (one line added to `gn_all` group)

These need to be preserved and reapplied after the Electron patches. Options:

- Create a branch with our modifications, rebase after patching
- Export our changes as a patch, reapply after Electron's patches
- Simply re-commit them on top (they don't touch any files Electron patches)

### Step 3: Apply the patch set

The patches are at `electron/patches/chromium/` and the ordered list is in
`electron/patches/chromium/.patches`. Apply them in order:

```bash
cd ts4/termsurf-chromium/src

# Read the patch list and apply each one
while IFS= read -r patch; do
  git am --3way "../../electron/patches/chromium/$patch" || {
    echo "FAILED: $patch"
    break
  }
done < ../../electron/patches/chromium/.patches
```

If a patch fails to apply, it means our Chromium version doesn't match
Electron's expected version. Fix by checking out the correct version first.

### Step 4: Reapply TermSurf modifications

Re-commit our modifications on top of the Electron patches:

- Re-add `content/two_profiles/` directory
- Re-add the `//content/two_profiles` line to `BUILD.gn`

### Step 5: Rebuild and test

```bash
gn gen out/Default --args='is_debug=false symbol_level=0 is_component_build=true'
autoninja -C out/Default content/two_profiles:two_profiles
```

### Step 6: Wire up the throttling bypass

With the Electron patches applied, the Three throttling bypass APIs are now
available. Modify `two_profiles_main_parts.mm` to use them:

```cpp
// After creating each WebContents:
auto* rwh = RenderWidgetHostImpl::From(
    web_contents->GetRenderWidgetHostView()->GetRenderWidgetHost());
rwh->disable_hidden_ = true;  // Layer 1: prevent WasHidden()

// Layer 2: disable Blink scheduler throttling
web_contents->GetRenderViewHost()->SetSchedulerThrottling(false);

// Layer 3 is handled by the compositor patch automatically
```

### Step 7: Verify 60fps

Launch the Two Profiles app with the Bun test server running. Both panes should
now render the spinning blue square at 60fps with different localStorage
identity strings.

## Success Criteria

- All 147 Electron patches apply cleanly to our Chromium fork.
- The Two Profiles app builds and runs.
- Both panes render at 60fps (up from 2-3fps).
- Profile isolation still works (different localStorage strings).
- content_shell still builds and runs independently.

## Future: Staying in Sync with Electron

When Electron bumps its Chromium version, we can follow:

1. Check out the new vanilla Chromium version in our fork
2. Apply the updated patch set from the new Electron version
3. Reapply TermSurf's modifications on top
4. Rebuild and test

This keeps us on a well-tested Chromium version with well-tested patches,
without having to independently track Chromium releases.

## Relationship to Other Issues

| Issue | Relationship                                                      |
| ----- | ----------------------------------------------------------------- |
| 407   | Proved multi-profile works; identified 2-3fps throttling          |
| 408   | Traced throttling to three layers; discovered Electron's solution |
| 409   | This issue — applies the patch set to our fork                    |
