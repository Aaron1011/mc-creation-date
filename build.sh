docker pull clux/muslrust@sha256:c7307fd2b91f8861c014144211045cb43386c4504c9c216a6517745a270f3c3a

docker run \
    -v cargo-cache:/root/.cargo/registry \
    -v "$PWD:/volume" \
    --rm -it clux/muslrust cargo build --release
