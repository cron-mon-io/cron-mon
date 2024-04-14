FROM public.ecr.aws/docker/library/node:20.12-slim

WORKDIR /usr/cron-mon/app


RUN npm install -g npm@latest

COPY ./app .
ENV PATH /usr/cron-mon/app/node_modules/.bin:$PATH
