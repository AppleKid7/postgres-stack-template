FROM ghcr.io/linuxserver/baseimage-kasmvnc:ubuntunoble

ARG STARGATE_VERSION=0.1.0
ARG STARGATE_SHA256=9633cc238104e53b0c27e77a39d0a7bb18df402cb756a5639ff48a9d71234536

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      curl \
      ca-certificates && \
    curl -fSL -o /tmp/stargate.deb \
      "https://github.com/fabianboesiger/stargate/releases/download/v${STARGATE_VERSION}/stargate-linux-x86_64.deb" && \
    echo "${STARGATE_SHA256}  /tmp/stargate.deb" | sha256sum -c - && \
    apt-get install -y --no-install-recommends /tmp/stargate.deb && \
    rm -f /tmp/stargate.deb && \
    apt-get purge -y curl && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

RUN mkdir -p /defaults && \
    echo "stargate" > /defaults/autostart

ENV CUSTOM_USER=stargate
ENV PASSWORD=stargate
ENV TITLE="Stargate"
