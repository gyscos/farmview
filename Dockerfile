FROM golang:alpine

RUN apk add --update git && rm -rf /var/cache/apk/*

RUN mkdir -p /go/src/farmview
WORKDIR /go/src/farmview

COPY static /go/src/farmview/static
COPY templates /go/src/farmview/templates
COPY *.go /go/src/farmview/

RUN go get -v -d
RUN go install

ENTRYPOINT ["farmview", "-c", "/config.toml"]

CMD ["-p", "8080"]
