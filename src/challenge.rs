use reqwest::Client;
use serde::Deserialize;
use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use std::thread;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[derive(Deserialize, Debug)]
pub struct ChallengeResponse {
	pub prefix: String,
	pub target: String,
}

pub async fn request_challenge() -> Result<ChallengeResponse, reqwest::Error> {
	let client = Client::new();
	let resp = client
		.get("https://lrclib.net/api/request-challenge")
		.send()
		.await?
		.json::<ChallengeResponse>()
		.await?;
	Ok(resp)
}

fn verify_nonce(result: &Vec<u8>, target: &Vec<u8>) -> bool {
    if result.len() != target.len() {
        return false;
    }

    for i in 0..(result.len() - 1) {
        if result[i] > target[i] {
            return false;
        } else if result[i] < target[i] {
            break;
        }
    }

    return true;
}

pub fn solve_challenge(prefix: &str, target_hex: &str) -> String {
    let num_threads = num_cpus::get();
    let target = HEXUPPER.decode(target_hex.as_bytes()).unwrap();
    let found = Arc::new(AtomicBool::new(false));
    let result = Arc::new(parking_lot::Mutex::new(None));
    let prefix = prefix.to_string();
    let target = Arc::new(target);

    crossbeam_utils::thread::scope(|s| {
        for thread_id in 0..num_threads {
            let prefix = prefix.clone();
            let target = Arc::clone(&target);
            let found = Arc::clone(&found);
            let result = Arc::clone(&result);
            s.spawn(move |_| {
                let mut nonce = thread_id as u64;
                while !found.load(Ordering::Relaxed) {
                    let mut context = Context::new(&SHA256);
                    let input = format!("{}:{}", prefix, nonce);
                    context.update(input.as_bytes());
                    let hashed = context.finish().as_ref().to_vec();
                    if verify_nonce(&hashed, &target) {
                        let mut res = result.lock();
                        *res = Some(nonce);
                        found.store(true, Ordering::Relaxed);
                        break;
                    }
                    nonce += num_threads as u64;
                }
            });
        }
    }).unwrap();

    let res = result.lock();
    res.unwrap().to_string()
}