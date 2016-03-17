# `before_deploy` phase: here we package the build artifacts

set -ex

# Generate artifacts for release
cargo build --target $TARGET --release

# create a "staging" directory
mkdir staging

# NOTE All Cargo build artifacts will be under the 'target/$TARGET/{debug,release}'
cp target/$TARGET/release/ilc* staging

cd staging

# release tarball will look like 'ilc-unknown-linux-gnu.tar.gz'
tar czf ../${PROJECT_NAME}-${TARGET}.tar.gz *
