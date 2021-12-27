use super::Writer;
use http::{uri::InvalidUri, StatusCode, Uri};

pub struct HttpWriter {
    inner: sealed::HttpClient,
}

impl HttpWriter {
    pub fn new<U>(url: U) -> Self
    where
        U: TryInto<Uri, Error = InvalidUri>,
    {
        let url = url.try_into().expect("http endpoint must be valid...");
        let scheme = url.scheme().map(|scheme| scheme.as_str()).unwrap_or("http");
        Self {
            inner: if scheme == "https" {
                sealed::HttpClient::new_https(url)
            } else {
                sealed::HttpClient::new_http(url)
            },
        }
    }
}

impl Writer for HttpWriter {
    fn write(&self, log: String) {
        let http_call = async move { self.inner.post(log).await };
        let res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(http_call);
        match res {
            Ok(response) => {
                let response_status = response.status();
                if response_status != StatusCode::CREATED && response_status != StatusCode::OK {
                    eprintln!(
                        "[Logging] Error response status from http endpoint: {}",
                        response_status
                    );
                }
            }
            Err(err) => {
                eprintln!(
                    "[Logging] Error while post log data to http endpoint: {}",
                    err
                );
            }
        }
    }
}

mod sealed {
    use http::Response;
    use hyper::{
        client::{connect::Connect, HttpConnector},
        header, Body, Result, Uri,
    };
    use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};

    #[derive(Debug, Clone)]
    pub struct HyperClient<C> {
        uri: Uri,
        inner: hyper::Client<C>,
    }

    impl<C> HyperClient<C> {
        pub fn new(uri: Uri, inner: hyper::Client<C>) -> Self {
            Self { uri, inner }
        }
    }

    impl<C> HyperClient<C>
    where
        C: Connect + Clone + Send + Sync + 'static,
    {
        pub async fn post(&self, log: String) -> Result<Response<Body>> {
            let log_bytes = log.into_bytes();
            let mut request = hyper::Request::builder()
                .method("POST")
                .uri(&self.uri)
                .body(hyper::Body::from(log_bytes))
                .expect("");
            let headers = request.headers_mut();
            headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
            self.inner.request(request).await
        }
    }

    #[derive(Debug, Clone)]
    pub enum HttpClient {
        Http(HyperClient<HttpConnector>),
        Https(HyperClient<HttpsConnector<HttpConnector>>),
    }

    impl HttpClient {
        pub fn new_http(uri: Uri) -> Self {
            Self::Http(HyperClient::new(uri, hyper::Client::new()))
        }
        pub fn new_https(uri: Uri) -> Self {
            let connector = HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_only()
                .enable_http1()
                .build();
            Self::Https(HyperClient::new(
                uri,
                hyper::Client::builder().build(connector),
            ))
        }
        pub async fn post(&self, log: String) -> Result<Response<Body>> {
            match self {
                HttpClient::Http(c) => c.post(log).await,
                HttpClient::Https(c) => c.post(log).await,
            }
        }
    }
}
