FROM ghcr.io/casualjim/bare:libcxx-ssl AS runtime

ENV RUST_LOG=info

COPY dist/ /app
WORKDIR /app

USER appuser

# Expose server port (configurable via EXPOSE_PORT build arg)
ARG PORT=8080
EXPOSE $PORT
ENTRYPOINT ["/app/{{project-name}}"]
