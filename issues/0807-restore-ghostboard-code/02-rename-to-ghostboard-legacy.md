# Experiment 2: Rename to Ghostboard Legacy

## Description

Rename the restored historical Ghostboard source directory from `ghostboard/` to
`ghostboard-legacy/`. The goal is only to make the restored directory's name
reflect that this is archived legacy code, not an active GUI implementation.

No adversarial review was performed for this experiment because the user
explicitly requested no adversarial review.

## Changes

- Renamed `ghostboard/` to `ghostboard-legacy/` with:

  ```bash
  git mv ghostboard ghostboard-legacy
  ```

- Updated the Issue 807 README experiment index and conclusion to record the new
  directory name.
- Did not build, run, modernize, fix, reformat, or integrate the restored
  Ghostboard source.

## Verification

Pass criteria:

- `ghostboard-legacy/` exists in the working tree.
- `ghostboard/` no longer exists in the working tree.
- Representative expected files still exist at the renamed path:
  - `ghostboard-legacy/build.zig`;
  - `ghostboard-legacy/src/Surface.zig`;
  - `ghostboard-legacy/src/apprt/embedded.zig`;
  - `ghostboard-legacy/macos/`;
  - `ghostboard-legacy/include/termsurf.h`.
- The file count under `ghostboard-legacy/` remains `1536`.
- `git status --short` reports the change as a path rename from `ghostboard/` to
  `ghostboard-legacy/`, aside from issue documentation updates.

## Result

**Result:** Pass

Renamed the restored directory with:

```bash
git mv ghostboard ghostboard-legacy
```

Verification completed:

- Confirmed `ghostboard-legacy/` exists.
- Confirmed `ghostboard/` no longer exists.
- Confirmed representative files exist at the renamed path:
  - `ghostboard-legacy/build.zig`;
  - `ghostboard-legacy/src/Surface.zig`;
  - `ghostboard-legacy/src/apprt/embedded.zig`;
  - `ghostboard-legacy/macos/`;
  - `ghostboard-legacy/include/termsurf.h`.
- Confirmed `find ghostboard-legacy -type f | wc -l` returns `1536`.

No build or run attempt was made, by design.

## Conclusion

The restored historical Ghostboard source now lives at `ghostboard-legacy/`.
This experiment only renamed the restored directory and updated the issue log;
it did not change the source content or attempt to make Ghostboard Legacy work.
