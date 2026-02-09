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

## Fork Structure

Our `termsurf-chromium` submodule (at `ts4/termsurf-chromium/src/`) is a
Chromium fork with a linear commit history:

```
Chromium 146.0.7650.0 (vanilla)
  + Electron's 147 patches (applied as commits)
  ← electron-base tag (marks where Electron patches end)
  + TermSurf's commits (content/two_profiles/, BUILD.gn, etc.)
  ← main branch HEAD
```

Electron does not maintain a Chromium fork — it applies patches at build time.
We take a different approach: the patches become permanent commits in our fork.
TermSurf's own modifications are regular commits on top.

### The `electron-base` tag

A tag called `electron-base` marks the boundary between Electron's patches and
TermSurf's commits. This serves two purposes:

1. **Visibility.** `git log electron-base..HEAD` shows exactly what TermSurf
   has changed on top of Electron's patches.
2. **Rebase target.** When the Electron patch set updates, TermSurf's commits
   are rebased onto the new `electron-base`.

### Rebase workflow

TermSurf's modifications are maintained as regular commits — not a separate
patch set. When Electron updates (new Chromium version or new patches), we
rebase our commits on top:

```
1. Check out vanilla Chromium at Electron's new version
2. Apply the updated Electron patch set → new electron-base
3. Rebase TermSurf's commits onto the new electron-base
4. Rebuild and test
```

This works cleanly because TermSurf's changes don't overlap with Electron's
patches — our files (`content/two_profiles/`) are entirely new, and our
`BUILD.gn` change is a single line in a section Electron doesn't touch.
Rebasing should be conflict-free or nearly so.

If a conflict does arise, we resolve it during the rebase. This is the same
workflow any fork uses to stay current with upstream.

## Steps

### Phase 1: Clean slate

Delete the Two Profiles app from the fork. It was built before the decision to
use the Electron patch set and will be rewritten from scratch using the new
APIs. This returns the fork to a clean vanilla Chromium state.

- [ ] Delete `content/two_profiles/` directory
- [ ] Revert the `//content/two_profiles` line in `BUILD.gn`
- [ ] Commit the deletion

### Phase 2: Match Chromium version

Our fork was created with `fetch chromium`, which pulled whatever was HEAD at
that time. Electron targets Chromium 146.0.7650.0. We need to verify our fork
is on the same version, and check out the correct tag if it isn't.

```bash
cd ts4/termsurf-chromium/src
git log --oneline -1  # What version are we on?
git tag -l '146.0.7650.0'  # Does this tag exist?
```

If the version doesn't match, check out the correct one:

```bash
git checkout 146.0.7650.0
```

If the tag doesn't exist, we may need to fetch it from upstream or use the
commit hash that corresponds to Electron's pinned version.

### Phase 3: Apply the Electron patch set

Apply all 147 patches in order from the Electron repo:

```bash
cd ts4/termsurf-chromium/src

while IFS= read -r patch; do
  git am --3way "../../electron/patches/chromium/$patch" || {
    echo "FAILED: $patch"
    break
  }
done < ../../electron/patches/chromium/.patches
```

If a patch fails to apply, our Chromium version doesn't match Electron's.
Fix by checking out the correct version in Phase 2.

After all patches apply, tag the boundary:

```bash
git tag electron-base HEAD
```

### Phase 4: Verify Content Shell (baseline)

Content Shell is our baseline — it must still build and run at 60fps after the
Electron patches. If Content Shell breaks, the patches are the problem, not
our code.

```bash
gn gen out/Default --args='is_debug=false symbol_level=0 is_component_build=true'
autoninja -C out/Default content_shell
```

Launch Content Shell with the test page:

```bash
cd /Users/ryan/dev/termsurf/ts4/box-demo && bun run server.ts &
./out/Default/Content\ Shell.app/Contents/MacOS/Content\ Shell http://localhost:9407
```

Verify: spinning blue square at 60fps, localStorage string persists across
restarts. This confirms the Electron patches don't break vanilla Chromium
windowed rendering.

### Phase 5: Rewrite Two Profiles

Rebuild the Two Profiles app from scratch, this time using the APIs that the
Electron patches provide:

- Create `content/two_profiles/` with the same macOS bundle structure as before
- Use the three-layer throttling bypass on each WebContents:
  ```cpp
  rwh_impl->disable_hidden_ = true;                                    // Layer 1
  web_contents->GetRenderViewHost()->SetSchedulerThrottling(false);    // Layer 2
  // Layer 3 handled by compositor patch automatically
  ```
- Consider using Chromium's `views` framework (`views::Widget` +
  `views::WebView`) for view composition instead of raw NSView manipulation
- Register the target in `BUILD.gn`

### Phase 6: Verify Two Profiles at 60fps

Launch the Two Profiles app with the test server. Both panes should render the
spinning blue square at 60fps with different localStorage identity strings.

```bash
autoninja -C out/Default content/two_profiles:two_profiles
./out/Default/Two\ Profiles.app/Contents/MacOS/Two\ Profiles
```

## Success Criteria

- [ ] Two Profiles deleted from the fork (clean slate).
- [ ] Fork checked out at Chromium 146.0.7650.0.
- [ ] All 147 Electron patches apply cleanly.
- [ ] `electron-base` tag marks the boundary.
- [ ] Content Shell builds and runs at 60fps (baseline holds).
- [ ] Two Profiles rewritten with throttling bypass APIs.
- [ ] Both panes render at 60fps (up from 2-3fps).
- [ ] Profile isolation still works (different localStorage strings).

## Future: Staying in Sync with Electron

When Electron bumps its Chromium version:

```bash
# 1. Fetch the new vanilla Chromium version
git fetch upstream
git checkout <new-chromium-version>

# 2. Apply the updated Electron patch set
while IFS= read -r patch; do
  git am --3way "../../electron/patches/chromium/$patch"
done < ../../electron/patches/chromium/.patches

# 3. Move the electron-base tag
git tag -f electron-base HEAD

# 4. Rebase TermSurf's commits
git rebase electron-base main

# 5. Rebuild and test
autoninja -C out/Default content/two_profiles:two_profiles
```

This keeps us on a well-tested Chromium version with well-tested patches. We
never independently track Chromium releases — we follow Electron's lead.

## Relationship to Other Issues

| Issue | Relationship                                                      |
| ----- | ----------------------------------------------------------------- |
| 407   | Proved multi-profile works; identified 2-3fps throttling          |
| 408   | Traced throttling to three layers; discovered Electron's solution |
| 409   | This issue — applies the patch set to our fork                    |
