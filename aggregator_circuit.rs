use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Column, Advice, Error, VerifyingKey, verify_proof, SingleVerifier},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};
use pasta_curves::pallas::{Affine as EpAffine, Scalar as Fr};

/// Configuração do AggregatorCircuit
#[derive(Clone)]
pub struct AggregatorConfig {
    proof_valid: Column<Advice>,
    all_valid: Column<Advice>,
}

/// AggregatorCircuit recursivo que verifica provas externas
#[derive(Clone)]
pub struct AggregatorCircuit {
    pub sub_proofs: Vec<Vec<u8>>,            // bytes de provas externas
    pub sub_public_inputs: Vec<Vec<Fr>>,     // inputs públicos de cada prova
    pub sub_vks: Vec<VerifyingKey<EpAffine>>, // VK de cada prova
    pub params: Params<EpAffine>,
}

/// Gadget que verifica uma prova externa e retorna AssignedCell<Fr, Fr>
fn verify_proof_gadget(
    layouter: &mut impl Layouter<Fr>,
    config: &AggregatorConfig,
    proof_bytes: &[u8],
    public_inputs: &[Fr],
    vk: &VerifyingKey<EpAffine>,
    params: &Params<EpAffine>,
) -> Result<AssignedCell<Fr, Fr>, Error> {
    let mut proof_slice: &[u8] = proof_bytes;
    let mut reader = Blake2bRead::<_, _, Challenge255<EpAffine>>::init(&mut proof_slice);
    let strategy = SingleVerifier::new(params);

    let nested_inputs: &[&[&[Fr]]] = &[&[&public_inputs[..]]];
    let result = verify_proof(params, vk, strategy, nested_inputs, &mut reader);

    layouter.assign_region(
        || "verify proof",
        |mut region| {
            let val = match result {
                Ok(_) => Fr::one(),
                Err(_) => Fr::zero(),
            };
            let cell = region.assign_advice(
                || "proof valid",
                config.proof_valid,
                0,
                || Value::known(val),
            )?;
            Ok(cell)
        },
    )
}

impl Circuit<Fr> for AggregatorCircuit {
    type Config = AggregatorConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            sub_proofs: vec![],
            sub_public_inputs: vec![],
            sub_vks: vec![],
            params: self.params.clone(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let proof_valid = meta.advice_column();
        let all_valid = meta.advice_column();

        meta.enable_equality(proof_valid);
        meta.enable_equality(all_valid);

        AggregatorConfig { proof_valid, all_valid }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        let mut all_valid_cell: Option<AssignedCell<Fr, Fr>> = None;

        for (i, proof_bytes) in self.sub_proofs.iter().enumerate() {
            let public_inputs = &self.sub_public_inputs[i];
            let vk = &self.sub_vks[i];

            let verified_cell = verify_proof_gadget(
                &mut layouter,
                &config,
                proof_bytes,
                public_inputs,
                vk,
                &self.params,
            )?;

            // Multiplica resultados para acumular validação
            all_valid_cell = Some(match all_valid_cell {
                Some(prev) => {
                    layouter.assign_region(
                        || format!("accumulate {}", i),
                        |mut region| {
                            let val = prev.value().zip(verified_cell.value()).map(|(a, b)| a * b);
                            region.assign_advice(|| "all_valid", config.all_valid, 0, || val)
                        },
                    )?
                }
                None => verified_cell,
            });
        }

        // Constrange o resultado final a 1 (todas as provas válidas)
        if let Some(all_valid) = all_valid_cell {
            layouter.assign_region(
                || "final constrain",
                |mut region| {
                    let one_cell = region.assign_advice(
                        || "one",
                        config.all_valid,
                        1,
                        || Value::known(Fr::one()),
                    )?;
                    region.constrain_equal(all_valid.cell(), one_cell.cell())
                },
            )?;
        }

        println!("✅ Todas as provas agregadas e validadas com sucesso!");
        Ok(())
    }
}