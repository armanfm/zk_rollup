use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, create_proof, keygen_pk, keygen_vk, ProvingKey, VerifyingKey, ConstraintSystem, Error, SingleVerifier, verify_proof},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Blake2bRead, Challenge255},
};
use pasta_curves::pallas::{Affine as EpAffine, Base as Fp, Scalar as Fr};
use rand_core::OsRng;
use std::fs::File;
use std::io::{Read, BufWriter, Write};

// ---------------- PaiCircuit ----------------
#[derive(Clone)]
pub struct PaiCircuit {
    pub x: Option<Fp>,
    pub y: Option<Fp>,
}

#[derive(Clone, Debug)]
pub struct PaiConfig {
    x: halo2_proofs::plonk::Column<halo2_proofs::plonk::Advice>,
    y: halo2_proofs::plonk::Column<halo2_proofs::plonk::Advice>,
    s_mul2: halo2_proofs::plonk::Selector,
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
            let x_val = meta.query_advice(x, halo2_proofs::poly::Rotation::cur());
            let y_val = meta.query_advice(y, halo2_proofs::poly::Rotation::cur());
            let two_x = x_val.clone() + x_val.clone();
            vec![s * (y_val - two_x)]
        });

        PaiConfig { x, y, s_mul2 }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_region(
            || "mul2",
            |mut region| {
                config.s_mul2.enable(&mut region, 0)?;
                let x_val = self.x.ok_or(Error::Synthesis)?;
                let y_val = self.y.unwrap_or(x_val + x_val);

                region.assign_advice(|| "x", config.x, 0, || Value::known(x_val))?;
                region.assign_advice(|| "y", config.y, 0, || Value::known(y_val))?;
                Ok(())
            },
        )
    }
}

// ---------------- VerifierCircuit ----------------
#[derive(Clone)]
pub struct VerifierCircuit {
    pub proof_bytes: Vec<u8>,
    pub public_input: Fp,
    pub vk: VerifyingKey<EpAffine>,
    pub params: Params<EpAffine>,
}

impl Circuit<Fp> for VerifierCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            proof_bytes: vec![],
            public_input: Fp::zero(),
            vk: self.vk.clone(),
            params: self.params.clone(),
        }
    }

    fn configure(_meta: &mut ConstraintSystem<Fp>) -> Self::Config { () }

    fn synthesize(&self, _config: Self::Config, _layouter: impl Layouter<Fp>) -> Result<(), Error> {
        // Prepara array público para a verificação
        let binding = [self.public_input];
        let instances_ref: Vec<&[Fp]> = vec![&binding];
        let instances: &[&[&[Fp]]] = &[&instances_ref];

        let mut transcript = Blake2bRead::<_, _, Challenge255<EpAffine>>::init(&self.proof_bytes[..]);
        let strategy = SingleVerifier::new(&self.params);

        verify_proof(&self.params, &self.vk, strategy, instances, &mut transcript)
            .map_err(|_| Error::Synthesis)?;
        println!("✅ Prova externa verificada: {:?}", self.public_input);
        Ok(())
    }
}

// ---------------- AggregatorCircuit ----------------
#[derive(Clone)]
pub struct AggregatorCircuit {
    pub subcircuits: Vec<VerifierCircuit>,
    pub params: Params<EpAffine>,
}

impl Circuit<Fp> for AggregatorCircuit {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { subcircuits: vec![], params: self.params.clone() }
    }

    fn configure(_meta: &mut ConstraintSystem<Fp>) -> Self::Config { () }

    fn synthesize(&self, _config: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        for (i, sub) in self.subcircuits.iter().enumerate() {
            sub.synthesize((), layouter.namespace(|| format!("subcircuit {}", i)))?;
        }
        Ok(())
    }
}

// ---------------- Main ----------------
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let k = 8;
    let params: Params<EpAffine> = Params::new(k);

    // 1️⃣ Gerar provas individuais do PaiCircuit
    let pai_inputs = vec![Fp::from(3), Fp::from(7), Fp::from(11)];
    let mut proofs = vec![];
    let mut vks = vec![];

    for &x in &pai_inputs {
        let circuit = PaiCircuit { x: Some(x), y: None };
        let vk = keygen_vk(&params, &circuit)?;
        let pk = keygen_pk(&params, vk.clone(), &circuit)?;

        let mut proof_bytes = Vec::new();
        let mut transcript = Blake2bWrite::<_, _, Challenge255<EpAffine>>::init(&mut proof_bytes);
        create_proof(&params, &pk, &[circuit], &[], &mut OsRng, &mut transcript)?;
        proofs.push(proof_bytes);
        vks.push(vk);
    }

    // 2️⃣ Criar VerifierCircuit para cada prova
    let subcircuits: Vec<VerifierCircuit> = proofs
        .into_iter()
        .zip(pai_inputs.iter())
        .zip(vks.into_iter())
        .map(|((proof, &x), vk)| VerifierCircuit { proof_bytes: proof, public_input: x, vk, params: params.clone() })
        .collect();

    // 3️⃣ AggregatorCircuit
    let agg_circuit = AggregatorCircuit { subcircuits, params: params.clone() };

    // 4️⃣ Criar prova agregada
    let vk_agg = keygen_vk(&params, &agg_circuit)?;
    let pk_agg = keygen_pk(&params, vk_agg.clone(), &agg_circuit)?;

    let mut proof_final = Vec::new();
    let mut transcript = Blake2bWrite::<_, _, Challenge255<EpAffine>>::init(&mut proof_final);
    create_proof(&params, &pk_agg, &[agg_circuit], &[], &mut OsRng, &mut transcript)?;

    let mut file = BufWriter::new(File::create("aggregated_proof.bin")?);
    file.write_all(&proof_final)?;

    println!("✅ Prova agregada gerada com {} bytes", proof_final.len());
    Ok(())
}
