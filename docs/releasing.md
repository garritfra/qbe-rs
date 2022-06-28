# Release Workflow

1. **Optional**: Use the
   [cargo-public-api](https://crates.io/crates/cargo-public-api) crate to spot
   possible breaking changes
1. Update version in `Cargo.toml`
1. Add entry in `CHANGELOG.md`
1. Commit change with semantic version number (e.g.: `v0.1.1`)
1. Tag commit using `git tag -a <new release> -m "$(git shortlog <last release>..HEAD)"`
1. Push the tag using `git push --tags`
1. Publish package using `cargo publish`
1. Create new GitHub Release
