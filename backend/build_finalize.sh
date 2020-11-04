start_peer_port=7091
start_public_port=9000

while getopts "n:c:s" arg; do
    case $arg in
    n) node_count=$(($OPTARG)) ;;
    c) rm -r example ;;
    s) build=0
    esac
done

echo "node count = $node_count"

if build; then
    cargo install --path .
    ret=$?
    if [ "$ret" != "0" ]; then
        exit 1
    fi
fi


if [ -d ./example ]; then
    echo "example dir exists"
else
    mkdir example
    cd example
    echo "Generating node configs..."
    exonum-ML generate-template common.toml --validators-count ${node_count}
    for i in $(seq 0 $((node_count - 1))); do
        peer_port=$((start_peer_port + i))
        exonum-ML generate-config common.toml $((i + 1)) --peer-address 127.0.0.1:${peer_port} -n
    done
    cd ..
fi

cd example

node_list=($(seq 1 $node_count))

node_list=("${node_list[@]/%//pub.toml}")

echo "Finalizing nodes.."
for i in $(seq 0 $((node_count - 1))); do
    public_port=$((start_public_port + i))
    private_port=$((public_port + node_count))
    exonum-ML finalize --public-api-address 0.0.0.0:${public_port} \
    --private-api-address 0.0.0.0:${private_port} $((i + 1))/sec.toml $((i + 1))/node.toml \
    --public-configs "${node_list[@]}"
done
