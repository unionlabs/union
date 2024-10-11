ARG VARIANT="1-1.21-bullseye"
FROM mcr.microsoft.com/devcontainers/go:${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive

RUN type -p curl >/dev/null || (apt-get install curl -y)


# install gloud sdk
RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] http://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key --keyring /usr/share/keyrings/cloud.google.gpg  add - && apt-get update -y && apt-get install google-cloud-cli -y
