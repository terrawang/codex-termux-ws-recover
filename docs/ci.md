# Continuous integration

This fork focuses automatic CI on the Termux Android arm64 package and the
checks needed to review its changes.

## Automatic checks

- **blocking-ci** (pull requests and main): codespell, blob size, package and
  repository checks, Rust formatting, cargo shear, and cargo-deny. The aggregate
  `CI required` check requires these checks to succeed. SDK builds and tests are
  excluded from this workflow.
- **WebSocket recovery** (pull requests): client state and completion tests,
  fallback integration tests, and Termux launcher tests. Linux binaries are
  compiled only to execute these tests; they are not release artifacts.
- **termux-npm-build-publish** (pushes to main and lts): Android arm64 release
  binaries, the pinned Android V8 archive sandbox check, the Termux patch
  contract, and npm package validation. The installable tarball and its SHA-256
  checksum are uploaded as GitHub Actions artifacts. This workflow never
  publishes to npm.

## Building an Android artifact for a PR

Run **termux-npm-build-publish** manually and select the PR branch as the ref:

```sh
gh workflow run termux-npm-build-publish.yml --ref YOUR_PR_BRANCH
```

PR updates do not automatically start another Android release build. Dispatch
this builder again after changing Rust sources, dependencies, package contents,
or build scripts. Documentation and CI policy updates can reuse an artifact
built from the same source tree; record its actual build commit with the link.

## Optional broader checks

- **v8-canary** no longer runs on pull requests. Its Linux V8 matrix remains
  available through `workflow_dispatch` and `workflow_call` when deliberately
  validating a V8 update. Android builds consume the pinned fork-owned Android
  archive instead of building these non-Android variants.
- **postmerge-ci** no longer runs automatically on main. It can be dispatched
  manually to run the broader Rust and V8 suites.
- **termux-gate** remains a manual workflow for full workspace checks. It is not
  needed for every Android package build.
- **SDK checks** remain in their reusable workflow for deliberate use, outside
  the automatic merge gate.

## Inherited platform limitations

The Bazel jobs in `bazel.yml` and the Bazel prebuilt argument-comment lint job in
`rust-ci.yml` remain disabled with `if: false`. They require upstream cache or
runner access. `CI required` does not depend on Bazel; `CI results` does not
require the disabled prebuilt lint job. Cargo checks remain strict.

The inherited private runner jobs in `rust-ci-full.yml` and paid macOS or
unavailable Windows matrix legs remain disabled. They can be restored if the
required runners become available. Disabling a matrix entry requires removing
it from `include`; an `exclude` entry cannot remove a later `include` entry.
