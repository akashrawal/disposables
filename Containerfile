FROM docker.io/rust:1.81-alpine AS build

RUN apk add build-base

WORKDIR /build
COPY . .
RUN cargo install --path dlc --root dlc_out

FROM scratch
COPY --from=build /build/dlc_out/ /
ENV RUST_BACKTRACE=1
ENTRYPOINT ["/bin/dlc"]




