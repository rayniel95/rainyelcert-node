FROM docker.uclv.cu/ubuntu:focal as builder
# TODO - delete next line if you are not from Cuba
COPY ./sources.list /etc/apt/
# TODO - delete proxy lines
RUN export HTTP_PROXY="http://192.168.49.1:8282" \
    && export http_proxy="http://192.168.49.1:8282" \
    https_proxy="http://192.168.49.1:8282" \
    && apt update \
    && unset HTTP_PROXY http_proxy https_proxy

RUN export HTTP_PROXY="http://192.168.49.1:8282" \
    && export http_proxy="http://192.168.49.1:8282" \
    https_proxy="http://192.168.49.1:8282" \
    && apt install --no-install-recommends -y git clang curl libssl-dev llvm libudev-dev ca-certificates \
    && unset HTTP_PROXY http_proxy https_proxy

RUN export HTTP_PROXY="http://192.168.49.1:8282" \
    && export http_proxy="http://192.168.49.1:8282" \
    https_proxy="http://192.168.49.1:8282" \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && unset HTTP_PROXY http_proxy https_proxy

RUN export HTTP_PROXY="http://192.168.49.1:8282" \
    && export http_proxy="http://192.168.49.1:8282" \
    https_proxy="http://192.168.49.1:8282" \
    && . $HOME/.cargo/env \
    && rustup default stable \
    && rustup update \
    && rustup update nightly \
    && rustup target add wasm32-unknown-unknown --toolchain nightly \
    && unset HTTP_PROXY http_proxy https_proxy

WORKDIR /usr/src/app

COPY ./ ./

RUN export HTTP_PROXY="http://192.168.49.1:8282" \
    && export http_proxy="http://192.168.49.1:8282" \
    https_proxy="http://192.168.49.1:8282" \
    && . $HOME/.cargo/env \
    && cargo build --release \
    && unset HTTP_PROXY http_proxy https_proxy

################################################################

FROM docker.uclv.cu/ubuntu:focal
    
COPY --from=builder /usr/src/app/target /usr/local/bin/myapp

WORKDIR /usr/local/bin/myapp

CMD ["release/node-template"]