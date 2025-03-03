use opentelemetry::{global, KeyValue};
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_VERSION,
};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use tracing::Level;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

pub fn setup_tracing() -> OtelGuard {
    let otel_exporter_url =
        std::env::var("OTEL_EXPORTER_URL").unwrap_or_else(|_| "http://localhost:4317".to_string());

    let otel_meter_provider = init_meter_provider(otel_exporter_url.as_str());
    let otel_tracer_provider = init_tracer_provider(otel_exporter_url.as_str());
    let otel_logger_provider = init_logger_provider(otel_exporter_url.as_str());

    let otel_layer = OpenTelemetryTracingBridge::new(&otel_logger_provider);
    let otel_env_filter_layer = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let otel_logger_layer = otel_layer.with_filter(otel_env_filter_layer);

    let fmt_env_filter_layer = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("opentelemetry=debug".parse().unwrap());

    let fmt_logger_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .with_filter(fmt_env_filter_layer);

    let tracer = otel_tracer_provider.tracer("fibchain-otel-subscriber");
    let otel_tracer_layer = OpenTelemetryLayer::new(tracer);

    let otel_meter_layer = MetricsLayer::new(otel_meter_provider.clone());

    let level_filter_layer = tracing_subscriber::filter::LevelFilter::from_level(
        Level::INFO,
    );

    tracing_subscriber::Registry::default()
        .with(level_filter_layer)
        .with(otel_logger_layer)
        .with(fmt_logger_layer)
        .with(otel_meter_layer)
        .with(otel_tracer_layer)
        .init();
    
    OtelGuard {
        logger_provider: otel_logger_provider,
        tracer_provider: otel_tracer_provider,
        meter_provider: otel_meter_provider,
    }
}

fn get_span_resource() -> Resource {
    Resource::builder()
        .with_schema_url(
            [
                KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "development"),
            ],
            SCHEMA_URL,
        )
        .with_service_name(env!("CARGO_PKG_NAME"))
        .build()
}

fn init_meter_provider(otel_collector_url: &str) -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(otel_collector_url)
        .with_temporality(opentelemetry_sdk::metrics::Temporality::Delta)
        .build()
        .unwrap();

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(10))
        .build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(get_span_resource())
        .with_reader(reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    meter_provider
}

fn init_tracer_provider(otel_collector_url: &str) -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otel_collector_url)
        .build()
        .unwrap();

    let sampler = Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(1.0)));

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(get_span_resource())
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    tracer_provider
}

fn init_logger_provider(otel_collector_url: &str) -> SdkLoggerProvider {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(otel_collector_url)
        .build()
        .unwrap();

    SdkLoggerProvider::builder()
        .with_resource(get_span_resource())
        .with_batch_exporter(exporter)
        .build()
}

pub struct OtelGuard {
    logger_provider: SdkLoggerProvider,
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.logger_provider.shutdown() {
            eprintln!("Error shutting down logger provider: {}", err);
        }
        
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {}", err);
        }
        
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Error shutting down meter provider: {}", err);
        }
    }
}