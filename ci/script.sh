# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cargo fmt --all -- --check
    cross check --target $TARGET
    cross check --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
