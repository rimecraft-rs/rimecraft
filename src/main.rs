#[tokio::main]
async fn main() {
    if cfg!(feature = "client") {
        todo!()
    } else if cfg!(feature = "dedicated_server") {
        rimecraft::server::run().await
    } else {
        unreachable!()
    }
}
