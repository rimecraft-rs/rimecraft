fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        rimecraft::client::main::main(None).await;
    })
}
