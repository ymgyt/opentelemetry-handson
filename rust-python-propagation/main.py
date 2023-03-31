import fastapi
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import ConsoleSpanExporter, SimpleSpanProcessor,BatchSpanProcessor
from opentelemetry.trace.span import Span
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.sdk.resources import Resource
from opentelemetry.semconv.trace import SpanAttributes
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
import logging


def configure_tracer():
    # exporter = ConsoleSpanExporter()
    exporter= OTLPSpanExporter(endpoint="http://localhost:4317", insecure=True)
    span_processor = BatchSpanProcessor(exporter)
    resource = Resource.create(
        {
        "service.name": "goodservice",
        "service.version": "0.1.2",
        }
    )

    provider = TracerProvider(resource=resource)
    # comment out to surpress otel
    provider.add_span_processor(span_processor)
    
    trace.set_tracer_provider(provider)
    return trace.get_tracer(__name__, "0.0.1")

def configure_logger():
    level = "INFO"
    handler = logging.StreamHandler()
    handler.setLevel(level)
    logger = logging.getLogger(__name__)
    logger.setLevel(level)
    logger.addHandler(handler)
    return logger


app = fastapi.FastAPI()
tracer = configure_tracer()
logger = configure_logger()

@app.get("/foo")
async def foo():
    user = "ymgyt"
    with tracer.start_as_current_span(
        "work",
        kind=trace.SpanKind.SERVER,
        attributes={ 
            SpanAttributes.ENDUSER_ID: user,
            SpanAttributes.ENDUSER_ROLE: "admin",
        },
    )as span:
        result = work()
        span.set_attribute("work_result", result["result"])

        return { "message": "hello" }

def work():
    span = trace.get_current_span()
    span.add_event("work", {
        "condition": 100,
    })
    span.set_attribute("xxx", "yyy")
    work_inner()

    return {"result": "OK"}

@tracer.start_as_current_span("work_inner")
def work_inner():
    return 100

@app.get("/healthcheck")
async def healthcheck():
    return "OK"

def server_request_hook(span: Span,scope: dict):
    if span and span.is_recording():
        span.set_attribute("from_request_hook", "v1")

def client_request_hook(span: Span, scope: dict):
    if span and span.is_recording():
        span.set_attribute("custom_user_attribute_from_client_request_hook", "some-value")

        
def client_response_hook(span: Span, message: dict):
    if span and span.is_recording():
        span.set_attribute("custom_user_attribute_from_response_hook", "some-value")


FastAPIInstrumentor.instrument_app(
    app,
    excluded_urls="internal/*, healthcheck", # OTEL_PYTHON_FASTAPI_EXCLUDED_URLS=xxx
    # server_request_hook=server_request_hook,
    # client_request_hook=client_request_hook,
    # client_response_hook=client_response_hook,
    tracer_provider=trace.get_tracer_provider(),

)
