# WebKit Patch Archives

This directory stores TermSurf WebKit patch sets generated from `webkit/src`.

Each issue that modifies WebKit source should get a subdirectory:

```text
webkit/patches/issue-{N}/
```

Generate patches from the recorded upstream base commit to the issue branch tip:

```bash
rm -rf webkit/patches/issue-{N}
mkdir -p webkit/patches/issue-{N}
git -C webkit/src format-patch {base-commit}..HEAD \
  -o ../../webkit/patches/issue-{N}
```

Apply patches from a fresh checkout with:

```bash
git -C webkit/src switch -C webkit-{short-base}-issue-{N} {base-commit}
git -C webkit/src am ../../webkit/patches/issue-{N}/*.patch
```

Issue 756 has not archived WebKit source patches yet; Experiments 1-4 are
bootstrap, compositor-proof, lifecycle-proof, and workflow setup only.
