FROM postgres:14.1

RUN apt-get update \
    && apt-get install -y postgresql-contrib-14