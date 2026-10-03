# Releasing the crate

## Release contract

Use `Cargo.toml` as the version source. Tag the corresponding commit `vVERSION`,
for example `v0.1.0`. The publishing script reads the tag and requires it to
match the manifest; it never rewrites a version or makes a commit.

The same commit must be tested, pushed to the public repository and published
to crates.io. A Git tag and a crates.io release are separate actions: pushing
a tag alone does not publish the crate.

This follows Cargo's recommendation to version the manifest, keep a changelog
and tag the published commit. [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html).

## Before the first release

- Resolve the upstream specification/derived-code licence question described
  in the README. This applies to the contents of the public repository too.
- Create the public repository and configure its URL as Git remote `origin`.
  Add its public URL as `package.repository` in `Cargo.toml`.
- Confirm the crate name is available and configure crates.io authentication
  with `cargo login` or Cargo's supported credential provider.
- Change `publish = false` to `publish = ["crates-io"]` once the crate is ready
  for publication. The script respects this restriction, including in dry runs.
- Install the toolchains and tools declared in `mise.toml`, plus Python 3 for
  parsing Cargo's JSON metadata in the Bash script.

No remote repository or crates.io release has been created by adding this workflow.

## Prepare a version

1. Update `package.version` in `Cargo.toml` using SemVer. Update `Cargo.lock`
   by running Cargo after the change, and describe the changes in `CHANGELOG.md`.
2. Review generated-code changes and run `mise run check`.
3. Inspect `cargo package --list`. The crate should contain its generated Rust
   code and build without downloading the specification or running the generator.
4. Commit the complete release, including the lockfile and generated output.
5. Create an annotated tag matching the manifest, then push the commit and tag.

For example, after preparing version 0.1.0:

```sh
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin HEAD
git push origin v0.1.0
```

Use an explicit branch destination if publishing from a detached checkout.

## Verify and publish

From the project root (or a subdirectory):

```sh
mise run release          # validate and dry-run; detect the tag at HEAD
mise run release:publish  # validate, dry-run, then upload
```

Both tasks run from the directory containing `mise.toml`, using `config_root`.
With no tag argument, the script detects the unique `v*` tag pointing at HEAD,
such as `v0.1.0`, then verifies it against `Cargo.toml`. An absent or ambiguous
tag is an error. It does not select a tag from an older commit.

To choose explicitly when HEAD has more than one version tag:

```sh
mise run release v0.1.0
mise run release:publish v0.1.0
```

The underlying `scripts/publish.sh [vVERSION] [--publish]` remains available.
Task working directories and argument forwarding follow
[mise's task conventions](https://mise.jdx.dev/tasks/toml-tasks.html).

The default command performs a dry run. The script requires a clean checkout,
checks that the tag points to HEAD, checks the tag against the manifest version,
and verifies the identical tag exists on `origin`. It runs `mise run check`
(release-script tests, formatting, Clippy, tests and generation freshness), followed by
`cargo publish --dry-run --locked --registry crates-io`.

`release:publish` repeats those gates, then uploads with Cargo. It never commits,
tags, pushes, edits the manifest or passes a token on the command line.
Cargo uses its configured credentials. Both modes read `origin` and may access
package registries; only `release:publish` (or the script's `--publish`) uploads a crate.

Cargo's dry run packages the source and checks that the unpacked crate builds.
It does not prove upload authorization or reserve a crate name/version.
[Cargo publish reference](https://doc.rust-lang.org/cargo/commands/cargo-publish.html).

## After publication or failure

Confirm the expected version appears on crates.io. Keep its Git tag unchanged.
Registry versions cannot be overwritten; corrections require a new version.
If Cargo reports an upload/index timeout, check crates.io before retrying:
the upload may already have succeeded. If the upload did not happen and the
tagged code is unchanged, rerun the script against the same tag.

If code must change after the tag was pushed, prepare a new version/tag.
Use yanking for an unsuitable published version when appropriate; it is not
a replacement for publishing a corrected version.

For this single crate, a short script is sufficient. If releases later need
automated version bumps, coordinated workspace releases or release PRs, evaluate
the tools linked by Cargo, such as `cargo-release` or `release-plz`, before
expanding the script into a release framework.

## Maintaining the script

Run `bash -n scripts/publish.sh` and `python3 scripts/test-publish.py` after
changes. The tests use disposable local Git repositories and replace Cargo and
mise with stubs, so they verify release gates without making registry requests
or uploading anything.
