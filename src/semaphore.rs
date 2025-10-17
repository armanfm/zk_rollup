use warp::Filter;
use std::sync::Arc;
use tokio::sync::Mutex;
use halo2_proofs::{
    plonk::{keygen_vk, SingleVerifier, verify_proof, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};
use halo2curves::pasta::pallas::Affine as EpAffine;
use halo2_minimal::AggregatorCircuit;

pub struct Semaphore {
    pub params: Params<EpAffine>,
}

impl Semaphore {
    pub fn new(params: Params<EpAffine>) -> Self {
        Self { params }
    }

    pub fn verify_recursive(
        &self,
        proof_bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Gera o VK do AggregatorCircuit
        let agg_shape = AggregatorCircuit {
            sub_proofs: vec![],
            sub_public_inputs: vec![],
            sub_vks: vec![],
            params: self.params.clone(),
        };
        let vk: VerifyingKey<EpAffine> = keygen_vk(&self.params, &agg_shape)?;

        // Verifica a prova recursiva recebida
        let mut transcript = Blake2bRead::<_, _, Challenge255<EpAffine>>::init(proof_bytes);
        let strategy = SingleVerifier::new(&self.params);
        verify_proof(&self.params, &vk, strategy, &[], &mut transcript)?;
        Ok(())
    }
}

fn with_semaphore(
    semaphore: Arc<Mutex<Semaphore>>,
) -> impl Filter<Extract = (Arc<Mutex<Semaphore>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || semaphore.clone())
}

async fn handle_verify(
    proof_bytes: bytes::Bytes,
    semaphore: Arc<Mutex<Semaphore>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let sem = semaphore.lock().await;
    match sem.verify_recursive(&proof_bytes) {
        Ok(()) => Ok(warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Prova recursiva verificada!"
        }))),
        Err(e) => Ok(warp::reply::json(&serde_json::json!({
            "success": false,
            "message": format!("Erro na verificação: {}", e)
        }))),
    }
}

#[tokio::main]
async fn main() {
    let k = 8u32;
    let params: Params<EpAffine> = Params::new(k);
    let semaphore = Arc::new(Mutex::new(Semaphore::new(params)));

    let route = warp::post()
        .and(warp::path("submit_proof"))
        .and(warp::body::bytes())
        .and(with_semaphore(semaphore))
        .and_then(handle_verify);

    println!("🚀 Semaphore rodando em http://127.0.0.1:3030");
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}
