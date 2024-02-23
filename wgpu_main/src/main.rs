#[tokio::main]
async fn main() {
    wgpu_main::run().await;
    //  match wgpu_main::run(){
    //     Err(e) => eprintln!("{e}"),
    //     _ => (),
    // };
}
