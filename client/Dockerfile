FROM node:22.16.0
LABEL maintainer="Primus Labs"

WORKDIR /por-client
COPY ./package.json /por-client/package.json
RUN npm install

COPY ./src/utils.ts /por-client/src/

CMD ["npx", "tsx", "src/binance.ts"]
