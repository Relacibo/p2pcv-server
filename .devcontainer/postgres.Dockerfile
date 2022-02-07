FROM postgres:14.1

RUN apt-get update \
    && apt-get install -y postgresql-contrib-14 \
    && apt-get autoremove -y && apt-get clean -y