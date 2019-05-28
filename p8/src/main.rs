#[macro_use]
extern crate lazy_static;

use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use prometheus::*;
use rand::prelude::*;

lazy_static! {
    static ref REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_requests_duration",
        "Histogram of HTTP request duration in seconds",
        &["method"],
        exponential_buckets(0.005, 2.0, 20).unwrap()
    )
    .unwrap();
}

fn thread_simulate_requests() {
    let mut rng = rand::thread_rng();
    loop {
        // Simulate duration 0s ~ 2s
        let duration = rng.gen_range(0f64, 2f64);
        // Simulate HTTP method
        let method = ["GET", "POST", "PUT", "DELETE"].choose(&mut rng).unwrap();
        // Record metrics
        println!("{}\t{:.3}s", method, duration);
        REQUEST_DURATION
            .with_label_values(&[method])
            .observe(duration);
        // One request per second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn metric_service(_req: Request<Body>) -> Response<Body> {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    let mf = prometheus::gather();
    encoder.encode(&mf, &mut buffer).unwrap();
    Response::builder()
        .header(hyper::header::CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap()
}

fn main() {
    std::thread::spawn(thread_simulate_requests);

    let addr = ([127, 0, 0, 1], 12345).into();
    let service = || service_fn_ok(metric_service);
    let server = Server::bind(&addr)
        .serve(service)
        .map_err(|e| panic!("{}", e));

    hyper::rt::run(server);
}
