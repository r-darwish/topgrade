# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cargo fmt --all -- --check
    cross clippy --all-targets -- -D warnings
    cross clippy --all-targets --all-features -- -D warnings
    cross check --target $TARGET --release --all-features

    if [ ! -z $DISABLE_TESTS ]; then
        cross test
        return
    fi

}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
