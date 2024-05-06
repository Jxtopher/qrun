FROM rust:slim-buster

ARG DEBIAN_FRONTEND=noninteractive

# Package installation
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends git \
    python3 \
    sudo \
    htop \
    stress \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* \
    && rustup component add rustfmt \
    && useradd -ms /bin/bash user \
    && chmod 660 /etc/sudoers && echo "user ALL=NOPASSWD: ALL" >> /etc/sudoers && chmod 400 /etc/sudoers

WORKDIR /app

ENTRYPOINT ["su", "user"]
