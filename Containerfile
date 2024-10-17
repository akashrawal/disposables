FROM --platform=$BUILDPLATFORM docker.io/rust:1.81-alpine AS build
ARG BUILDARCH
ARG TARGETARCH

RUN apk add build-base

WORKDIR /build
COPY . .
RUN ./dlc/crossbuild.sh

FROM scratch
COPY --from=build /build/dlc_out/ /
ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
ENTRYPOINT ["/bin/dlc"]




