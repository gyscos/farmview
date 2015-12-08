FROM golang:1.5-onbuild

ENTRYPOINT ["app", "-c", "/config.toml"]

CMD ["-p", "8080"]
