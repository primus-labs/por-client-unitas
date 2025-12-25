FROM ubuntu:22.04
LABEL maintainer="Primus Labs"

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends apt-utils gcc g++ make build-essential python3 curl ca-certificates \
    && update-ca-certificates && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://nodejs.org/dist/v22.16.0/node-v22.16.0-linux-x64.tar.xz -o node.tar.xz \
  && tar -xf node.tar.xz -C /usr/local/ --strip-components=1 && rm node.tar.xz
  
COPY ./package.json /app/package.json
RUN npm config set registry https://registry.npmmirror.com && npm install

COPY ./src/binance.ts /app/src/

CMD ["npx", "tsx", "src/binance.ts"]
