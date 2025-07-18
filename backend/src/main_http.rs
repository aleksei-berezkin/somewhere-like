use std::convert::Infallible;
use std::net::SocketAddr;

use backend::library::handle_request::handle_request;
use http_body_util::Full;
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

/// https://hyper.rs/guides/1/server/hello-world/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;

    eprintln!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let req_body = req.into_body().collect().await;
    match req_body {
        Ok(collected) => {
            let req_body_bytes = collected.to_bytes().into_iter().collect::<Vec<_>>();
            match String::from_utf8(req_body_bytes) {
                Ok(req_body_utf8) => {
                    match handle_request(req_body_utf8, false) {
                        Ok(response) => {
                            Ok(ok_resp(response))
                        }
                        Err(msg) => {
                            Ok(bad_req_resp(msg))
                        }
                    }
                }
                Err(msg) => {
                    Ok(bad_req_resp(msg.to_string()))
                }
            }
        }
        Err(msg) => {
            Ok(bad_req_resp(msg.to_string()))
        }
    }
}

fn bad_req_resp(msg: String) -> Response<Full<Bytes>> {
    let mut resp = ok_resp(msg);
    *resp.status_mut() = StatusCode::BAD_REQUEST;
    resp
}

fn ok_resp(str: String) -> Response<Full<Bytes>> {
    Response::new(Full::new(Bytes::from(str)))
}
