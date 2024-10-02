Building for target platform (Milk-V Duo S)

`docker run --rm -v $(pwd):/workspace -w /workspace openlch-runtime-sdk /root/.cargo/bin/cargo +nightly build --target riscv64gc-unknown-linux-musl -Zbuild-std --release`

Uploading to the board (ethernet over usb c)
`scp -O target/riscv64gc-unknown-linux-musl/release/runtime root@192.168.42.1:`

Remember to first build the docker container with sdk/toolchain

```
cd ../runtime-sdk
docker build . -t openlch-runtime-sdk
```

