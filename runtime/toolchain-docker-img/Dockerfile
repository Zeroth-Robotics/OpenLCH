FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y git curl build-essential file bc bison  \
      build-essential flex libssl-dev unzip wget cpio rsync && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN . "$HOME/.cargo/env" && \
    rustup toolchain install nightly && \
    rustup component add rust-src --toolchain nightly

RUN git clone https://github.com/kinsamanka/milkv-buildroot --depth 1

RUN make O=$(pwd)/sdk -C milkv-buildroot milkv_duo_toolchain_defconfig && \
    cd sdk && \
    make sdk -j$(nproc)

RUN echo 'source ~/.cargo/env' >> ~/.bashrc


