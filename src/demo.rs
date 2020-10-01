use hyper::body::Buf;
use hyper::{Client, Uri};
use std::fs::File;
use std::io::Write;
// https://www.mobt3ath.com/uplode/book/

pub async fn download() {
    let client = Client::new();
    let response = client
        .get(Uri::from_static("https://www.mobt3ath.com/uplode/book/"))
        .await
        .unwrap();
    // let body = response.body();
    let bytes = hyper::body::to_bytes(response).await.unwrap();
    let mut file = File::create(r"C:\Users\l\CLionProjects\Example\a.pdf").unwrap();
    file.write_all(bytes.bytes()).unwrap()
}
