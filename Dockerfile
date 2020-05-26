FROM node:14.3.0-alpine3.11
COPY . /opt/bot
WORKDIR /opt/bot
RUN yarn
ENTRYPOINT node index.js