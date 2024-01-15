FROM archwaynetwork/archwayd:latest

COPY ./entry-point.sh /data/entry-point.sh
RUN chmod +x /data/entry-point.sh

EXPOSE 9090
EXPOSE 1317
EXPOSE 26656
EXPOSE 26657

ENTRYPOINT ["/data/entry-point.sh"]
#CMD ["archwayd", "start", "--pruning", "nothing", "--log_format", "json"]