use arma_bench::{Client, CompareRequest, CompareResult, ServerConfig};

fn main() {
    let client = Client::connect("localhost", &ServerConfig::default()).expect("Failed to connect");
    let requests = vec![
        CompareRequest {
            id: 0,
            content: "private _a = 1; private _b = 2; _a + _b"
                .as_bytes()
                .to_vec(),
            sqfc: false,
        },
        CompareRequest {
            id: 1,
            content: b"1 + 2".to_vec(),
            sqfc: false,
        },
    ];
    let results = client.compare(requests).expect("Failed to compare");
    for result in results {
        let CompareResult {
            id,
            time,
            iter,
            ret,
        } = result;
        println!("[{id}] Time: {time} ms, Iterations: {iter}");
        println!("Result: {ret}");
    }
}
