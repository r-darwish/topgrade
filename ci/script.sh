# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cargo fmt --all -- --check
    cross clippy --all-targets -- -D warnings
    cross clippy --all-targets --all-features -- -D warnings

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
