//verifier_circuit só recebe os bytes não verifica

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Error, VerifyingKey},
};
use pasta_curves::pallas::{Affine as EpAffine, Scalar as Fr};

/// Circuito que verifica uma prova externa
#[derive(Clone)]
pub struct VerifierCircuit {
    pub proof_bytes: Vec<u8>,
    pub public_input: Fr,
    pub vk: VerifyingKey<EpAffine>,
}

impl Circuit<Fr> for VerifierCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            proof_bytes: vec![],
            public_input: Fr::zero(),
            vk: self.vk.clone(),
        }
    }

    fn configure(_meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        ()
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "verify external proof",
            |_region| {
                // 🔹 Aqui seria o gadget real de verificação:
                // Por enquanto apenas simulamos, mas mantendo o valor público.
                println!(
                    "🔹 Verificando prova externa de {} bytes com input {:?}",
                    self.proof_bytes.len(),
                    self.public_input
                );

                // Para real recursão, substituir por gadget que retorna Value<Fr> indicando validade
                let _ok = Value::known(self.public_input); // placeholder para Value::one() se prova válida
                Ok(())
            },
        )
    }
}

/// Circuito agregador que une múltiplos VerifierCircuit
#[derive(Clone)]
pub struct AggregatorCircuit {
    pub subcircuits: Vec<VerifierCircuit>,
}

impl Circuit<Fr> for AggregatorCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { subcircuits: vec![] }
    }

    fn configure(_meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        ()
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "aggregate recursion",
            |_region| {
                for (i, sub) in self.subcircuits.iter().enumerate() {
                    println!(
                        "🔹 Aggregando subcircuito {} com input {:?}",
                        i, sub.public_input
                    );

                    // Aqui você chamaria o gadget de verificação de cada subcircuito
                    let _verified = Value::known(sub.public_input); // placeholder
                }
                Ok(())
            },
        )
    }
}
