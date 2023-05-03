use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use async_std::task::block_on;
use axum::response::Sse;
use axum::response::sse::Event;
use axum::{Router};
use axum::routing::get;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;
use futures_util::stream::StreamExt;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let middleware_stack = ServiceBuilder::new()
        // add high level tracing of requests and responses
        .layer(TraceLayer::new_for_http())
        // compression responses
        .layer(CompressionLayer::new())
        // convert the `ServiceBuilder` into a `tower::Layer`
        .into_inner();

    let app = Router::new()
        .route("/sse", get(sse_handler)).layer(middleware_stack);

    let addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {

    let (sender_response, receiver_response) = mpsc::channel::<Event>(1000);

    let data_string = r#"{"aId":"12345678","bId":"987654321","cId":"abcd","mTNMoCqZU1":"ZlrWCKb059","3yP7piSDG7":"GtfCgYLgH2","3nuoqDxyRz":{"OucP7ijiVU":0.14,"NWn5eWTRaI":0.0,"r4IOJ70eyL":"jf7NIgxIps"},"z1B2mZkewj":{"cqK8P5PywK":0.0,"WmYFwzxjiY":0.0,"ubsmYvJ5Hg":"IWxVYO9JKf"},"dYHmrI5Y9z":{"I8rQVmnQeb":0.0,"NalXDhKiIa":0.0,"d8PTvwdTWr":"eI35Jtaypy"},"0B95i170aM":{"ZOOqle02qc":0.14,"NIFqhnXQPF":"IzefB2DUMD"},"eI35Jtaypy":{"K25PRv3UQ0":0.14,"crqvveK9et":"vk5tc1h5rV"},"K25PRv3UQ0":{"kOaKR8HL4O":0.14,"IjAsfT3tvI":"jDxH8kt2V9"},"lZThnkPs5H":{"IXbxrs8ogd":{"QEnsb0MHBa":"200","v6RNnCGvSa":"Kt8FMmMm5w"}},"EJSK7OWYCY":{"ghbtUQUWZD":1.0,"rLuBOaDwNU":0.0,"5ZVLA2cudi":0.0,"vo0Kl4voMe":0.0}}"#;
    let _spawn_handle = std::thread::spawn(move || block_on(async {
        loop {
            let send_result = sender_response.send(Event::default().data(data_string).event("dummy_data").id(Uuid::new_v4().to_string())
            ).await;

            if let Err(_) = send_result {
                break;
            }
        }

    }));

    let response_stream = ReceiverStream::new(receiver_response);
    let stream = response_stream.map(|event|
        Ok(event)
    );


    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
