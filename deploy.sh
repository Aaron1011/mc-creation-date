set -xe

./build.sh

cp target/x86_64-unknown-linux-musl/release/openwhisk exec
zip action.zip exec
ibmcloud fn action update mc_creation_date action.zip --native --web true
ibmcloud fn action invoke mc_creation_date --param name Aaron1011 --result
