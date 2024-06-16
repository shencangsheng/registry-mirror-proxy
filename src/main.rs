use hyper::client::HttpConnector;
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, Uri};
use hyperlocal::{UnixClientExt, UnixConnector, Uri as LocalUri};
use std::convert::Infallible;
use std::ptr::null;
use hyper::body::HttpBody;
use hyper::header::{CONTENT_TYPE, HeaderValue};

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let docker_register_url = "http://127.0.0.1:5000";

    // 检查是否为 pull image 请求
    if let Some((image_name, image_reference)) = extract_image_info(req.method(), req.uri()) {
        // 执行 pull image 操作
        if let Err(err) = perform_docker_pull_push(&image_name, &image_reference, docker_register_url).await {
            eprintln!("Error in pulling and pushing Docker image: {}", err);
        }
    }

    // 克隆方法和 URI ，以便在消费 req 之后还能使用
    let method = req.method().clone();
    let uri = format!("{}{}", docker_register_url, req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("")).parse::<Uri>().unwrap();

    // 转发请求到私有的 Docker register
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

async fn perform_docker_pull_push(image_name: &str, image_reference: &str, registry_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let docker_client = Client::unix();

    // 使用 Docker REST API pull image
    let pull_url = LocalUri::new(
        "/var/run/docker.sock",
        &format!("/images/create?fromImage={}:{}", image_name, image_reference),
    );
    let pull_req = Request::post(pull_url).body(Body::empty()).unwrap();
    let pull_res = docker_client.request(pull_req).await?;


    // 使用 Docker REST API tag image
    let new_image_tag = format!("{}/{}:{}", registry_url.strip_prefix("http://").unwrap_or(registry_url), image_name, image_reference);
    let tag_url = LocalUri::new(
        "/var/run/docker.sock",
        &format!("/images/{}/tag?repo={}&tag={}", image_name, new_image_tag, image_reference),
    );
    let tag_req = Request::post(tag_url).body(Body::empty()).unwrap();
    docker_client.request(tag_req).await?;

    // 使用 Docker REST API push image
    let push_url = LocalUri::new(
        "/var/run/docker.sock",
        &format!("/images/{}/push", new_image_tag),
    );
    let mut push_req = Request::post(push_url).body(Body::empty()).unwrap();
    push_req.headers_mut().insert("X-Registry-Auth", "123".parse().unwrap());
    push_req.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/vnd.docker.distribution.manifest.v2+json"));

    let push_res = docker_client.request(push_req).await?;

    // 打印 push 响应中的 body
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