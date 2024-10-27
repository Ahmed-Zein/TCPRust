#!/bin/bash

tun="tun0"
cargo b --release
ext=$?
if [[ $ext -ne 0 ]]; then
	exit $ext
fi
sudo setcap cap_net_admin=eip ./target/release/TCPRust
./target/release/TCPRust &

pid=$!
sudo ip addr add 192.168.0.1/24 dev $tun
sudo ip link set up dev $tun
trap "kill $pid" INT TERM
wait $pid
