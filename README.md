Rust re-implementation of the cs144 c++ tcp stack project: [cs144 c++ sponge](https://github.com/aristotle0x01/sponge)

## benchmark vs c++

both under release build:

![](https://user-images.githubusercontent.com/2216435/223632916-e4885c40-7a39-473f-a03b-9a419fa5e936.png)



## Environment

Dev and debug use the original cs144 vbox ubuntu image

### Network

    // option 1
    $ /tun.sh start 144
    
    // option 2
    $ ip tuntap add dev tun144 mode tun
    $ ip link set tun144 up
    $ ip addr add 10.0.0.1/24 dev tun144
    $ tshark -i tun144
    
    // monitor commands
    $ tshark -f "tcp port 1080"
    $ tshark -Pw /tmp/debug.raw -i tun144
    $ tshark -f "src 169.254.144.9 or dst 169.254.144.9 or src 169.254.145.9 or dst 169.254.145.9" -i tun144



## build options

**General build**

    $ cargo build
    $ cargo build --release --bins // release build all the binaries
    $ cargo build --release --bin tun // release build certain binary
    $ cargo build --example bidirectional_stream_copy // build an example

**test build**

    //**** integration tests under "tests" folder
    $ cargo test // all tests
    $ cargo test --test fsm_winsize // specific test
    $ cargo test --test fsm_winsize -- --show-output // specific test
    
    //**** unittest inside a specific class
    $ cargo test --lib test_deref -- --show-output // specific unittest



## dev & profiling

**debug**

    // show stacktrace when assert failed or fault
    $ RUST_BACKTRACE=1 cargo run --bin tcp_udp
    $ RUST_BACKTRACE=1 cargo test --test fsm_winsize
    $ RUST_BACKTRACE=1 cargo test --test fsm_winsize -- --show-output
    $ RUST_BACKTRACE=1 ./target/debug/tcp_benchmark 2>&1 | grep "xout" > a.txt
    $ RUST_BACKTRACE=1 ./target/debug/tcp_benchmark 2>a.txt
    $ RUST_BACKTRACE=1 ./tcp_udp -t 12 -w 1450  169.254.144.1 7107
    $ RUST_BACKTRACE=1 ./txrx.sh -ucSd 1M -w 32K
    $ lldb -- ./target/debug/fsm_winsize --test
    $ target/debug/tcp_native "-l" "127.0.0.1" "1234"

**perf**

    $ valgrind --tool=callgrind ./target/debug/tcp_benchmark // output callgrind.out.pid
    
    // on Mac
    $ qcachegrind callgrind.out.pid

![](https://user-images.githubusercontent.com/2216435/223631959-5fe8076a-4d0b-468b-b6cd-d80c3224be34.png)



## Fundamentals

**TcpConnection**

![](https://user-images.githubusercontent.com/2216435/223634619-465cea82-fee1-4815-a2d6-84893227b5c9.png)

**bidirectional_stream_copy**

![](https://user-images.githubusercontent.com/2216435/223634573-c4c03c71-29e4-4ac2-8c54-0e077580d8b1.png)

**tcp_sponge_socket**

![](https://user-images.githubusercontent.com/2216435/223950969-38a3875e-c6b3-4f23-80f4-dd01e02fd85b.png)

**tun/tap interface**

[Virtual Networking Devices - TUN, TAP and VETH Pairs Explained](https://www.packetcoders.io/virtual-networking-devices-tun-tap-and-veth-pairs-explained/)

![](https://www.packetcoders.io/content/images/2020/10/image2.png)
