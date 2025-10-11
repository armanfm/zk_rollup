
//pai_circuit verifica a prova real
use pasta_curves::pallas::Base as Fp;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Column, ConstraintSystem, Selector, Circuit, Error},
    poly::Rotation,
    plonk::{keygen_vk, verify_proof, SingleVerifier, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};
use std::fs::File;
use std::io::Read;
use halo2_proofs::pasta::EqAffine;

const K: u32 = 10;

/// Configuração do circuito
#[derive(Clone, Debug)]
pub struct PaiConfig {
    x: Column<Advice>,
    y: Column<Advice>,
    s_mul2: Selector,
}

/// Circuito principal
pub struct PaiCircuit {
    pub x: Option<Fp>,
    pub y: Option<Fp>,
}

impl Circuit<Fp> for PaiCircuit {
    type Config = PaiConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { x: None, y: None }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let s_mul2 = meta.selector();

        meta.create_gate("y = 2 * x", |meta| {
            let s = meta.query_selector(s_mul2);
            let x_val = meta.query_advice(x, Rotation::cur());
            let y_val = meta.query_advice(y, Rotation::cur());
            let two_x = x_val.clone() + x_val.clone();
            vec![s * (y_val - two_x)]
        });

        PaiConfig { x, y, s_mul2 }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_region(
            || "mul2 region",
            |mut region| {
                config.s_mul2.enable(&mut region, 0)?;
                let x_val = self.x.ok_or(Error::Synthesis)?;
                let y_val = self.y.unwrap_or(x_val + x_val);

                region.assign_advice(|| "x value", config.x, 0, || Value::known(x_val))?;
                region.assign_advice(|| "y value", config.y, 0, || Value::known(y_val))?;

                Ok(())
            },
        )
    }
}

impl PaiCircuit {
    /// Cria circuito a partir de 64 bytes (32 para x, 32 para y)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 64 {
            return Err("Esperado 64 bytes".into());
        }

        let x_bytes: [u8; 32] = bytes[0..32].try_into().unwrap();
        let y_bytes: [u8; 32] = bytes[32..64].try_into().unwrap();

        let x = Fp::from_raw([
            u64::from_le_bytes(x_bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(x_bytes[8..16].try_into().unwrap()),
            u64::from_le_bytes(x_bytes[16..24].try_into().unwrap()),
            u64::from_le_bytes(x_bytes[24..32].try_into().unwrap()),
        ]);

        let y = Fp::from_raw([
            u64::from_le_bytes(y_bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(y_bytes[8..16].try_into().unwrap()),
            u64::from_le_bytes(y_bytes[16..24].try_into().unwrap()),
            u64::from_le_bytes(y_bytes[24..32].try_into().unwrap()),
        ]);

        Ok(PaiCircuit { x: Some(x), y: Some(y) })
    }
}

/// Verifica uma única prova usando VerifyingKey já gerado
pub fn verify_pai_circuit_with_vk(
    vk: &VerifyingKey<EqAffine>,
    params: &Params<EqAffine>,
    proof_path: &str,
    public_input_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let circuit = PaiCircuit::from_bytes(public_input_bytes)
        .map_err(|e| format!("Falha ao converter bytes para Fp: {}", e))?;

    let mut proof_file = File::open(proof_path)?;
    let mut proof_bytes = Vec::new();
    proof_file.read_to_end(&mut proof_bytes)?;

    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof_bytes[..]);

    let x_val = circuit.x.unwrap();
    let y_val = circuit.y.unwrap();
    let public_inputs: &[&[Fp]] = &[&[x_val, y_val]];
    let public_inputs_wrapper: &[&[&[Fp]]] = &[public_inputs];

    let strategy = SingleVerifier::new(params);
    verify_proof(params, vk, strategy, public_inputs_wrapper, &mut transcript)?;

    Ok(())
}

/// Verifica múltiplas provas reutilizando o mesmo VerifyingKey e Params
pub fn verify_multiple_pai_proofs(
    vk: &VerifyingKey<EqAffine>,
    params: &Params<EqAffine>,
    proofs: &[(String, Vec<u8>)], // (caminho da prova, 64 bytes de input)
) -> Result<(), Box<dyn std::error::Error>> {
    for (proof_path, public_input_bytes) in proofs {
        verify_pai_circuit_with_vk(vk, params, proof_path, public_input_bytes)?;
    }
    Ok(())
}
