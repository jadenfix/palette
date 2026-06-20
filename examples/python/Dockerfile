FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

WORKDIR /examples

RUN pip install --no-cache-dir \
    opentelemetry-sdk \
    opentelemetry-exporter-otlp-proto-grpc

COPY five_line_otel.py otel_smoke.py ./

CMD ["python", "/examples/five_line_otel.py"]
