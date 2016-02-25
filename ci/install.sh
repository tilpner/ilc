# `install` phase: install stuff needed for the `script` phase

set -ex

case $TARGET in
  # Install standard libraries needed for cross compilation
  arm-unknown-linux-gnueabihf | \
  i686-apple-darwin | \
  i686-unknown-linux-gnu | \
  x86_64-unknown-linux-musl)
    if [ "$TARGET" = "arm-unknown-linux-gnueabihf" ]; then
      # information about the cross compiler
      arm-linux-gnueabihf-gcc -v

      # tell cargo which linker to use for cross compilation
      mkdir -p .cargo
      cat >.cargo/config <<EOF
[target.$TARGET]
linker = "arm-linux-gnueabihf-gcc"
EOF
    fi

    # e.g. 1.6.0
    # doesn't work for nightly
    # version=$(rustc -V | cut -d' ' -f2)
    version=$(rustc -V | cut -d' ' -f2 | cut -d'-' -f2)

    if [ "$TARGET" = "x86_64-unknown-linux-musl" ]; then
        version="nightly"
    fi
    tarball=rust-std-${version}-${TARGET}

    curl -Os http://static.rust-lang.org/dist/${tarball}.tar.gz

    tar xzf ${tarball}.tar.gz

    ${tarball}/install.sh --prefix=$(rustc --print sysroot)

    rm -r ${tarball}
    rm ${tarball}.tar.gz
    ;;
  # Nothing to do for native builds
  *)
    ;;
esac

# TODO if you need to install extra stuff add it here
