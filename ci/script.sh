# `script` phase: you usually build, test and generate docs in this phase

set -ex

# PROTIP Always pass `--target $TARGET` to cargo commands, this makes cargo output build artifacts
# to target/$TARGET/{debug,release} which can reduce the number of needed conditionals in the
# `before_deploy`/packaging phase

case "$TRAVIS_OS_NAME" in
  linux)
    host=x86_64-unknown-linux-gnu
    ;;
  osx)
    host=x86_64-apple-darwin
    ;;
esac

# NOTE Workaround for rust-lang/rust#31907 - disable doc tests when cross compiling
# This has been fixed in the nightly channel but it would take a while to reach the other channels
if [ "$host" != "$TARGET" ] && [ "$CHANNEL" != "nightly" ]; then
  if [ "$TRAVIS_OS_NAME" = "osx" ]; then
    brew install gnu-sed --default-names
  fi

  find src -name '*.rs' -type f | xargs sed -i -e 's:\(//.\s*```\):\1 ignore,:g'
fi

case $TARGET in
  # use an emulator to run the cross compiled binaries
  arm-unknown-linux-gnueabihf)
    # build tests but don't run them
    # cargo test --target $TARGET --no-run

    # run tests in emulator
    # find target/$TARGET/debug -maxdepth 1 -executable -type f \
    #   -exec qemu-arm -L /usr/arm-linux-gnueabihf '{}' ';'

    # build the main executable
    cargo build --target $TARGET --verbose

    # sanity check the file type
    file target/$TARGET/debug/ilc
    qemu-arm -L /usr/arm-linux-gnueabihf target/$TARGET/debug/ilc -V
    ;;
  *)
    cargo build --target $TARGET --verbose

    # sanity check the file type
    file target/$TARGET/debug/ilc
    target/$TARGET/debug/ilc -V

    # this isn't even a temporary solution. :(
    # cargo test --target $TARGET --verbose
    ;;
esac

