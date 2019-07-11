set -xe

./build.sh

cp target/x86_64-unknown-linux-musl/release/openwhisk exec
zip action.zip exec
wsk action update mc_creation_date action.zip --native --web true
