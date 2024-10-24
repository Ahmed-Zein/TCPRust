#!/bin/bash

tap_name="tap0" 

# Function to clean up on exit
cleanup() {
  echo "Cleaning up..."
  sudo ip link set down dev $tap_name
  sudo ip addr del 192.168.0.1/24 dev $tap_name
  kill $pid
}

# Set capability to TCPRust binary
sudo setcap CAP_NET_ADMIN=eip ./target/release/TCPRust

# Run the program in the background
cargo run --release &
pid=$!

# Ensure the tap interface exists (create if needed)
# if ! ip link show $tap_name > /dev/null 2>&1; then
 # sudo ip tuntap add dev $tap_name mode tap
# fi

# Configure the network interface
sudo ip addr add 192.168.0.1/24 dev $tap_name 
sudo ip link set up dev $tap_name

# Trap SIGINT and SIGTERM to cleanup
trap cleanup INT TERM

# Wait for the process to finish
wait $pid

