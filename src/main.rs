use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, Uri};
use hyperlocal::{UnixClientExt, Uri as LocalUri};
use std::convert::Infallible;
use hyper::header::{CONTENT_TYPE, HeaderValue};

const DOCKER_REGISTER_URL: &str = "http://docker-registry:5000";
const DOCKER_SOCKET_PATH: &str = "/var/run/docker.sock";

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if let Some((image_name, image_reference)) = extract_image_info(req.method(), req.uri()) {
        if let Err(err) = perform_docker_pull_push(&image_name, &image_reference).await {
            eprintln!("Error in pulling and pushing Docker image: {}", err);
        }
    }

    let method = req.method().clone();
    let uri = format!("{}{}", DOCKER_REGISTER_URL, req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("")).parse::<Uri>().unwrap();

    let client = Client::new();
    let mut new_req = Request::new(req.into_body());
    *new_req.method_mut() = method;
    *new_req.uri_mut() = uri;

    match client.request(new_req).await {
        Ok(res) => Ok(res),
        Err(_) => Ok(Response::new(Body::from("Something went wrong."))),
    }
}

fn extract_image_info(method: &Method, uri: &Uri) -> Option<(String, String)> {
    if method == Method::GET && uri.path().starts_with("/v2/") {
        let segments: Vec<&str> = uri.path().trim_start_matches("/v2/").split('/').collect();
        if segments.len() >= 3 && segments[segments.len() - 2] == "manifests" {
            let image_name = segments[0..(segments.len() - 2)].join("/");
            let image_reference = segments.last().unwrap().to_string();
            return Some((image_name, image_reference));
        }
    }
    None
}

async fn perform_docker_pull_push(image_name: &str, image_reference: &str) -> Result<(), Box<dyn std::error::Error>> {
    let docker_client = Client::unix();

    let pull_url = LocalUri::new(
        DOCKER_SOCKET_PATH,
        &format!("/images/create?fromImage={}:{}", image_name, image_reference),
    );
    let pull_req = Request::post(pull_url).body(Body::empty()).unwrap();
    docker_client.request(pull_req).await?;

    let new_image_tag = format!("{}/{}:{}", DOCKER_REGISTER_URL.strip_prefix("http://").unwrap_or(DOCKER_REGISTER_URL), image_name, image_reference);
    let tag_url = LocalUri::new(
        DOCKER_SOCKET_PATH,
        &format!("/images/{}/tag?repo={}&tag={}", image_name, new_image_tag, image_reference),
    );
    let tag_req = Request::post(tag_url).body(Body::empty()).unwrap();
    docker_client.request(tag_req).await?;

    let push_url = LocalUri::new(
        DOCKER_SOCKET_PATH,
        &format!("/images/{}/push", new_image_tag),
    );
    let mut push_req = Request::post(push_url).body(Body::empty()).unwrap();
    push_req.headers_mut().insert("X-Registry-Auth", "123".parse().unwrap());
    push_req.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/vnd.docker.distribution.manifest.v2+json"));

    let push_res = docker_client.request(push_req).await?;
    let push_body_bytes = to_bytes(push_res.into_body()).await.unwrap();
    println!("Push Response Body: {}", String::from_utf8_lossy(&push_body_bytes));

    Ok(())
}

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}